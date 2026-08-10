use crate::error::{AppError, AppResult};
use url::{Host, Url};

use super::types::ObservationDraft;

pub fn validate_public_https(raw: &str) -> AppResult<Url> {
    let url = Url::parse(raw).map_err(|_| AppError::Validation("La URL no es válida.".into()))?;
    if url.scheme() != "https" {
        return Err(AppError::Validation(
            "Sólo se permiten fuentes HTTPS.".into(),
        ));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(AppError::Validation(
            "La URL no puede contener credenciales.".into(),
        ));
    }
    match url.host() {
        Some(Host::Domain(domain)) => {
            let lower = domain.to_ascii_lowercase();
            if lower == "localhost" || lower.ends_with(".localhost") || !lower.contains('.') {
                return Err(AppError::Validation(
                    "No se permiten destinos locales.".into(),
                ));
            }
        }
        Some(Host::Ipv4(ip))
            if ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_unspecified()
                || ip.is_multicast()
                || ip.is_broadcast()
                || ip.octets()[0] == 0
                || (ip.octets()[0] == 100 && (64..=127).contains(&ip.octets()[1])) =>
        {
            return Err(AppError::Validation(
                "No se permiten redes privadas o locales.".into(),
            ));
        }
        Some(Host::Ipv6(ip))
            if ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_unique_local()
                || ip.is_unicast_link_local()
                || ip.is_multicast() =>
        {
            return Err(AppError::Validation(
                "No se permiten redes privadas o locales.".into(),
            ));
        }
        None => {
            return Err(AppError::Validation(
                "La fuente necesita un dominio válido.".into(),
            ))
        }
        _ => {}
    }
    Ok(url)
}

pub fn validate_observation(draft: &mut ObservationDraft) -> AppResult<()> {
    let prices = [
        draft.price_min_minor,
        draft.price_max_minor,
        draft.price_value_minor,
    ];
    if prices.iter().all(Option::is_none) {
        return Err(AppError::Validation(
            "La observación no contiene precio.".into(),
        ));
    }
    if prices.iter().flatten().any(|value| *value < 0) {
        return Err(AppError::Validation(
            "El precio no puede ser negativo.".into(),
        ));
    }
    if draft
        .price_min_minor
        .zip(draft.price_max_minor)
        .is_some_and(|(min, max)| min > max)
    {
        return Err(AppError::Validation(
            "El rango de precios está invertido.".into(),
        ));
    }
    let allowed_currency = ["ARS", "USD", "GBP", "EUR"];
    if !allowed_currency.contains(&draft.currency.as_str()) {
        return Err(AppError::Validation(
            "La moneda detectada no es compatible.".into(),
        ));
    }
    let allowed_types = [
        "HOURLY",
        "DAILY",
        "PROJECT",
        "PER_MINUTE",
        "PER_ITEM",
        "MONTHLY_SALARY",
        "ANNUAL_SALARY",
        "FIXED",
        "RANGE",
        "UNKNOWN",
    ];
    if !allowed_types.contains(&draft.price_type.as_str()) {
        return Err(AppError::Validation(
            "El tipo de precio no es válido.".into(),
        ));
    }
    if draft.price_type == "UNKNOWN" {
        draft.confidence = "REVIEW_REQUIRED".into();
        draft.comparison_eligibility = "REVIEW_REQUIRED".into();
        draft.exclusion_reason = Some("Unidad o tipo de precio sin identificar.".into());
    }
    if matches!(
        draft.price_type.as_str(),
        "MONTHLY_SALARY" | "ANNUAL_SALARY"
    ) {
        draft.comparison_eligibility = "CONTEXT_ONLY".into();
        draft.exclusion_reason = Some(
            "Los salarios se muestran como contexto y no se mezclan con tarifas freelance.".into(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_local_and_unsafe_urls() {
        assert!(validate_public_https("https://example.com/rates").is_ok());
        assert!(validate_public_https("http://example.com").is_err());
        assert!(validate_public_https("https://127.0.0.1/data").is_err());
        assert!(validate_public_https("https://100.64.0.1/data").is_err());
        assert!(validate_public_https("https://[fc00::1]/data").is_err());
        assert!(validate_public_https("file:///tmp/rates").is_err());
    }
}
