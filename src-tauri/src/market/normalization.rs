use regex::Regex;
use sha2::{Digest, Sha256};

use super::types::ObservationDraft;

pub fn parse_localized_minor(raw: &str, currency: &str, locale: &str) -> Option<i64> {
    let cleaned = raw
        .replace('$', "")
        .replace("ARS", "")
        .replace("USD", "")
        .replace(['£', '€'], "")
        .trim()
        .to_string();
    if cleaned.is_empty() {
        return None;
    }
    let normalized = if locale.eq_ignore_ascii_case("es-AR") || currency == "ARS" {
        if cleaned.contains('.') && cleaned.contains(',') {
            cleaned.replace('.', "").replace(',', ".")
        } else if cleaned.contains('.') {
            let tail = cleaned.rsplit('.').next().unwrap_or_default();
            if tail.len() == 3 {
                cleaned.replace('.', "")
            } else {
                cleaned
            }
        } else {
            cleaned.replace(',', ".")
        }
    } else if cleaned.contains(',') && cleaned.contains('.') {
        cleaned.replace(',', "")
    } else if cleaned.contains(',') {
        let tail = cleaned.rsplit(',').next().unwrap_or_default();
        if tail.len() == 3 {
            cleaned.replace(',', "")
        } else {
            cleaned.replace(',', ".")
        }
    } else {
        cleaned
    };
    normalized.parse::<f64>().ok().and_then(|value| {
        (value.is_finite() && value >= 0.0).then_some((value * 100.0).round() as i64)
    })
}

pub fn detect_price_type(text: &str) -> (&'static str, &'static str) {
    let lower = text.to_lowercase();
    if lower.contains("por minuto") || lower.contains("/min") {
        ("PER_MINUTE", "por minuto")
    } else if lower.contains("por hora") || lower.contains("/hour") || lower.contains("hourly") {
        ("HOURLY", "por hora")
    } else if lower.contains("por día") || lower.contains("day rate") || lower.contains("/day") {
        ("DAILY", "por día")
    } else if lower.contains("por mes") || lower.contains("/mes") || lower.contains("monthly") {
        ("MONTHLY_SALARY", "por mes")
    } else if lower.contains("anual") || lower.contains("annual") || lower.contains("/year") {
        ("ANNUAL_SALARY", "por año")
    } else if lower.contains("por unidad") || lower.contains("per item") {
        ("PER_ITEM", "por unidad")
    } else if lower.contains("proyecto") || lower.contains("project") {
        ("PROJECT", "por proyecto")
    } else {
        ("UNKNOWN", "sin unidad")
    }
}

pub fn extract_numeric_tokens(text: &str) -> Vec<String> {
    Regex::new(r"(?x)(?:USD|ARS|US\$|\$|£|€)\s*[0-9]+(?:[\.,][0-9]{1,3})*(?:\s*[kK])?")
        .expect("price regex")
        .find_iter(text)
        .map(|item| item.as_str().trim().to_string())
        .collect()
}

pub fn parse_range_minor(raw: &str, currency: &str, locale: &str) -> Option<(i64, i64)> {
    let values = extract_numeric_tokens(raw)
        .iter()
        .filter_map(|value| parse_localized_minor(value, currency, locale))
        .collect::<Vec<_>>();
    match values.as_slice() {
        [value] => Some((*value, *value)),
        [first, second, ..] => Some(((*first).min(*second), (*first).max(*second))),
        _ => None,
    }
}

pub fn fingerprint(source_id: &str, draft: &ObservationDraft) -> String {
    let canonical = format!(
        "{}|{}|{}|{}|{}|{}|{:?}|{:?}|{:?}|{}",
        source_id,
        draft.source_url,
        draft.service_type,
        draft.subservice.as_deref().unwrap_or(""),
        draft.currency,
        draft.unit,
        draft.price_min_minor,
        draft.price_max_minor,
        draft.price_value_minor,
        draft.published_at.as_deref().unwrap_or("")
    );
    format!("{:x}", Sha256::digest(canonical.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_argentine_and_us_numbers_without_confusing_separators() {
        assert_eq!(
            parse_localized_minor("$36.528", "ARS", "es-AR"),
            Some(3_652_800)
        );
        assert_eq!(
            parse_localized_minor("USD 95.50", "USD", "en-US"),
            Some(9_550)
        );
        assert_eq!(
            parse_localized_minor("$3,500", "USD", "en-US"),
            Some(350_000)
        );
        assert_eq!(parse_localized_minor("dato roto", "USD", "en-US"), None);
    }

    #[test]
    fn detects_supported_units() {
        assert_eq!(detect_price_type("USD 95/hour").0, "HOURLY");
        assert_eq!(detect_price_type("USD 650 day rate").0, "DAILY");
        assert_eq!(detect_price_type("ARS por minuto").0, "PER_MINUTE");
        assert_eq!(detect_price_type("USD/mes").0, "MONTHLY_SALARY");
        assert_eq!(detect_price_type("annual salary").0, "ANNUAL_SALARY");
        assert!(extract_numeric_tokens("95 por hora, sin moneda").is_empty());
    }

    #[test]
    fn parses_ranges_without_losing_original_unit_context() {
        assert_eq!(
            parse_range_minor("USD $3,500 - $8,500 USD/mes", "USD", "en-US"),
            Some((350_000, 850_000))
        );
    }
}
