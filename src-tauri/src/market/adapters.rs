use regex::Regex;
use scraper::{Html, Selector};
use serde_json::Value;

use crate::{
    error::{AppError, AppResult},
    models::MarketSource,
};

use super::{
    normalization::{
        detect_price_type, extract_numeric_tokens, parse_localized_minor, parse_range_minor,
    },
    types::{MarketQueryContext, ObservationDraft},
};

pub trait SourceAdapter {
    fn key(&self) -> &'static str;
    fn extract(
        &self,
        body: &str,
        source: &MarketSource,
        context: &MarketQueryContext,
        final_url: &str,
    ) -> AppResult<Vec<ObservationDraft>>;
}

pub fn extract_with_adapter(
    body: &str,
    source: &MarketSource,
    context: &MarketQueryContext,
    final_url: &str,
) -> AppResult<Vec<ObservationDraft>> {
    let adapter: Box<dyn SourceAdapter> = match source.adapter_key.as_deref() {
        Some("tarifario") => Box::new(TarifarioAdapter),
        Some("yunojuno") => Box::new(YunoJunoAdapter),
        Some("remotejobs") => Box::new(RemoteJobsAdapter),
        Some("bcra") => Box::new(BcraAdapter),
        Some("generic") | None => Box::new(GenericAdapter),
        Some(_) => {
            return Err(AppError::Validation(
                "La fuente no tiene un adapter compatible.".into(),
            ))
        }
    };
    debug_assert!(!adapter.key().is_empty());
    adapter.extract(body, source, context, final_url)
}

fn html_text(body: &str) -> String {
    Html::parse_document(body)
        .root_element()
        .text()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

struct TarifarioAdapter;

impl SourceAdapter for TarifarioAdapter {
    fn key(&self) -> &'static str {
        "tarifario"
    }

    fn extract(
        &self,
        body: &str,
        _source: &MarketSource,
        context: &MarketQueryContext,
        final_url: &str,
    ) -> AppResult<Vec<ObservationDraft>> {
        let document = Html::parse_document(body);
        let rows = Selector::parse("tr").expect("valid selector");
        let cells = Selector::parse("th, td").expect("valid selector");
        let mut result = Vec::new();
        for row in document.select(&rows) {
            let values = row
                .select(&cells)
                .map(|cell| cell.text().collect::<Vec<_>>().join(" ").trim().to_string())
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>();
            if values.len() < 4 {
                continue;
            }
            let service = values[0].to_lowercase();
            let relevant = if context.service == "video-editing" {
                service.contains("edición de video")
                    || service.contains("edicion de video")
                    || (context.subtype.as_deref() == Some("advertising")
                        && service.contains("spot publicitario"))
            } else {
                service.contains("diseño") || service.contains("desarrollo")
            };
            if !relevant {
                continue;
            }
            let (price_type, unit) = detect_price_type(&values[0]);
            for (index, tier) in ["A", "B", "C"].into_iter().enumerate() {
                let raw = values.get(index + 1).cloned().unwrap_or_default();
                let Some(price) = parse_localized_minor(&raw, "ARS", "es-AR") else {
                    continue;
                };
                result.push(ObservationDraft {
                    service_type: context.service.clone(),
                    subservice: Some(values[0].clone()),
                    category: Some("Video".into()),
                    region: "AR".into(),
                    country: Some("Argentina".into()),
                    currency: "ARS".into(),
                    price_type: price_type.into(),
                    unit: unit.into(),
                    price_min_minor: None,
                    price_max_minor: None,
                    price_value_minor: Some(price),
                    original_value_text: raw,
                    experience_level: None,
                    client_tier: Some(tier.into()),
                    source_url: final_url.into(),
                    published_at: Some("2025-12-01".into()),
                    confidence: "HIGH".into(),
                    comparison_eligibility: if price_type == "UNKNOWN" {
                        "REVIEW_REQUIRED".into()
                    } else {
                        "ELIGIBLE".into()
                    },
                    exclusion_reason: None,
                    evidence_snippet: Some(values.join(" · ").chars().take(300).collect()),
                    notes: Some(
                        "Cliente A/B/C conserva la clasificación original de Tarifario.org.".into(),
                    ),
                });
            }
        }
        Ok(result)
    }
}

struct YunoJunoAdapter;

impl SourceAdapter for YunoJunoAdapter {
    fn key(&self) -> &'static str {
        "yunojuno"
    }

    fn extract(
        &self,
        body: &str,
        _source: &MarketSource,
        context: &MarketQueryContext,
        final_url: &str,
    ) -> AppResult<Vec<ObservationDraft>> {
        let text = html_text(body);
        let role = if context.service == "video-editing" {
            "Video Editor"
        } else {
            "Developer"
        };
        let role_regex =
            Regex::new(&format!(r"(?i){}[^.]*", regex::escape(role))).expect("role regex");
        let price_regex =
            Regex::new(r"(?i)(?:USD\s*|\$|£)([0-9][0-9,\.]*)\s*(?:/|per\s*)?(hour|day)?")
                .expect("price regex");
        let year_regex = Regex::new(r"20[0-9]{2}").expect("year regex");
        let mut result = Vec::new();
        for role_match in role_regex.find_iter(&text) {
            let mut start = role_match.start().saturating_sub(80);
            let mut end = (role_match.end() + 180).min(text.len());
            while start > 0 && !text.is_char_boundary(start) {
                start -= 1;
            }
            while end < text.len() && !text.is_char_boundary(end) {
                end += 1;
            }
            let window = &text[start..end];
            for capture in price_regex.captures_iter(window) {
                let full = capture
                    .get(0)
                    .map(|value| value.as_str())
                    .unwrap_or_default();
                let numeric = capture
                    .get(1)
                    .map(|value| value.as_str())
                    .unwrap_or_default();
                let currency = if full.contains('£') { "GBP" } else { "USD" };
                let lower = window.to_lowercase();
                let unit_token = capture.get(2).map(|value| value.as_str().to_lowercase());
                let (price_type, unit) =
                    if unit_token.as_deref() == Some("hour") || lower.contains("hourly") {
                        ("HOURLY", "por hora")
                    } else {
                        ("DAILY", "por día")
                    };
                let locale = if currency == "GBP" { "en-GB" } else { "en-US" };
                let Some(price) = parse_localized_minor(numeric, currency, locale) else {
                    continue;
                };
                let year = year_regex
                    .find(window)
                    .map(|year| format!("{}-01-01", year.as_str()));
                result.push(ObservationDraft {
                    service_type: context.service.clone(), subservice: Some(role.into()), category: Some("Film & Motion".into()), region: "INTERNATIONAL".into(), country: Some("United Kingdom".into()),
                    currency: currency.into(), price_type: price_type.into(), unit: unit.into(), price_min_minor: None, price_max_minor: None, price_value_minor: Some(price), original_value_text: full.into(),
                    experience_level: if lower.contains("senior") { Some("Senior".into()) } else { None }, client_tier: None, source_url: final_url.into(), published_at: year,
                    confidence: "MEDIUM".into(), comparison_eligibility: if price_type == "DAILY" { "CONTEXT_ONLY".into() } else { "ELIGIBLE".into() },
                    exclusion_reason: (price_type == "DAILY").then_some("La tarifa diaria se conserva en su unidad original y no se transforma automáticamente en precio de proyecto.".into()),
                    evidence_snippet: Some(window.chars().take(300).collect()), notes: None,
                });
            }
        }
        result.sort_by_key(|item| (item.price_type.clone(), item.price_value_minor));
        result.dedup_by_key(|item| (item.price_type.clone(), item.price_value_minor));
        Ok(result)
    }
}

struct RemoteJobsAdapter;

impl SourceAdapter for RemoteJobsAdapter {
    fn key(&self) -> &'static str {
        "remotejobs"
    }

    fn extract(
        &self,
        body: &str,
        _source: &MarketSource,
        context: &MarketQueryContext,
        final_url: &str,
    ) -> AppResult<Vec<ObservationDraft>> {
        let document = Html::parse_document(body);
        let rows = Selector::parse("tr").expect("valid selector");
        let cells = Selector::parse("th, td").expect("valid selector");
        let mut observations = Vec::new();
        for row in document.select(&rows) {
            let values = row
                .select(&cells)
                .map(|cell| cell.text().collect::<Vec<_>>().join(" ").trim().to_string())
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>();
            if values.len() < 2 {
                continue;
            }
            let role = &values[0];
            if ![
                "javascript",
                "react",
                "node",
                "typescript",
                "python",
                "rust",
                "full stack",
            ]
            .iter()
            .any(|needle| role.to_lowercase().contains(needle))
            {
                continue;
            }
            for (index, level) in
                values
                    .iter()
                    .skip(1)
                    .zip(["Junior", "Mid", "Senior", "Staff/Lead"])
            {
                let Some((price_min, price_max)) = parse_range_minor(index, "USD", "en-US") else {
                    continue;
                };
                observations.push(ObservationDraft {
                    service_type: context.service.clone(),
                    subservice: Some(role.clone()),
                    category: Some("Remote technology salary".into()),
                    region: "LATAM".into(),
                    country: None,
                    currency: "USD".into(),
                    price_type: "MONTHLY_SALARY".into(),
                    unit: "por mes".into(),
                    price_min_minor: Some(price_min),
                    price_max_minor: Some(price_max),
                    price_value_minor: (price_min == price_max).then_some(price_min),
                    original_value_text: index.clone(),
                    experience_level: Some(level.into()),
                    client_tier: None,
                    source_url: final_url.into(),
                    published_at: None,
                    confidence: "MEDIUM".into(),
                    comparison_eligibility: "CONTEXT_ONLY".into(),
                    exclusion_reason: Some(
                        "Es salario mensual de empleo remoto, no una tarifa freelance directa."
                            .into(),
                    ),
                    evidence_snippet: Some(format!("{} · {} · {}", role, level, index)),
                    notes: None,
                });
            }
        }
        Ok(observations)
    }
}

struct BcraAdapter;

impl SourceAdapter for BcraAdapter {
    fn key(&self) -> &'static str {
        "bcra"
    }

    fn extract(
        &self,
        body: &str,
        _source: &MarketSource,
        _context: &MarketQueryContext,
        final_url: &str,
    ) -> AppResult<Vec<ObservationDraft>> {
        let value: Value = serde_json::from_str(body)?;
        let result = value
            .get("results")
            .ok_or_else(|| AppError::Validation("Respuesta BCRA sin resultados.".into()))?;
        let date = result
            .get("fecha")
            .and_then(Value::as_str)
            .map(str::to_string);
        let detail = result
            .get("detalle")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                AppError::Validation("Respuesta BCRA sin detalle de cotizaciones.".into())
            })?;
        let usd = detail
            .iter()
            .find(|item| item.get("codigoMoneda").and_then(Value::as_str) == Some("USD"))
            .ok_or_else(|| {
                AppError::Validation("BCRA no devolvió la cotización USD/ARS.".into())
            })?;
        let rate = usd
            .get("tipoCotizacion")
            .and_then(Value::as_f64)
            .filter(|value| value.is_finite() && *value > 0.0)
            .ok_or_else(|| AppError::Validation("Cotización USD/ARS inválida.".into()))?;
        Ok(vec![ObservationDraft {
            service_type: "currency".into(),
            subservice: Some("USD/ARS".into()),
            category: Some("Currency".into()),
            region: "AR".into(),
            country: Some("Argentina".into()),
            currency: "ARS".into(),
            price_type: "FIXED".into(),
            unit: "ARS por USD".into(),
            price_min_minor: None,
            price_max_minor: None,
            price_value_minor: Some((rate * 100.0).round() as i64),
            original_value_text: rate.to_string(),
            experience_level: None,
            client_tier: None,
            source_url: final_url.into(),
            published_at: date,
            confidence: "HIGH".into(),
            comparison_eligibility: "CONTEXT_ONLY".into(),
            exclusion_reason: Some("Es una cotización de moneda, no un precio de servicio.".into()),
            evidence_snippet: Some(format!("USD/ARS {rate}")),
            notes: Some("API oficial del BCRA sin autenticación.".into()),
        }])
    }
}

struct GenericAdapter;

impl SourceAdapter for GenericAdapter {
    fn key(&self) -> &'static str {
        "generic"
    }

    fn extract(
        &self,
        body: &str,
        _source: &MarketSource,
        context: &MarketQueryContext,
        final_url: &str,
    ) -> AppResult<Vec<ObservationDraft>> {
        let text = html_text(body);
        Ok(extract_numeric_tokens(&text)
            .into_iter()
            .take(8)
            .filter_map(|raw| {
                let currency = if raw.contains("ARS") {
                    "ARS"
                } else if raw.contains('£') {
                    "GBP"
                } else if raw.contains('€') {
                    "EUR"
                } else {
                    "USD"
                };
                let value = parse_localized_minor(
                    &raw,
                    currency,
                    if currency == "ARS" { "es-AR" } else { "en-US" },
                )?;
                Some(ObservationDraft {
                    service_type: context.service.clone(),
                    subservice: context.subtype.clone(),
                    category: None,
                    region: context
                        .region_targets
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "GLOBAL".into()),
                    country: None,
                    currency: currency.into(),
                    price_type: "UNKNOWN".into(),
                    unit: "sin unidad".into(),
                    price_min_minor: None,
                    price_max_minor: None,
                    price_value_minor: Some(value),
                    original_value_text: raw.clone(),
                    experience_level: None,
                    client_tier: None,
                    source_url: final_url.into(),
                    published_at: None,
                    confidence: "REVIEW_REQUIRED".into(),
                    comparison_eligibility: "REVIEW_REQUIRED".into(),
                    exclusion_reason: Some(
                        "Extractor genérico: requiere confirmación de unidad y significado.".into(),
                    ),
                    evidence_snippet: Some(raw),
                    notes: None,
                })
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(adapter: &str) -> MarketSource {
        MarketSource {
            id: format!("source-{adapter}"),
            name: adapter.into(),
            base_url: Some("https://example.com".into()),
            source_type: "rate_benchmark".into(),
            regions_json: "[]".into(),
            supported_services_json: "[]".into(),
            priority: 1,
            enabled: true,
            usage_mode: "market_price".into(),
            acquisition_mode: "auto_http".into(),
            cooldown_hours: Some(24),
            notes: None,
            is_system_source: true,
            system_key: Some(adapter.into()),
            default_data_json: None,
            purpose: None,
            data_contribution: None,
            app_benefit: None,
            participates_in_suggestions: true,
            automation_status: "APPROVED".into(),
            current_status: "READY".into(),
            adapter_key: Some(adapter.into()),
            last_request_at: None,
            last_success_at: None,
            last_failure_at: None,
            cooldown_until: None,
            consecutive_failures: 0,
            last_http_status: None,
            last_error: None,
            observation_count: 0,
            archived_at: None,
            created_at: "now".into(),
            updated_at: "now".into(),
        }
    }

    #[test]
    fn tarifario_extracts_client_tiers_and_ars_per_minute() {
        let html = include_str!("fixtures/tarifario.html");
        let rows = TarifarioAdapter
            .extract(
                html,
                &source("tarifario"),
                &MarketQueryContext::generic("video-editing".into(), vec!["AR".into()]),
                "https://tarifario.org/multimedia-c27",
            )
            .unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].client_tier.as_deref(), Some("A"));
        assert_eq!(rows[0].currency, "ARS");
        assert_eq!(rows[0].price_type, "PER_MINUTE");
        assert_eq!(rows[0].price_value_minor, Some(3_652_800));
    }

    #[test]
    fn yunojuno_preserves_hour_and_day_units() {
        let html = include_str!("fixtures/yunojuno.html");
        let rows = YunoJunoAdapter
            .extract(
                html,
                &source("yunojuno"),
                &MarketQueryContext::generic("video-editing".into(), vec!["INTERNATIONAL".into()]),
                "https://www.yunojuno.com/report",
            )
            .unwrap();
        assert!(rows
            .iter()
            .any(|item| item.price_type == "HOURLY" && item.currency == "USD"));
        assert!(rows.iter().any(|item| item.price_type == "DAILY"));
    }

    #[test]
    fn remote_jobs_salary_stays_context_only() {
        let html = include_str!("fixtures/remotejobs.html");
        let rows = RemoteJobsAdapter
            .extract(
                html,
                &source("remotejobs"),
                &MarketQueryContext::generic("programming".into(), vec!["LATAM".into()]),
                "https://remotejobs.lat/tools",
            )
            .unwrap();
        assert!(!rows.is_empty());
        assert!(rows.iter().all(|item| item.price_type == "MONTHLY_SALARY"
            && item.comparison_eligibility == "CONTEXT_ONLY"));
    }

    #[test]
    fn bcra_extracts_official_usd_ars_rate_and_date() {
        let body = include_str!("fixtures/bcra.json");
        let rows = BcraAdapter
            .extract(
                body,
                &source("bcra"),
                &MarketQueryContext::generic("currency".into(), vec!["AR".into()]),
                "https://api.bcra.gob.ar/estadisticascambiarias/v1.0/Cotizaciones",
            )
            .unwrap();
        assert_eq!(rows[0].subservice.as_deref(), Some("USD/ARS"));
        assert_eq!(rows[0].price_value_minor, Some(149_850));
        assert_eq!(rows[0].published_at.as_deref(), Some("2026-08-07"));
    }

    #[test]
    #[ignore = "verificación pública manual; los tests normales nunca dependen de internet"]
    fn live_public_adapters_return_traceable_data() {
        tauri::async_runtime::block_on(async {
            let client = crate::market::acquisition::http_client().unwrap();
            let cases = [
                (
                    "bcra",
                    "https://api.bcra.gob.ar/estadisticascambiarias/v1.0/Cotizaciones",
                    "currency",
                ),
                (
                    "remotejobs",
                    "https://remotejobs.lat/tools/calculadora-salario-remoto-latam",
                    "programming",
                ),
                (
                    "yunojuno",
                    "https://www.yunojuno.com/blogs/day-rates-update-film-motion",
                    "video-editing",
                ),
            ];
            for (adapter, url, service) in cases {
                let response = crate::market::acquisition::fetch_once(&client, url)
                    .await
                    .unwrap_or_else(|error| panic!("{adapter} acquisition: {error}"));
                let rows = extract_with_adapter(
                    &response.body,
                    &source(adapter),
                    &MarketQueryContext::generic(service.into(), vec!["GLOBAL".into()]),
                    &response.final_url,
                )
                .unwrap_or_else(|error| panic!("{adapter} parser: {error}"));
                assert!(!rows.is_empty(), "{adapter} did not yield observations");
            }
        });
    }

    #[test]
    #[ignore = "verificación pública manual del bloqueo; los tests normales no usan internet"]
    fn live_tarifario_is_stopped_when_site_is_suspended() {
        tauri::async_runtime::block_on(async {
            let client = crate::market::acquisition::http_client().unwrap();
            let response = crate::market::acquisition::fetch_once(
                &client,
                "https://tarifario.org/multimedia-c27",
            )
            .await
            .unwrap();
            assert!(crate::market::acquisition::blocked_reason(&response).is_some());
        });
    }
}
