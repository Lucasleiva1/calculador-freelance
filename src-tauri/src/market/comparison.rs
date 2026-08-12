use std::collections::HashSet;

use chrono::{DateTime, Duration, NaiveDate, Utc};

use crate::models::MarketObservation;

use super::types::{ComparableObservation, ComparisonSummary, MarketQueryContext};

fn midpoint(observation: &MarketObservation) -> Option<i64> {
    observation
        .price_value_minor
        .or_else(|| {
            observation
                .price_min_minor
                .zip(observation.price_max_minor)
                .map(|(min, max)| min + (max - min) / 2)
        })
        .or(observation.price_min_minor)
        .or(observation.price_max_minor)
}

fn convert(value: i64, from: &str, to: &str, usd_to_ars_micros: Option<i64>) -> Option<i64> {
    if from == to {
        return Some(value);
    }
    // Pricing OS conserva el contrato histórico: 4 decimales de ARS por USD.
    let rate = usd_to_ars_micros? as f64 / 10_000.0;
    if rate <= 0.0 {
        return None;
    }
    match (from, to) {
        ("USD", "ARS") => Some((value as f64 * rate).round() as i64),
        ("ARS", "USD") => Some((value as f64 / rate).round() as i64),
        _ => None,
    }
}

fn project_value(
    observation: &MarketObservation,
    context: &MarketQueryContext,
    currency: &str,
    rate: Option<i64>,
) -> Result<i64, String> {
    let original = midpoint(observation).ok_or_else(|| "Sin precio utilizable.".to_string())?;
    let value = convert(original, &observation.currency, currency, rate).ok_or_else(|| {
        format!(
            "No existe una conversión auditable {}→{}.",
            observation.currency, currency
        )
    })?;
    match observation.price_type.as_str() {
        "PROJECT" | "FIXED" | "RANGE" => Ok(value),
        "PER_MINUTE" => context
            .duration_minutes
            .filter(|duration| *duration > 0.0)
            .map(|duration| {
                (value as f64 * duration * context.quantity.unwrap_or(1.0).max(1.0)).round() as i64
            })
            .ok_or_else(|| "Falta duración final para normalizar precio por minuto.".into()),
        "PER_ITEM" => context
            .quantity
            .filter(|quantity| *quantity > 0.0)
            .map(|quantity| (value as f64 * quantity).round() as i64)
            .ok_or_else(|| "Falta cantidad para normalizar precio por unidad.".into()),
        "HOURLY" => context
            .estimated_hours
            .filter(|hours| *hours > 0.0)
            .map(|hours| (value as f64 * hours).round() as i64)
            .ok_or_else(|| "Faltan horas para normalizar tarifa horaria.".into()),
        "DAILY" => Err(
            "La tarifa diaria conserva su unidad y no se convierte automáticamente a proyecto."
                .into(),
        ),
        "MONTHLY_SALARY" | "ANNUAL_SALARY" => {
            Err("El salario es contexto separado y no participa en la referencia freelance.".into())
        }
        _ => Err("Tipo de precio no comparable.".into()),
    }
}

fn percentile(values: &[i64], percentile: f64) -> Option<i64> {
    if values.is_empty() {
        return None;
    }
    let position = (values.len() - 1) as f64 * percentile;
    let low = position.floor() as usize;
    let high = position.ceil() as usize;
    if low == high {
        Some(values[low])
    } else {
        let fraction = position - low as f64;
        Some((values[low] as f64 + (values[high] - values[low]) as f64 * fraction).round() as i64)
    }
}

fn is_recent(value: Option<&str>) -> bool {
    let cutoff = Utc::now() - Duration::days(730);
    value
        .and_then(|raw| {
            DateTime::parse_from_rfc3339(raw)
                .ok()
                .map(|date| date.with_timezone(&Utc))
                .or_else(|| {
                    NaiveDate::parse_from_str(raw, "%Y-%m-%d")
                        .ok()
                        .and_then(|date| date.and_hms_opt(0, 0, 0).map(|date| date.and_utc()))
                })
        })
        .is_some_and(|date| date >= cutoff)
}

fn region_matches(observation: &str, targets: &[String]) -> bool {
    targets.iter().any(|target| {
        target == observation
            || (target == "INTERNATIONAL"
                && matches!(observation, "GLOBAL" | "INTERNATIONAL"))
            || (target == "LATAM" && matches!(observation, "AR" | "LATAM"))
    })
}

fn level_matches(observation: &MarketObservation, context: &MarketQueryContext) -> bool {
    let Some(target) = context.level.as_deref() else {
        return true;
    };
    if target == "custom" {
        return true;
    }
    let Some(level) = observation.experience_level.as_deref() else {
        return true;
    };
    let level = level.to_lowercase();
    match target {
        "basic" | "low" => ["junior", "entry", "principiante"]
            .iter()
            .any(|value| level.contains(value)),
        "professional" | "medium" => ["mid", "intermediate", "intermedio"]
            .iter()
            .any(|value| level.contains(value)),
        "advanced" | "high" => ["senior", "expert", "lead"]
            .iter()
            .any(|value| level.contains(value)),
        _ => true,
    }
}

fn subtype_matches(observation: &MarketObservation, context: &MarketQueryContext) -> bool {
    if matches!(
        observation.price_type.as_str(),
        "HOURLY" | "PER_MINUTE" | "PER_ITEM"
    ) {
        return true;
    }
    let Some(subtype) = context.subtype.as_deref() else {
        return true;
    };
    if subtype == "custom" {
        return true;
    }
    let description = format!(
        "{} {}",
        observation.subservice.as_deref().unwrap_or_default(),
        observation.category.as_deref().unwrap_or_default()
    )
    .to_lowercase();
    if description.contains(subtype) {
        return true;
    }
    let aliases: &[&str] = match subtype {
        "reel-short" => &["reel", "short", "social"],
        "youtube" => &["youtube"],
        "advertising" => &["publicidad", "advertising", "commercial", "spot"],
        "institutional" => &["institucional", "institutional", "corporate"],
        "podcast" => &["podcast"],
        "videoclip" => &["videoclip", "music video"],
        "landing" => &["landing"],
        "web" => &["website", "sitio web", "web site", "web development"],
        "desktop" => &["desktop"],
        "dashboard" => &["dashboard", "panel"],
        "internal" => &["herramienta interna", "internal tool"],
        "automation" => &["automatización", "automation"],
        "ai" => &["inteligencia artificial", "artificial intelligence", " ai "],
        _ => &[],
    };
    aliases.iter().any(|alias| description.contains(alias))
}

pub fn compare_market(
    observations: &[MarketObservation],
    context: &MarketQueryContext,
    currency: &str,
    usd_to_ars_micros: Option<i64>,
    participating_source_ids: &HashSet<String>,
) -> (Vec<ComparableObservation>, ComparisonSummary) {
    let mut comparable = Vec::new();
    let mut salary_excluded = 0;
    let mut recent_count = 0;
    let mut reliable_count = 0;
    for observation in observations {
        if observation.service_type != context.service && observation.service_type != "currency" {
            continue;
        }
        if matches!(
            observation.price_type.as_str(),
            "MONTHLY_SALARY" | "ANNUAL_SALARY"
        ) {
            salary_excluded += 1;
        }
        let observation_date = observation
            .published_at
            .as_deref()
            .or(Some(&observation.retrieved_at));
        let normalized = if !is_recent(observation_date) {
            Err("La referencia tiene más de dos años y se conserva sólo como historial.".into())
        } else if !participating_source_ids.contains(&observation.source_id) {
            Err("La fuente aporta contexto, pero no participa en sugerencias.".into())
        } else if !region_matches(&observation.region, &context.region_targets) {
            Err("La región no coincide con el objetivo de esta cotización.".into())
        } else if !level_matches(observation, context) {
            Err("El nivel profesional no coincide con la complejidad elegida.".into())
        } else if !subtype_matches(observation, context) {
            Err("El subservicio no coincide con el alcance concreto de la cotización.".into())
        } else if observation.comparison_eligibility == "ELIGIBLE" {
            project_value(observation, context, currency, usd_to_ars_micros)
        } else {
            Err(observation
                .exclusion_reason
                .clone()
                .unwrap_or_else(|| "La fuente clasificó el dato como contexto.".into()))
        };
        match normalized {
            Ok(value) => {
                if is_recent(observation_date) {
                    recent_count += 1;
                }
                if matches!(observation.confidence.as_str(), "HIGH" | "MEDIUM") {
                    reliable_count += 1;
                }
                comparable.push(ComparableObservation {
                    observation_id: observation.id.clone(),
                    source_id: observation.source_id.clone(),
                    normalized_value_minor: Some(value),
                    included: true,
                    reason: None,
                })
            }
            Err(reason) => comparable.push(ComparableObservation {
                observation_id: observation.id.clone(),
                source_id: observation.source_id.clone(),
                normalized_value_minor: None,
                included: false,
                reason: Some(reason),
            }),
        }
    }
    let mut initial = comparable
        .iter()
        .filter(|item| item.included)
        .filter_map(|item| item.normalized_value_minor)
        .collect::<Vec<_>>();
    initial.sort_unstable();
    if initial.len() >= 8 {
        if let (Some(p25), Some(p75)) = (percentile(&initial, 0.25), percentile(&initial, 0.75)) {
            let iqr = p75 - p25;
            let low = p25 - (iqr as f64 * 1.5).round() as i64;
            let high = p75 + (iqr as f64 * 1.5).round() as i64;
            for item in &mut comparable {
                if item.included
                    && item
                        .normalized_value_minor
                        .is_some_and(|value| value < low || value > high)
                {
                    item.included = false;
                    item.reason = Some("Posible outlier excluido mediante regla IQR 1,5×.".into());
                }
            }
        }
    }
    let mut values = comparable
        .iter()
        .filter(|item| item.included)
        .filter_map(|item| item.normalized_value_minor)
        .collect::<Vec<_>>();
    values.sort_unstable();
    let sources = comparable
        .iter()
        .filter(|item| item.included)
        .map(|item| item.source_id.as_str())
        .collect::<HashSet<_>>()
        .len() as i64;
    let confidence = if values.len() >= 10 && sources >= 3 && recent_count >= 5 {
        "HIGH"
    } else if values.len() >= 5 && sources >= 2 {
        "MEDIUM"
    } else if values.len() >= 3 || (!values.is_empty() && reliable_count > 0) {
        "LOW"
    } else {
        "INSUFFICIENT"
    };
    let explanations = vec![
        format!("{} observaciones comparables", values.len()),
        format!("{} fuentes comparables", sources),
        format!("{} datos recientes", recent_count),
        format!("{} referencias salariales excluidas", salary_excluded),
    ];
    let summary = ComparisonSummary {
        minimum_filtered_minor: values.first().copied(),
        p25_minor: percentile(&values, 0.25),
        median_minor: percentile(&values, 0.5),
        p75_minor: percentile(&values, 0.75),
        maximum_filtered_minor: values.last().copied(),
        confidence_level: confidence.into(),
        comparable_count: values.len() as i64,
        source_count: sources,
        recent_count,
        salary_excluded_count: salary_excluded,
        explanations,
    };
    (comparable, summary)
}

pub fn suggested_with_market(
    calculated: Option<i64>,
    summary: &ComparisonSummary,
    strategy: &str,
) -> Option<i64> {
    if summary.confidence_level == "INSUFFICIENT" {
        return None;
    }
    let market_target = match strategy {
        "competitive" => summary.p25_minor,
        "premium" => summary.p75_minor,
        _ => summary.median_minor,
    }?;
    Some(match calculated {
        Some(internal) => {
            internal.max(((internal as f64 * 0.4) + (market_target as f64 * 0.6)).round() as i64)
        }
        None => market_target,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn observation(id: &str, price_type: &str, value: i64) -> MarketObservation {
        MarketObservation {
            id: id.into(),
            source_id: format!("source-{id}"),
            source_name: id.into(),
            origin: "MANUAL".into(),
            service_type: "programming".into(),
            subservice: None,
            category: None,
            region: "INTERNATIONAL".into(),
            country: None,
            currency: "USD".into(),
            price_type: price_type.into(),
            unit: "por proyecto".into(),
            price_min_minor: None,
            price_max_minor: None,
            price_value_minor: Some(value),
            original_value_text: value.to_string(),
            converted_value_minor: None,
            converted_currency: None,
            exchange_rate_micros: None,
            exchange_rate_date: None,
            exchange_rate_source: None,
            experience_level: None,
            client_tier: None,
            source_type: "rate_benchmark".into(),
            source_url: "https://example.com".into(),
            published_at: Some(Utc::now().date_naive().to_string()),
            retrieved_at: Utc::now().to_rfc3339(),
            parser_version: "test".into(),
            confidence: "HIGH".into(),
            comparison_eligibility: if price_type.contains("SALARY") {
                "CONTEXT_ONLY".into()
            } else {
                "ELIGIBLE".into()
            },
            exclusion_reason: None,
            raw_fingerprint: id.into(),
            evidence_snippet: None,
            notes: None,
            created_at: Utc::now().to_rfc3339(),
            snapshot_included: None,
            snapshot_exclusion_reason: None,
            snapshot_normalized_value_minor: None,
        }
    }

    #[test]
    fn salary_never_participates_in_freelance_median() {
        let observations = vec![
            observation("a", "PROJECT", 50_000),
            observation("b", "PROJECT", 70_000),
            observation("salary", "MONTHLY_SALARY", 900_000),
        ];
        let (_, summary) = compare_market(
            &observations,
            &MarketQueryContext::generic("programming".into(), vec!["INTERNATIONAL".into()]),
            "USD",
            None,
            &observations
                .iter()
                .map(|item| item.source_id.clone())
                .collect(),
        );
        assert_eq!(summary.median_minor, Some(60_000));
        assert_eq!(summary.salary_excluded_count, 1);
    }

    #[test]
    fn context_only_source_cannot_influence_the_reference() {
        let observations = vec![
            observation("included", "PROJECT", 50_000),
            observation("context", "PROJECT", 900_000),
        ];
        let participating = HashSet::from(["source-included".to_string()]);
        let (items, summary) = compare_market(
            &observations,
            &MarketQueryContext::generic("programming".into(), vec!["INTERNATIONAL".into()]),
            "USD",
            None,
            &participating,
        );
        assert_eq!(summary.median_minor, Some(50_000));
        assert_eq!(summary.comparable_count, 1);
        assert!(items.iter().any(|item| {
            item.source_id == "source-context"
                && item.reason.as_deref()
                    == Some("La fuente aporta contexto, pero no participa en sugerencias.")
        }));
    }

    #[test]
    fn project_reference_for_another_subservice_is_excluded() {
        let mut dashboard = observation("dashboard", "PROJECT", 300_000);
        dashboard.subservice = Some("Dashboard con autenticación".into());
        let observations = vec![dashboard];
        let participating = HashSet::from(["source-dashboard".to_string()]);
        let context = MarketQueryContext {
            subtype: Some("landing".into()),
            ..MarketQueryContext::generic("programming".into(), vec!["INTERNATIONAL".into()])
        };
        let (items, summary) = compare_market(&observations, &context, "USD", None, &participating);
        assert_eq!(summary.comparable_count, 0);
        assert_eq!(
            items[0].reason.as_deref(),
            Some("El subservicio no coincide con el alcance concreto de la cotización.")
        );
    }

    #[test]
    fn references_older_than_two_years_remain_history_only() {
        let mut stale = observation("stale", "HOURLY", 5_000);
        stale.published_at = Some("2020-01-01".into());
        let observations = vec![stale];
        let participating = HashSet::from(["source-stale".to_string()]);
        let context = MarketQueryContext {
            estimated_hours: Some(10.0),
            ..MarketQueryContext::generic("programming".into(), vec!["INTERNATIONAL".into()])
        };
        let (items, summary) = compare_market(&observations, &context, "USD", None, &participating);
        assert_eq!(summary.comparable_count, 0);
        assert_eq!(
            items[0].reason.as_deref(),
            Some("La referencia tiene más de dos años y se conserva sólo como historial.")
        );
    }

    #[test]
    fn global_reference_never_counts_as_argentina() {
        let global = observation("global", "HOURLY", 5_000);
        let observations = vec![global];
        let participating = HashSet::from(["source-global".to_string()]);
        let context = MarketQueryContext {
            estimated_hours: Some(10.0),
            ..MarketQueryContext::generic("programming".into(), vec!["AR".into()])
        };
        let (items, summary) = compare_market(&observations, &context, "USD", None, &participating);
        assert_eq!(summary.comparable_count, 0);
        assert_eq!(
            items[0].reason.as_deref(),
            Some("La región no coincide con el objetivo de esta cotización.")
        );
    }

    #[test]
    fn argentina_reference_never_counts_as_international() {
        let mut argentina = observation("argentina", "HOURLY", 1_000);
        argentina.region = "AR".into();
        let observations = vec![argentina];
        let participating = HashSet::from(["source-argentina".to_string()]);
        let context = MarketQueryContext {
            estimated_hours: Some(10.0),
            ..MarketQueryContext::generic("programming".into(), vec!["INTERNATIONAL".into()])
        };
        let (_, summary) = compare_market(&observations, &context, "USD", None, &participating);
        assert_eq!(summary.comparable_count, 0);
    }

    #[test]
    fn complexity_selects_the_matching_experience_band() {
        let mut junior = observation("junior", "HOURLY", 1_000);
        junior.experience_level = Some("Junior".into());
        let mut senior = observation("senior", "HOURLY", 5_000);
        senior.experience_level = Some("Senior".into());
        let observations = vec![junior, senior];
        let participating = observations
            .iter()
            .map(|item| item.source_id.clone())
            .collect();
        let context = MarketQueryContext {
            level: Some("basic".into()),
            estimated_hours: Some(10.0),
            ..MarketQueryContext::generic("programming".into(), vec!["INTERNATIONAL".into()])
        };
        let (_, summary) = compare_market(&observations, &context, "USD", None, &participating);
        assert_eq!(summary.median_minor, Some(10_000));
        assert_eq!(summary.confidence_level, "LOW");
    }
}
