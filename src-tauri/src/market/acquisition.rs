use std::{net::IpAddr, time::Duration};

use reqwest::{redirect::Policy, Client};
use tokio::net::lookup_host;
use url::{Host, Url};

use crate::error::{AppError, AppResult};

use super::{types::AcquisitionResponse, validation::validate_public_https};

pub fn http_client() -> AppResult<Client> {
    Client::builder()
        // Las redirecciones se procesan manualmente para validar esquema, host y DNS
        // antes de conectar con cada nuevo destino.
        .redirect(Policy::none())
        .connect_timeout(Duration::from_secs(7))
        .timeout(Duration::from_secs(15))
        .user_agent("PricingOS/0.1 (+local market research; user initiated)")
        .build()
        .map_err(|error| AppError::Validation(format!("No se pudo preparar HTTP: {error}")))
}

fn is_private_or_local(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let octets = ip.octets();
            ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_unspecified()
                || ip.is_multicast()
                || ip.is_broadcast()
                || octets[0] == 0
                || (octets[0] == 100 && (64..=127).contains(&octets[1]))
        }
        IpAddr::V6(ip) => {
            ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_unique_local()
                || ip.is_unicast_link_local()
                || ip.is_multicast()
        }
    }
}

async fn validate_resolved_destination(url: &Url) -> AppResult<()> {
    match url.host() {
        Some(Host::Ipv4(ip)) if is_private_or_local(IpAddr::V4(ip)) => Err(AppError::Validation(
            "La fuente resolvió a una red privada o local.".into(),
        )),
        Some(Host::Ipv6(ip)) if is_private_or_local(IpAddr::V6(ip)) => Err(AppError::Validation(
            "La fuente resolvió a una red privada o local.".into(),
        )),
        Some(Host::Domain(domain)) => {
            let port = url.port_or_known_default().unwrap_or(443);
            let addresses = lookup_host((domain, port)).await.map_err(|error| {
                AppError::Validation(format!("No se pudo resolver el dominio: {error}"))
            })?;
            let addresses = addresses.collect::<Vec<_>>();
            if addresses.is_empty() || addresses.iter().any(|item| is_private_or_local(item.ip())) {
                return Err(AppError::Validation(
                    "El dominio no tiene un destino público seguro.".into(),
                ));
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

pub async fn fetch_once(client: &Client, raw_url: &str) -> AppResult<AcquisitionResponse> {
    let mut url = validate_public_https(raw_url)?;
    let mut redirect_count = 0;
    let mut response = loop {
        validate_resolved_destination(&url).await?;
        let response = client
            .get(url.clone())
            .header("Accept", "text/html, application/json;q=0.9, */*;q=0.5")
            .send()
            .await
            .map_err(|error| {
                AppError::Validation(format!("No se pudo consultar la fuente: {error}"))
            })?;
        if response.status().is_redirection() {
            if redirect_count >= 3 {
                return Err(AppError::Validation(
                    "La fuente excedió el límite de tres redirecciones.".into(),
                ));
            }
            let location = response
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|value| value.to_str().ok())
                .ok_or_else(|| {
                    AppError::Validation("La fuente devolvió una redirección inválida.".into())
                })?;
            url = validate_public_https(
                url.join(location)
                    .map_err(|_| AppError::Validation("Redirección inválida.".into()))?
                    .as_str(),
            )?;
            redirect_count += 1;
            continue;
        }
        break response;
    };
    let status = response.status();
    let retry_after_seconds = response
        .headers()
        .get("retry-after")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<i64>().ok());
    let final_url = response.url().to_string();
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| AppError::Validation(format!("No se pudo leer la respuesta: {error}")))?
    {
        if bytes.len() + chunk.len() > 1_000_000 {
            return Err(AppError::Validation(
                "La fuente superó el límite seguro de 1 MB por consulta.".into(),
            ));
        }
        bytes.extend_from_slice(&chunk);
    }
    let body = String::from_utf8_lossy(&bytes).to_string();
    Ok(AcquisitionResponse {
        body,
        http_status: status.as_u16(),
        final_url,
        retry_after_seconds,
    })
}

pub fn blocked_reason(response: &AcquisitionResponse) -> Option<String> {
    if matches!(response.http_status, 401 | 403) {
        return Some(format!(
            "Acceso restringido (HTTP {}). Se detuvo la fuente.",
            response.http_status
        ));
    }
    if response.http_status == 429 {
        return Some(
            "La fuente aplicó rate limit (HTTP 429). No se reintentará automáticamente.".into(),
        );
    }
    let lower = response.body.to_lowercase();
    for (marker, reason) in [
        ("captcha", "La fuente solicitó CAPTCHA."),
        (
            "cloudflare challenge",
            "La fuente presentó un challenge de Cloudflare.",
        ),
        (
            "account suspended",
            "La cuenta o sitio de la fuente está suspendido.",
        ),
        ("login required", "La fuente requiere iniciar sesión."),
        ("paywall", "La fuente está detrás de un paywall."),
    ] {
        if lower.contains(marker) {
            return Some(reason.into());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_anti_automation_and_access_blocks_without_retrying() {
        for (status, body) in [
            (403, ""),
            (429, ""),
            (200, "Please solve CAPTCHA"),
            (200, "Account Suspended"),
        ] {
            let response = AcquisitionResponse {
                body: body.into(),
                http_status: status,
                final_url: "https://example.com".into(),
                retry_after_seconds: None,
            };
            assert!(blocked_reason(&response).is_some());
        }
    }
}
