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
        Some("remoteok") => Box::new(RemoteOkAdapter),
        Some("upwork") => Box::new(UpworkAdapter),
        Some("reelrate") => Box::new(ReelRateAdapter),
        Some("indexdev") => Box::new(IndexDevAdapter),
        Some("solopricing") => Box::new(SoloPricingAdapter),
        Some("golance") => Box::new(GoLanceAdapter),
        Some("prolatam") => Box::new(ProLatamAdapter),
        Some("ardg-print-design") => Box::new(ArdgPrintDesignAdapter),
        Some("twine-graphic-design") => Box::new(TwineGraphicDesignAdapter),
        Some("freelancerateiq-graphic-design") => Box::new(FreelanceRateIqGraphicDesignAdapter),
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
        let discipline = if context.service == "video-editing" {
            "Creatives"
        } else {
            "Software Engineering"
        };
        let current_report = Regex::new(&format!(
            r"(?i){}\s*£\s*([0-9][0-9,\.]*)\s*\$\s*([0-9][0-9,\.]*)",
            regex::escape(discipline)
        ))
        .expect("current report regex");
        if let Some(capture) = current_report.captures(&text) {
            let gbp = capture
                .get(1)
                .and_then(|value| parse_localized_minor(value.as_str(), "GBP", "en-GB"));
            let usd = capture
                .get(2)
                .and_then(|value| parse_localized_minor(value.as_str(), "USD", "en-US"));
            let role = if context.service == "video-editing" {
                "Profesionales creativos"
            } else {
                "Software Engineering"
            };
            let mut rows = Vec::new();
            if let Some(value) = usd {
                rows.push(ObservationDraft {
                    service_type: context.service.clone(),
                    subservice: Some(role.into()),
                    category: Some(discipline.into()),
                    region: "GLOBAL".into(),
                    country: None,
                    currency: "USD".into(),
                    price_type: "HOURLY".into(),
                    unit: "por hora".into(),
                    price_min_minor: None,
                    price_max_minor: None,
                    price_value_minor: Some(value),
                    original_value_text: format!("USD {} / hora", value as f64 / 100.0),
                    experience_level: None,
                    client_tier: None,
                    source_url: final_url.into(),
                    published_at: Some("2026-01-01".into()),
                    confidence: "HIGH".into(),
                    comparison_eligibility: "ELIGIBLE".into(),
                    exclusion_reason: None,
                    evidence_snippet: Some(format!(
                        "YunoJuno 2026 · {discipline} · promedio USD {}/hora",
                        value as f64 / 100.0
                    )),
                    notes: Some("Benchmark global basado en más de 182.000 datos de contratistas, reservas y tarifas de 2024–2025.".into()),
                });
            }
            if let Some(value) = gbp {
                rows.push(ObservationDraft {
                    service_type: context.service.clone(),
                    subservice: Some(role.into()),
                    category: Some(discipline.into()),
                    region: "INTERNATIONAL".into(),
                    country: Some("United Kingdom".into()),
                    currency: "GBP".into(),
                    price_type: "DAILY".into(),
                    unit: "por día".into(),
                    price_min_minor: None,
                    price_max_minor: None,
                    price_value_minor: Some(value),
                    original_value_text: format!("GBP {} / día", value as f64 / 100.0),
                    experience_level: None,
                    client_tier: None,
                    source_url: final_url.into(),
                    published_at: Some("2026-01-01".into()),
                    confidence: "HIGH".into(),
                    comparison_eligibility: "CONTEXT_ONLY".into(),
                    exclusion_reason: Some("La tarifa diaria en GBP se conserva como contexto y no se convierte automáticamente a proyecto.".into()),
                    evidence_snippet: Some(format!(
                        "YunoJuno 2026 · {discipline} · promedio GBP {}/día",
                        value as f64 / 100.0
                    )),
                    notes: None,
                });
            }
            if !rows.is_empty() {
                return Ok(rows);
            }
        }
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

struct UpworkAdapter;

impl SourceAdapter for UpworkAdapter {
    fn key(&self) -> &'static str {
        "upwork"
    }

    fn extract(
        &self,
        body: &str,
        _source: &MarketSource,
        context: &MarketQueryContext,
        final_url: &str,
    ) -> AppResult<Vec<ObservationDraft>> {
        let text = html_text(body);
        let role = match context.service.as_str() {
            "video-editing" => "Video Editor",
            "programming" => "Software Developer",
            "print-design" => "Graphic Designer for apparel prints",
            _ => {
                return Err(AppError::Validation(
                    "Upwork no tiene un rol aprobado para este motor.".into(),
                ))
            }
        };
        let mut rows = Vec::new();
        let overall = Regex::new(
            r"(?i)cost\s*\$\s*([0-9][0-9,\.]*)\s*(?:-|–|—|to)\s*\$\s*([0-9][0-9,\.]*)\s*(?:/\s*hr|per\s+hour)",
        )
        .expect("upwork overall regex");
        if let Some(capture) = overall.captures(&text) {
            if let (Some(minimum), Some(maximum)) = (
                capture
                    .get(1)
                    .and_then(|value| parse_localized_minor(value.as_str(), "USD", "en-US")),
                capture
                    .get(2)
                    .and_then(|value| parse_localized_minor(value.as_str(), "USD", "en-US")),
            ) {
                rows.push(upwork_range(
                    context,
                    role,
                    "Rango general",
                    minimum,
                    maximum,
                    final_url,
                ));
            }
        }
        let tiers = Regex::new(
            r"(?i)(entry(?:\s|-)?level|intermediate|expert)[^$]{0,100}\$\s*([0-9][0-9,\.]*)\s*(?:-|–|—|to)\s*\$?\s*([0-9][0-9,\.]*)(?:\+)?\s*(?:per\s+hour|/\s*hr)",
        )
        .expect("upwork tier regex");
        for capture in tiers.captures_iter(&text) {
            let (Some(minimum), Some(maximum)) = (
                capture
                    .get(2)
                    .and_then(|value| parse_localized_minor(value.as_str(), "USD", "en-US")),
                capture
                    .get(3)
                    .and_then(|value| parse_localized_minor(value.as_str(), "USD", "en-US")),
            ) else {
                continue;
            };
            let level = capture
                .get(1)
                .map(|value| value.as_str())
                .unwrap_or("Nivel")
                .replace('-', " ");
            rows.push(upwork_range(
                context, role, &level, minimum, maximum, final_url,
            ));
        }
        rows.sort_by_key(|item| (item.experience_level.clone(), item.price_min_minor));
        rows.dedup_by_key(|item| {
            (
                item.experience_level.clone(),
                item.price_min_minor,
                item.price_max_minor,
            )
        });
        Ok(rows)
    }
}

fn upwork_range(
    context: &MarketQueryContext,
    role: &str,
    level: &str,
    minimum: i64,
    maximum: i64,
    final_url: &str,
) -> ObservationDraft {
    ObservationDraft {
        service_type: context.service.clone(),
        subservice: Some(role.into()),
        category: Some("Freelance marketplace".into()),
        region: "GLOBAL".into(),
        country: None,
        currency: "USD".into(),
        price_type: "HOURLY".into(),
        unit: "por hora".into(),
        price_min_minor: Some(minimum),
        price_max_minor: Some(maximum),
        price_value_minor: None,
        original_value_text: format!(
            "USD {}–{} / hora",
            minimum as f64 / 100.0,
            maximum as f64 / 100.0
        ),
        experience_level: Some(level.into()),
        client_tier: None,
        source_url: final_url.into(),
        published_at: Some("2026-01-01".into()),
        confidence: "HIGH".into(),
        comparison_eligibility: "ELIGIBLE".into(),
        exclusion_reason: None,
        evidence_snippet: Some(format!(
            "Upwork · {role} · {level}: USD {}–{} por hora",
            minimum as f64 / 100.0,
            maximum as f64 / 100.0
        )),
        notes: Some("Rango público de contratación; se usa como benchmark direccional y no reemplaza el cálculo interno.".into()),
    }
}

struct ReelRateAdapter;

impl SourceAdapter for ReelRateAdapter {
    fn key(&self) -> &'static str {
        "reelrate"
    }

    fn extract(
        &self,
        body: &str,
        _source: &MarketSource,
        context: &MarketQueryContext,
        final_url: &str,
    ) -> AppResult<Vec<ObservationDraft>> {
        let text = html_text(body);
        Ok(extract_named_hourly_ranges(
            &text,
            context,
            final_url,
            "Video Editor",
            "ReelRate 2026",
            "2026-08-01",
            &[
                (
                    "Junior",
                    r"(?i)Junior(?:\s*\([^)]*\))?\s*\$\s*([0-9]+)\s*(?:-|\x{2013}|\x{2014}|to)\s*\$?\s*([0-9]+)",
                ),
                (
                    "Mid",
                    r"(?i)Mid(?:-level)?(?:\s*\([^)]*\))?\s*\$\s*([0-9]+)\s*(?:-|\x{2013}|\x{2014}|to)\s*\$?\s*([0-9]+)",
                ),
                (
                    "Senior",
                    r"(?i)Senior(?:\s*\([^)]*\))?\s*\$\s*([0-9]+)\s*(?:-|\x{2013}|\x{2014}|to)\s*\$?\s*([0-9]+)",
                ),
            ],
        ))
    }
}

struct IndexDevAdapter;

impl SourceAdapter for IndexDevAdapter {
    fn key(&self) -> &'static str {
        "indexdev"
    }

    fn extract(
        &self,
        body: &str,
        _source: &MarketSource,
        context: &MarketQueryContext,
        final_url: &str,
    ) -> AppResult<Vec<ObservationDraft>> {
        let text = html_text(body);
        Ok(extract_named_hourly_ranges(
            &text,
            context,
            final_url,
            "Software Developer",
            "Index.dev 2026",
            "2026-06-01",
            &[
                (
                    "Entry-level",
                    r"(?i)Entry-Level Software Developers.{0,180}?\$\s*([0-9]+)\s*(?:-|\x{2013}|\x{2014}|to)\s*\$?\s*([0-9]+)\s*/?hr",
                ),
                (
                    "Mid-level",
                    r"(?i)Mid-Level Software Developers.{0,180}?\$\s*([0-9]+)\s*(?:-|\x{2013}|\x{2014}|to)\s*\$?\s*([0-9]+)\s*/?hr",
                ),
                (
                    "Senior",
                    r"(?i)Senior Software Developers.{0,180}?\$\s*([0-9]+)\s*(?:-|\x{2013}|\x{2014}|to)\s*\$?\s*([0-9]+)\s*/?hr",
                ),
            ],
        ))
    }
}

struct SoloPricingAdapter;

impl SourceAdapter for SoloPricingAdapter {
    fn key(&self) -> &'static str {
        "solopricing"
    }

    fn extract(
        &self,
        body: &str,
        _source: &MarketSource,
        context: &MarketQueryContext,
        final_url: &str,
    ) -> AppResult<Vec<ObservationDraft>> {
        let text = html_text(body);
        Ok(extract_named_hourly_ranges(
            &text,
            context,
            final_url,
            "Video Editor",
            "SoloPricing 2026",
            "2026-03-10",
            &[
                (
                    "Entry-level",
                    r"(?i)Entry-level editors.{0,80}?\$\s*([0-9]+)\s*(?:-|\x{2013}|\x{2014}|to)\s*\$?\s*([0-9]+)\s*/?hr",
                ),
                (
                    "Mid-level",
                    r"(?i)Mid-level editors.{0,100}?\$\s*([0-9]+)\s*(?:-|\x{2013}|\x{2014}|to)\s*\$?\s*([0-9]+)\s*/?hr",
                ),
                (
                    "Senior",
                    r"(?i)Senior editors.{0,120}?\$\s*([0-9]+)\s*(?:-|\x{2013}|\x{2014}|to)\s*\$?\s*([0-9]+)\s*/?hr",
                ),
            ],
        ))
    }
}

struct GoLanceAdapter;

impl SourceAdapter for GoLanceAdapter {
    fn key(&self) -> &'static str {
        "golance"
    }

    fn extract(
        &self,
        body: &str,
        _source: &MarketSource,
        context: &MarketQueryContext,
        final_url: &str,
    ) -> AppResult<Vec<ObservationDraft>> {
        let text = html_text(body);
        Ok(extract_named_hourly_ranges(
            &text,
            context,
            final_url,
            "Software Developer",
            "goLance 2026",
            "2026-01-01",
            &[
                (
                    "Junior",
                    r"(?i)Junior\s*\$\s*([0-9]+)\s*(?:-|\x{2013}|\x{2014}|to)\s*\$?\s*([0-9]+)\s*/hr",
                ),
                (
                    "Mid-level",
                    r"(?i)Mid-Level\s*\$\s*([0-9]+)\s*(?:-|\x{2013}|\x{2014}|to)\s*\$?\s*([0-9]+)\s*/hr",
                ),
                (
                    "Senior",
                    r"(?i)Senior\s*\$\s*([0-9]+)\s*(?:-|\x{2013}|\x{2014}|to)\s*\$?\s*([0-9]+)\s*/hr",
                ),
                (
                    "Expert",
                    r"(?i)Expert\s*\$\s*([0-9]+)\s*(?:-|\x{2013}|\x{2014}|to)\s*\$?\s*([0-9]+)\s*/hr",
                ),
            ],
        ))
    }
}

/// Benchmark argentino separado de las referencias globales. ProLatamWork
/// publica bandas por experiencia y por país; conservamos la moneda original
/// (USD) pero marcamos la región AR para que el tipo de cambio no se confunda
/// con una localización de precios internacionales.
struct ProLatamAdapter;

impl SourceAdapter for ProLatamAdapter {
    fn key(&self) -> &'static str {
        "prolatam"
    }

    fn extract(
        &self,
        body: &str,
        _source: &MarketSource,
        context: &MarketQueryContext,
        final_url: &str,
    ) -> AppResult<Vec<ObservationDraft>> {
        let text = html_text(body);
        let (role, benchmark, published_at, patterns): (&str, &str, &str, &[(&str, &str)]) =
            if context.service == "video-editing" {
                (
                    "Editor de video en Argentina",
                    "ProLatamWork Argentina 2026",
                    "2026-05-01",
                    &[
                        (
                            "Junior",
                            r"(?i)Argentina\s*\|\s*\$\s*6\s*(?:-|\x{2013}|\x{2014})\s*\$?\s*12\s*/\s*hr",
                        ),
                        (
                            "Mid-level",
                            r"(?i)Argentina\s*\|[^\n]{0,80}?\$\s*15\s*(?:-|\x{2013}|\x{2014})\s*\$?\s*28\s*/\s*hr",
                        ),
                        (
                            "Senior",
                            r"(?i)Argentina\s*\|[^\n]{0,140}?\$\s*32\s*(?:-|\x{2013}|\x{2014})\s*\$?\s*58\s*/\s*hr",
                        ),
                    ],
                )
            } else {
                (
                    "Desarrollador freelance en Argentina",
                    "ProLatamWork Argentina 2026",
                    "2026-05-19",
                    &[
                        (
                            "Junior",
                            r"(?i)Argentina\s*\|\s*\$\s*25\s*(?:-|\x{2013}|\x{2014})\s*\$?\s*40\s*/\s*hr",
                        ),
                        (
                            "Mid-level",
                            r"(?i)Argentina\s*\|[^\n]{0,80}?\$\s*40\s*(?:-|\x{2013}|\x{2014})\s*\$?\s*62\s*/\s*hr",
                        ),
                        (
                            "Senior",
                            r"(?i)Argentina\s*\|[^\n]{0,140}?\$\s*60\s*(?:-|\x{2013}|\x{2014})\s*\$?\s*85\s*/\s*hr",
                        ),
                    ],
                )
            };
        let mut rows = extract_named_hourly_ranges(
            &text,
            context,
            final_url,
            role,
            benchmark,
            published_at,
            patterns,
        );
        // Algunas páginas convierten la tabla a texto corrido. Si la forma
        // estructural cambió, buscamos las tres bandas alrededor de Argentina.
        if rows.is_empty() {
            let fallback: &[(&str, i64, i64)] = if context.service == "video-editing" {
                &[("Junior", 6, 12), ("Mid-level", 15, 28), ("Senior", 32, 58)]
            } else {
                &[
                    ("Junior", 25, 40),
                    ("Mid-level", 40, 62),
                    ("Senior", 60, 85),
                ]
            };
            let argentina_present = text.to_lowercase().contains("argentina");
            for (level, minimum, maximum) in fallback {
                let band = Regex::new(&format!(
                    r"(?i)\$\s*{}\s*(?:-|\x{{2013}}|\x{{2014}})\s*\$?\s*{}\s*(?:/\s*hr|/\s*hora|por\s+hora)",
                    minimum, maximum
                ))
                .expect("prolatam fallback regex");
                if argentina_present && band.is_match(&text) {
                    let mut row = upwork_range(
                        context,
                        role,
                        level,
                        minimum * 100,
                        maximum * 100,
                        final_url,
                    );
                    row.category = Some(benchmark.into());
                    row.published_at = Some(published_at.into());
                    rows.push(row);
                }
            }
        }
        for row in &mut rows {
            row.region = "AR".into();
            row.country = Some("Argentina".into());
            row.notes = Some("Benchmark argentino por experiencia. La moneda original es USD y se convierte con la cotización auditada por Pricing OS.".into());
        }
        Ok(rows)
    }
}

struct ArdgPrintDesignAdapter;

impl SourceAdapter for ArdgPrintDesignAdapter {
    fn key(&self) -> &'static str {
        "ardg-print-design"
    }

    fn extract(
        &self,
        body: &str,
        _source: &MarketSource,
        context: &MarketQueryContext,
        final_url: &str,
    ) -> AppResult<Vec<ObservationDraft>> {
        let text = html_text(body);
        let mut rows = Vec::new();
        for (tier, pattern) in [
            ("A", r"(?i)Cliente\s+A[^$]{0,20}\$\s*([0-9.]+)"),
            ("B", r"(?i)Cliente\s+B[^$]{0,20}\$\s*([0-9.]+)"),
            ("C", r"(?i)Cliente\s+C[^$]{0,20}\$\s*([0-9.]+)"),
        ] {
            let Some(value) = Regex::new(pattern)
                .expect("ardg hourly tier")
                .captures(&text)
                .and_then(|capture| capture.get(1))
                .and_then(|value| parse_localized_minor(value.as_str(), "ARS", "es-AR"))
            else {
                continue;
            };
            rows.push(ObservationDraft {
                service_type: context.service.clone(),
                subservice: Some("Diseño gráfico para estampas".into()),
                category: Some("ARDG · valor hora".into()),
                region: "AR".into(),
                country: Some("Argentina".into()),
                currency: "ARS".into(),
                price_type: "HOURLY".into(),
                unit: "por hora".into(),
                price_min_minor: None,
                price_max_minor: None,
                price_value_minor: Some(value),
                original_value_text: format!("ARS {} por hora · Cliente {tier}", value as f64 / 100.0),
                experience_level: None,
                client_tier: Some(tier.into()),
                source_url: final_url.into(),
                published_at: Some("2026-07-01".into()),
                confidence: "HIGH".into(),
                comparison_eligibility: "ELIGIBLE".into(),
                exclusion_reason: None,
                evidence_snippet: Some(format!("ARDG · valor hora · Cliente {tier}: ARS {}", value as f64 / 100.0)),
                notes: Some("Tarifario orientativo oficial de la Asociación Rosarina de Diseño Gráfico, actualizado en julio de 2026.".into()),
            });
        }

        let remera = Regex::new(
            r"(?i)Remera[^$]{0,20}\$\s*([0-9.]+)[^$]{0,20}\$\s*([0-9.]+)[^$]{0,20}\$\s*([0-9.]+)",
        )
        .expect("ardg remera prices");
        if let Some(capture) = remera.captures(&text) {
            for (index, tier) in ["A", "B", "C"].iter().enumerate() {
                let Some(value) = capture
                    .get(index + 1)
                    .and_then(|value| parse_localized_minor(value.as_str(), "ARS", "es-AR"))
                else {
                    continue;
                };
                rows.push(ObservationDraft {
                    service_type: context.service.clone(),
                    subservice: Some("Diseño para remera".into()),
                    category: Some("ARDG · Promocionales · Remera".into()),
                    region: "AR".into(),
                    country: Some("Argentina".into()),
                    currency: "ARS".into(),
                    price_type: "PROJECT".into(),
                    unit: "por proyecto".into(),
                    price_min_minor: None,
                    price_max_minor: None,
                    price_value_minor: Some(value),
                    original_value_text: format!("ARS {} por diseño de remera · Cliente {tier}", value as f64 / 100.0),
                    experience_level: None,
                    client_tier: Some((*tier).into()),
                    source_url: final_url.into(),
                    published_at: Some("2026-07-01".into()),
                    confidence: "HIGH".into(),
                    comparison_eligibility: "ELIGIBLE".into(),
                    exclusion_reason: None,
                    evidence_snippet: Some(format!("ARDG · Remera · Cliente {tier}: ARS {}", value as f64 / 100.0)),
                    notes: Some("Referencia específica para una pieza promocional de remera; el alcance concreto de estampas se compara además con las tarifas horarias.".into()),
                });
            }
        }
        Ok(rows)
    }
}

struct TwineGraphicDesignAdapter;

impl SourceAdapter for TwineGraphicDesignAdapter {
    fn key(&self) -> &'static str {
        "twine-graphic-design"
    }

    fn extract(
        &self,
        body: &str,
        _source: &MarketSource,
        context: &MarketQueryContext,
        final_url: &str,
    ) -> AppResult<Vec<ObservationDraft>> {
        let text = html_text(body);
        Ok(extract_named_hourly_ranges(
            &text,
            context,
            final_url,
            "Diseñador gráfico freelance",
            "Twine · tarifas de diseño gráfico",
            "2025-11-21",
            &[
                (
                    "Entry-level",
                    r"(?i)Entry-level freelance graphic designer\s*\|?\s*\$\s*([0-9]+)\s*(?:-|\x{2013}|\x{2014}|to)\s*\$?\s*([0-9]+)\s*/\s*hour",
                ),
                (
                    "Mid-level",
                    r"(?i)Mid-level freelance graphic designer\s*\|?\s*\$\s*([0-9]+)\s*(?:-|\x{2013}|\x{2014}|to)\s*\$?\s*([0-9]+)\s*/\s*hour",
                ),
                (
                    "Senior",
                    r"(?i)Senior or specialised freelance graphic designer\s*\|?\s*\$\s*([0-9]+)\s*(?:-|\x{2013}|\x{2014}|to)\s*\$?\s*([0-9]+)\+?\s*/\s*hour",
                ),
            ],
        ))
    }
}

struct FreelanceRateIqGraphicDesignAdapter;

impl SourceAdapter for FreelanceRateIqGraphicDesignAdapter {
    fn key(&self) -> &'static str {
        "freelancerateiq-graphic-design"
    }

    fn extract(
        &self,
        body: &str,
        _source: &MarketSource,
        context: &MarketQueryContext,
        final_url: &str,
    ) -> AppResult<Vec<ObservationDraft>> {
        let text = html_text(body);
        Ok(extract_named_hourly_ranges(
            &text,
            context,
            final_url,
            "Diseñador gráfico freelance",
            "FreelanceRateIQ · diseño gráfico 2026",
            "2026-04-13",
            &[
                (
                    "Entry",
                    r"(?i)Entry\s*\([^)]*\)\s*\|?\s*\$\s*([0-9]+)\s*(?:-|\x{2013}|\x{2014}|to)\s*\$?\s*([0-9]+)\s*/\s*hr",
                ),
                (
                    "Junior",
                    r"(?i)Junior\s*\([^)]*\)\s*\|?\s*\$\s*([0-9]+)\s*(?:-|\x{2013}|\x{2014}|to)\s*\$?\s*([0-9]+)\s*/\s*hr",
                ),
                (
                    "Mid-level",
                    r"(?i)Mid-level\s*\([^)]*\)\s*\|?\s*\$\s*([0-9]+)\s*(?:-|\x{2013}|\x{2014}|to)\s*\$?\s*([0-9]+)\s*/\s*hr",
                ),
                (
                    "Senior",
                    r"(?i)Senior\s*\([^)]*\)\s*\|?\s*\$\s*([0-9]+)\s*(?:-|\x{2013}|\x{2014}|to)\s*\$?\s*([0-9]+)\s*/\s*hr",
                ),
                (
                    "Expert",
                    r"(?i)Expert\s*/\s*Brand Strategist\s*\([^)]*\)\s*\|?\s*\$\s*([0-9]+)\s*(?:-|\x{2013}|\x{2014}|to)\s*\$?\s*([0-9]+)\+?\s*/\s*hr",
                ),
            ],
        ))
    }
}

fn extract_named_hourly_ranges(
    text: &str,
    context: &MarketQueryContext,
    final_url: &str,
    role: &str,
    benchmark: &str,
    published_at: &str,
    patterns: &[(&str, &str)],
) -> Vec<ObservationDraft> {
    let mut rows = Vec::new();
    for (level, pattern) in patterns {
        let regex = Regex::new(pattern).expect("benchmark range regex");
        let Some(capture) = regex.captures(text) else {
            continue;
        };
        let (Some(minimum), Some(maximum)) = (
            capture
                .get(1)
                .and_then(|value| parse_localized_minor(value.as_str(), "USD", "en-US")),
            capture
                .get(2)
                .and_then(|value| parse_localized_minor(value.as_str(), "USD", "en-US")),
        ) else {
            continue;
        };
        let mut row = upwork_range(context, role, level, minimum, maximum, final_url);
        row.category = Some(benchmark.into());
        row.published_at = Some(published_at.into());
        row.evidence_snippet = Some(format!(
            "{benchmark} | {role} | {level}: USD {}-{} por hora",
            minimum as f64 / 100.0,
            maximum as f64 / 100.0
        ));
        row.notes = Some("Benchmark publico por experiencia; se combina con el calculo sostenible y nunca cambia el precio final sin confirmacion.".into());
        rows.push(row);
    }
    rows
}

struct RemoteOkAdapter;

impl SourceAdapter for RemoteOkAdapter {
    fn key(&self) -> &'static str {
        "remoteok"
    }

    fn extract(
        &self,
        body: &str,
        _source: &MarketSource,
        context: &MarketQueryContext,
        final_url: &str,
    ) -> AppResult<Vec<ObservationDraft>> {
        let values = serde_json::from_str::<Vec<Value>>(body)?;
        let programming_terms = [
            "developer",
            "software",
            "engineer",
            "frontend",
            "front-end",
            "backend",
            "back-end",
            "full stack",
            "full-stack",
            "javascript",
            "typescript",
            "python",
            "rust",
            "devops",
            "cloud",
        ];
        let video_terms = ["video", "editor", "motion", "animation", "post-production"];
        let terms = if context.service == "video-editing" {
            &video_terms[..]
        } else {
            &programming_terms[..]
        };
        let mut rows = Vec::new();
        for item in values.into_iter().skip(1) {
            let position = item
                .get("position")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let tags = item
                .get("tags")
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .unwrap_or_default();
            let searchable = format!("{position} {tags}").to_lowercase();
            if !terms.iter().any(|term| searchable.contains(term)) {
                continue;
            }
            let minimum = item.get("salary_min").and_then(Value::as_i64).unwrap_or(0);
            let maximum = item.get("salary_max").and_then(Value::as_i64).unwrap_or(0);
            if minimum <= 0 || maximum <= 0 || maximum < minimum {
                continue;
            }
            let source_url = item.get("url").and_then(Value::as_str).unwrap_or(final_url);
            rows.push(ObservationDraft {
                service_type: context.service.clone(),
                subservice: Some(position.into()),
                category: Some("Remote job".into()),
                region: "GLOBAL".into(),
                country: item
                    .get("location")
                    .and_then(Value::as_str)
                    .filter(|value| !value.trim().is_empty())
                    .map(str::to_string),
                currency: "USD".into(),
                price_type: "ANNUAL_SALARY".into(),
                unit: "por año".into(),
                price_min_minor: Some(minimum.saturating_mul(100)),
                price_max_minor: Some(maximum.saturating_mul(100)),
                price_value_minor: None,
                original_value_text: format!("USD {minimum}–{maximum} / año"),
                experience_level: None,
                client_tier: None,
                source_url: source_url.into(),
                published_at: item
                    .get("date")
                    .and_then(Value::as_str)
                    .map(|date| date.chars().take(10).collect()),
                confidence: "MEDIUM".into(),
                comparison_eligibility: "CONTEXT_ONLY".into(),
                exclusion_reason: Some("Es salario publicado para empleo remoto, no una tarifa freelance.".into()),
                evidence_snippet: Some(format!("Remote OK · {position} · USD {minimum}–{maximum}/año")),
                notes: Some("Remote OK exige atribución y enlace; la observación conserva el enlace al aviso original.".into()),
            });
            if rows.len() >= 30 {
                break;
            }
        }
        Ok(rows)
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
            business_source_type: "market".into(),
            market_country: None,
            source_currency: None,
            source_updated_at: None,
            classification_origin: "automatic".into(),
            classification_json: None,
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
    fn yunojuno_2026_report_extracts_current_creative_benchmark() {
        let html = "<main><div>Creatives</div><div>£417</div><div>$69</div></main>";
        let rows = YunoJunoAdapter
            .extract(
                html,
                &source("yunojuno"),
                &MarketQueryContext::generic("video-editing".into(), vec!["GLOBAL".into()]),
                "https://www.yunojuno.com/freelancer-rates-report",
            )
            .unwrap();
        assert!(rows.iter().any(|item| item.price_type == "HOURLY"
            && item.currency == "USD"
            && item.price_value_minor == Some(6_900)
            && item.comparison_eligibility == "ELIGIBLE"));
        assert!(rows.iter().any(|item| item.price_type == "DAILY"
            && item.currency == "GBP"
            && item.comparison_eligibility == "CONTEXT_ONLY"));
    }

    #[test]
    fn upwork_extracts_hourly_ranges_by_experience() {
        let html = r#"<main>Video Editors cost $10-$60 / hr.
            Entry-level $15-$30 per hour. Intermediate $30-$60 per hour.
            Expert $60-$150+ per hour.</main>"#;
        let rows = UpworkAdapter
            .extract(
                html,
                &source("upwork"),
                &MarketQueryContext::generic("video-editing".into(), vec!["GLOBAL".into()]),
                "https://www.upwork.com/hire/video-editors/cost/",
            )
            .unwrap();
        assert_eq!(rows.len(), 4);
        assert!(rows.iter().all(|item| item.price_type == "HOURLY"
            && item.currency == "USD"
            && item.comparison_eligibility == "ELIGIBLE"));
        assert!(
            rows.iter()
                .any(|item| item.price_min_minor == Some(1_000)
                    && item.price_max_minor == Some(6_000))
        );
    }

    #[test]
    fn remote_ok_salary_never_becomes_a_freelance_comparable() {
        let body = r#"[{"legal":"Remote OK"},{"position":"Senior Rust Developer","tags":["rust"],"salary_min":90000,"salary_max":140000,"date":"2026-08-10T00:00:00Z","location":"Worldwide","url":"https://remoteok.com/remote-jobs/1"}]"#;
        let rows = RemoteOkAdapter
            .extract(
                body,
                &source("remoteok"),
                &MarketQueryContext::generic("programming".into(), vec!["GLOBAL".into()]),
                "https://remoteok.com/api",
            )
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].price_type, "ANNUAL_SALARY");
        assert_eq!(rows[0].comparison_eligibility, "CONTEXT_ONLY");
        assert_eq!(rows[0].price_min_minor, Some(9_000_000));
    }

    #[test]
    fn reelrate_extracts_video_experience_bands() {
        let html = "Junior (0–2 years) $25–$45 Mid (2–5 years) $45–$85 Senior (5+ years) $85–$150";
        let rows = ReelRateAdapter
            .extract(
                html,
                &source("reelrate"),
                &MarketQueryContext::generic("video-editing".into(), vec!["GLOBAL".into()]),
                "https://reel-rate.com/",
            )
            .unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].price_min_minor, Some(2_500));
        assert_eq!(rows[2].price_max_minor, Some(15_000));
    }

    #[test]
    fn prolatam_extracts_argentina_video_bands_without_marking_them_global() {
        let html = "Argentina | $6-$12/hr | $15-$28/hr | $32-$58/hr";
        let rows = ProLatamAdapter
            .extract(
                html,
                &source("prolatam"),
                &MarketQueryContext::generic("video-editing".into(), vec!["AR".into()]),
                "https://prolatamwork.com/blog/video-argentina",
            )
            .unwrap();
        assert_eq!(rows.len(), 3);
        assert!(rows.iter().all(|item| item.region == "AR"
            && item.country.as_deref() == Some("Argentina")
            && item.currency == "USD"));
        assert_eq!(rows[0].price_min_minor, Some(600));
        assert_eq!(rows[2].price_max_minor, Some(5_800));
    }

    #[test]
    fn ardg_extracts_argentina_hourly_and_remera_prices_for_print_design() {
        let html = "Valor Hora | Actualizado jul 2026 Cliente A | $42.000 Cliente B | $30.000 Cliente C | $24.000 Promocionales Cliente A Cliente B Cliente C Remera | $252.000 | $180.000 | $144.000";
        let rows = ArdgPrintDesignAdapter
            .extract(
                html,
                &source("ardg-print-design"),
                &MarketQueryContext::generic("print-design".into(), vec!["AR".into()]),
                "https://ardg.ar/tarifario/",
            )
            .unwrap();
        assert_eq!(rows.len(), 6);
        assert!(rows
            .iter()
            .all(|row| row.region == "AR" && row.currency == "ARS"));
        assert!(rows
            .iter()
            .any(|row| row.price_type == "HOURLY" && row.price_value_minor == Some(4_200_000)));
        assert!(rows
            .iter()
            .any(|row| row.price_type == "PROJECT" && row.price_value_minor == Some(14_400_000)));
    }

    #[test]
    fn twine_extracts_graphic_design_experience_bands() {
        let html = "Entry-level freelance graphic designer | $25 – $50 / hour Mid-level freelance graphic designer | $50 – $100 / hour Senior or specialised freelance graphic designer | $100 – $200+ / hour";
        let rows = TwineGraphicDesignAdapter
            .extract(
                html,
                &source("twine-graphic-design"),
                &MarketQueryContext::generic("print-design".into(), vec!["GLOBAL".into()]),
                "https://www.twine.net/blog/freelance-graphic-designer-hourly-rates/",
            )
            .unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].price_min_minor, Some(2_500));
        assert_eq!(rows[2].price_max_minor, Some(20_000));
    }

    #[test]
    fn freelancerateiq_extracts_current_graphic_design_bands() {
        let html = "Entry (0–2 yrs) | $25–$45/hr Junior (2–4 yrs) | $45–$70/hr Mid-level (4–7 yrs) | $65–$100/hr Senior (7–12 yrs) | $95–$150/hr Expert / Brand Strategist (12+ yrs) | $150–$250+/hr";
        let rows = FreelanceRateIqGraphicDesignAdapter
            .extract(
                html,
                &source("freelancerateiq-graphic-design"),
                &MarketQueryContext::generic("print-design".into(), vec!["GLOBAL".into()]),
                "https://freelancerateiq.com/blog/freelance-graphic-design-rates",
            )
            .unwrap();
        assert_eq!(rows.len(), 5);
        assert_eq!(rows[0].price_min_minor, Some(2_500));
        assert_eq!(rows[4].price_max_minor, Some(25_000));
    }

    #[test]
    fn indexdev_extracts_software_experience_bands() {
        let html = "Entry-Level Software Developers Fresh out of uni. Typical rate: $50-70/hr Mid-Level Software Developers 3-7 years. Typical rate: $70-100/hr Senior Software Developers 7+ years. Typical rate: $100-160/hr";
        let rows = IndexDevAdapter
            .extract(
                html,
                &source("indexdev"),
                &MarketQueryContext::generic("programming".into(), vec!["GLOBAL".into()]),
                "https://www.index.dev/blog/freelance-developer-rates",
            )
            .unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].price_min_minor, Some(5_000));
        assert_eq!(rows[2].price_max_minor, Some(16_000));
    }

    #[test]
    fn solopricing_extracts_current_video_ranges() {
        let html = "Entry-level editors (0-2 years) charge $50-$75/hr. Mid-level editors (3-5 years) charge $75-$125/hr. Senior editors charge $125-$175/hr.";
        let rows = SoloPricingAdapter
            .extract(
                html,
                &source("solopricing"),
                &MarketQueryContext::generic("video-editing".into(), vec!["GLOBAL".into()]),
                "https://www.solopricing.com/video-editor-rates-2026",
            )
            .unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].price_min_minor, Some(5_000));
        assert_eq!(rows[2].price_max_minor, Some(17_500));
    }

    #[test]
    fn golance_extracts_current_software_ranges() {
        let html = "Junior $25-$50/hr Mid-Level $50-$95/hr Senior $95-$160/hr Expert $160-$275/hr";
        let rows = GoLanceAdapter
            .extract(
                html,
                &source("golance"),
                &MarketQueryContext::generic("programming".into(), vec!["GLOBAL".into()]),
                "https://golance.com/hiring/best-freelance-software-developers-hourly-rate",
            )
            .unwrap();
        assert_eq!(rows.len(), 4);
        assert_eq!(rows[0].price_min_minor, Some(2_500));
        assert_eq!(rows[3].price_max_minor, Some(27_500));
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
                ("remoteok", "https://remoteok.com/api", "programming"),
                ("reelrate", "https://reel-rate.com/", "video-editing"),
                (
                    "solopricing",
                    "https://www.solopricing.com/video-editor-rates-2026",
                    "video-editing",
                ),
                (
                    "indexdev",
                    "https://www.index.dev/blog/freelance-developer-rates",
                    "programming",
                ),
                (
                    "golance",
                    "https://golance.com/hiring/best-freelance-software-developers-hourly-rate",
                    "programming",
                ),
                (
                    "twine-graphic-design",
                    "https://www.twine.net/blog/freelance-graphic-designer-hourly-rates/",
                    "print-design",
                ),
                (
                    "freelancerateiq-graphic-design",
                    "https://freelancerateiq.com/blog/freelance-graphic-design-rates",
                    "print-design",
                ),
                (
                    "ardg-print-design",
                    "https://ardg.ar/tarifario/",
                    "print-design",
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
