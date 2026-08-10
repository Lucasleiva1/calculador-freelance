use std::{collections::HashSet, process::Command, time::Instant};

use chrono::{DateTime, Duration, SecondsFormat, Utc};
use reqwest::Client;
use serde_json::{json, Value};
use sqlx::{Row, SqlitePool};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    db::AppState,
    error::{AppError, AppResult},
    models::{
        ManualObservationInput, MarketObservation, MarketObservationFilter,
        MarketObservationPreview, MarketOverview, MarketResearchJob, MarketResearchJobItem,
        MarketSnapshot, MarketSource, SourceTestResult,
    },
};

use super::{
    acquisition::{blocked_reason, fetch_once},
    adapters::extract_with_adapter,
    comparison::{compare_market, suggested_with_market},
    normalization::fingerprint,
    types::{MarketQueryContext, ObservationDraft},
    validation::{validate_observation, validate_public_https},
};

const SOURCE_COLUMNS: &str =
    "id, name, base_url, source_type, regions_json, supported_services_json,
 priority, enabled, usage_mode, acquisition_mode, cooldown_hours, notes, is_system_source,
 system_key, default_data_json, purpose, data_contribution, app_benefit,
 participates_in_suggestions, automation_status, current_status, adapter_key,
 last_request_at, last_success_at, last_failure_at, cooldown_until, consecutive_failures,
 last_http_status, last_error, observation_count, archived_at, business_source_type,
 market_country, source_currency, source_updated_at, classification_origin,
 classification_json, created_at, updated_at";

const OBSERVATION_COLUMNS: &str =
    "o.id, o.source_id, s.name AS source_name, o.origin, o.service_type,
 o.subservice, o.category, o.region, o.country, o.currency, o.price_type, o.unit,
 o.price_min_minor, o.price_max_minor, o.price_value_minor, o.original_value_text,
 o.converted_value_minor, o.converted_currency, o.exchange_rate_micros,
 o.exchange_rate_date, o.exchange_rate_source, o.experience_level, o.client_tier,
 o.source_type, o.source_url, o.published_at, o.retrieved_at, o.parser_version,
 o.confidence, o.comparison_eligibility, o.exclusion_reason, o.raw_fingerprint,
 o.evidence_snippet, o.notes, o.created_at, NULL AS snapshot_included,
 NULL AS snapshot_exclusion_reason, NULL AS snapshot_normalized_value_minor";

fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

async fn source_by_id(pool: &SqlitePool, id: &str) -> AppResult<MarketSource> {
    sqlx::query_as::<_, MarketSource>(&format!(
        "SELECT {SOURCE_COLUMNS} FROM market_sources WHERE id=? AND archived_at IS NULL"
    ))
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

async fn enabled_sources(pool: &SqlitePool, service_type: &str) -> AppResult<Vec<MarketSource>> {
    let sources = sqlx::query_as::<_, MarketSource>(&format!("SELECT {SOURCE_COLUMNS} FROM market_sources WHERE enabled=1 AND archived_at IS NULL ORDER BY priority, name COLLATE NOCASE"))
        .fetch_all(pool).await?;
    let assigned: Vec<String> = sqlx::query_scalar(
        "SELECT pes.source_id FROM pricing_engine_sources pes
         JOIN pricing_engines pe ON pe.id=pes.engine_id
         WHERE pe.engine_key=? AND pes.preference<>'excluded'",
    )
    .bind(service_type)
    .fetch_all(pool)
    .await?;
    if !assigned.is_empty() {
        let assigned = assigned.into_iter().collect::<HashSet<_>>();
        return Ok(sources
            .into_iter()
            .filter(|source| assigned.contains(&source.id) || source.usage_mode == "currency")
            .collect());
    }
    Ok(sources
        .into_iter()
        .filter(|source| {
            serde_json::from_str::<Vec<String>>(&source.supported_services_json)
                .unwrap_or_default()
                .iter()
                .any(|service| service == service_type || service == "all")
                || source.usage_mode == "currency"
        })
        .collect())
}

async fn participating_sources(
    pool: &SqlitePool,
    service_type: &str,
    sources: &[MarketSource],
) -> AppResult<HashSet<String>> {
    let assigned: Vec<String> = sqlx::query_scalar(
        "SELECT pes.source_id FROM pricing_engine_sources pes
         JOIN pricing_engines pe ON pe.id=pes.engine_id
         WHERE pe.engine_key=? AND pes.preference<>'excluded'
           AND pes.participates_in_suggestions=1",
    )
    .bind(service_type)
    .fetch_all(pool)
    .await?;
    if !assigned.is_empty() {
        return Ok(assigned.into_iter().collect());
    }
    Ok(sources
        .iter()
        .filter(|source| source.participates_in_suggestions)
        .map(|source| source.id.clone())
        .collect())
}

fn regions(source: &MarketSource) -> Vec<String> {
    serde_json::from_str(&source.regions_json).unwrap_or_else(|_| vec!["GLOBAL".into()])
}

fn preview(
    source_id: &str,
    status: &str,
    message: String,
    http_status: Option<i64>,
    drafts: Vec<ObservationDraft>,
) -> SourceTestResult {
    SourceTestResult {
        source_id: source_id.into(),
        status: status.into(),
        message,
        http_status,
        observations: drafts
            .into_iter()
            .map(|item| MarketObservationPreview {
                service_type: item.service_type,
                subservice: item.subservice,
                price_min_minor: item.price_min_minor,
                price_max_minor: item.price_max_minor,
                price_value_minor: item.price_value_minor,
                currency: item.currency,
                unit: item.unit,
                price_type: item.price_type,
                region: item.region,
                evidence: item.evidence_snippet,
            })
            .collect(),
    }
}

struct FetchLogOutcome<'a> {
    status: &'a str,
    http_status: Option<i64>,
    duration_ms: i64,
    cache_hit: bool,
    observation_count: i64,
    error_type: Option<&'a str>,
    error_message: Option<&'a str>,
}

async fn record_log(
    pool: &SqlitePool,
    source: &MarketSource,
    url: &str,
    started_at: &str,
    outcome: FetchLogOutcome<'_>,
) -> AppResult<()> {
    sqlx::query("INSERT INTO market_fetch_logs (id, source_id, url, method, started_at, finished_at, status, http_status, duration_ms, cache_hit, observation_count, error_type, error_message) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(Uuid::new_v4().to_string()).bind(&source.id).bind(url).bind(&source.acquisition_mode)
        .bind(started_at).bind(now()).bind(outcome.status).bind(outcome.http_status).bind(outcome.duration_ms)
        .bind(outcome.cache_hit).bind(outcome.observation_count).bind(outcome.error_type).bind(outcome.error_message)
        .execute(pool).await?;
    Ok(())
}

async fn persist_draft(
    pool: &SqlitePool,
    source: &MarketSource,
    mut draft: ObservationDraft,
    origin: &str,
) -> AppResult<(String, bool)> {
    validate_observation(&mut draft)?;
    if origin != "MANUAL" || !draft.source_url.starts_with("pricing-os://manual/") {
        validate_public_https(&draft.source_url)?;
    }
    let hash = fingerprint(&source.id, &draft);
    let id = Uuid::new_v4().to_string();
    let timestamp = now();
    let inserted = sqlx::query("INSERT OR IGNORE INTO market_observations (
      id, source_id, origin, service_type, subservice, category, region, country, currency,
      price_type, unit, price_min_minor, price_max_minor, price_value_minor, original_value_text,
      experience_level, client_tier, source_type, source_url, published_at, retrieved_at,
      parser_version, confidence, comparison_eligibility, exclusion_reason, raw_fingerprint,
      evidence_snippet, notes, created_at
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'market-v1', ?, ?, ?, ?, ?, ?, ?)")
        .bind(&id).bind(&source.id).bind(origin).bind(&draft.service_type).bind(&draft.subservice)
        .bind(&draft.category).bind(&draft.region).bind(&draft.country).bind(&draft.currency)
        .bind(&draft.price_type).bind(&draft.unit).bind(draft.price_min_minor).bind(draft.price_max_minor)
        .bind(draft.price_value_minor).bind(&draft.original_value_text).bind(&draft.experience_level)
        .bind(&draft.client_tier).bind(&source.source_type).bind(&draft.source_url).bind(&draft.published_at)
        .bind(&timestamp).bind(&draft.confidence).bind(&draft.comparison_eligibility)
        .bind(&draft.exclusion_reason).bind(&hash).bind(&draft.evidence_snippet).bind(&draft.notes)
        .bind(&timestamp).execute(pool).await?.rows_affected() > 0;
    let persisted_id = if inserted {
        id
    } else {
        sqlx::query_scalar::<_, String>(
            "SELECT id FROM market_observations WHERE raw_fingerprint=?",
        )
        .bind(&hash)
        .fetch_one(pool)
        .await?
    };
    if source.adapter_key.as_deref() == Some("bcra")
        && draft.subservice.as_deref() == Some("USD/ARS")
    {
        if let (Some(rate_minor), Some(rate_date)) =
            (draft.price_value_minor, draft.published_at.as_deref())
        {
            sqlx::query("INSERT OR IGNORE INTO market_fx_rates (id, source_id, base_currency, quote_currency, rate_micros, rate_date, source_url, retrieved_at) VALUES (?, ?, 'USD', 'ARS', ?, ?, ?, ?)")
                .bind(Uuid::new_v4().to_string()).bind(&source.id).bind(rate_minor.saturating_mul(100))
                .bind(rate_date).bind(&draft.source_url).bind(&timestamp).execute(pool).await?;
        }
    }
    sqlx::query("UPDATE market_sources SET observation_count=(SELECT COUNT(*) FROM market_observations WHERE source_id=?), updated_at=? WHERE id=?")
        .bind(&source.id).bind(&timestamp).bind(&source.id).execute(pool).await?;
    Ok((persisted_id, inserted))
}

fn cooldown_active(raw: Option<&str>) -> bool {
    raw.and_then(|raw| DateTime::parse_from_rfc3339(raw).ok())
        .is_some_and(|value| value.with_timezone(&Utc) > Utc::now())
}

fn should_use_cache(raw: Option<&str>, force: bool) -> bool {
    !force && cooldown_active(raw)
}

fn observation_midpoint(observation: &MarketObservation) -> Option<i64> {
    observation
        .price_value_minor
        .or_else(|| {
            observation
                .price_min_minor
                .zip(observation.price_max_minor)
                .map(|(minimum, maximum)| minimum + (maximum - minimum) / 2)
        })
        .or(observation.price_min_minor)
        .or(observation.price_max_minor)
}

fn converted_midpoint(
    observation: &MarketObservation,
    currency: &str,
    rate_micros: Option<i64>,
) -> Option<i64> {
    let value = observation_midpoint(observation)?;
    if observation.currency == currency {
        return Some(value);
    }
    let rate = rate_micros? as f64 / 10_000.0;
    if rate <= 0.0 {
        return None;
    }
    match (observation.currency.as_str(), currency) {
        ("USD", "ARS") => Some((value as f64 * rate).round() as i64),
        ("ARS", "USD") => Some((value as f64 / rate).round() as i64),
        _ => None,
    }
}

async fn process_source(
    pool: &SqlitePool,
    client: &Client,
    request_lock: &Mutex<()>,
    source: &MarketSource,
    context: &MarketQueryContext,
    force: bool,
    persist: bool,
) -> AppResult<SourceTestResult> {
    let url = source
        .base_url
        .as_deref()
        .ok_or_else(|| AppError::Validation("La fuente no tiene URL base.".into()))?;
    validate_public_https(url)?;
    if source.acquisition_mode == "manual" {
        return Ok(preview(
            &source.id,
            "MANUAL",
            "Fuente manual: abrila y agregá una observación trazable.".into(),
            None,
            vec![],
        ));
    }
    if source.acquisition_mode == "disabled" || !source.enabled {
        return Ok(preview(
            &source.id,
            "DISABLED",
            "La fuente está desactivada.".into(),
            None,
            vec![],
        ));
    }
    if source.acquisition_mode == "auto_browser" {
        return Ok(preview(
            &source.id,
            "NEEDS_CONFIGURATION",
            "AUTO_BROWSER está aislado: no existe un sidecar Playwright aprobado para esta fuente."
                .into(),
            None,
            vec![],
        ));
    }
    if source.automation_status != "APPROVED" {
        return Ok(preview(
            &source.id,
            "NEEDS_CONFIGURATION",
            "La automatización aún no fue aprobada mediante Probar fuente.".into(),
            None,
            vec![],
        ));
    }
    // Una cola global conservadora impide solicitudes simultáneas y, por lo tanto,
    // satisface también el límite de una solicitud por dominio.
    let _request_guard = request_lock.lock().await;
    let current_cooldown: Option<String> =
        sqlx::query_scalar("SELECT cooldown_until FROM market_sources WHERE id=?")
            .bind(&source.id)
            .fetch_one(pool)
            .await?;
    if should_use_cache(current_cooldown.as_deref(), force) {
        let started = now();
        record_log(
            pool,
            source,
            url,
            &started,
            FetchLogOutcome {
                status: "CACHED",
                http_status: source.last_http_status,
                duration_ms: 0,
                cache_hit: true,
                observation_count: 0,
                error_type: None,
                error_message: None,
            },
        )
        .await?;
        sqlx::query("UPDATE market_sources SET current_status='CACHED', updated_at=? WHERE id=?")
            .bind(now())
            .bind(&source.id)
            .execute(pool)
            .await?;
        return Ok(preview(
            &source.id,
            "CACHED",
            "Usando datos en caché dentro del cooldown configurado.".into(),
            source.last_http_status,
            vec![],
        ));
    }
    let started_at = now();
    let timer = Instant::now();
    sqlx::query("UPDATE market_sources SET current_status='FETCHING', last_request_at=?, updated_at=? WHERE id=?")
        .bind(&started_at).bind(&started_at).bind(&source.id).execute(pool).await?;
    let response = match fetch_once(client, url).await {
        Ok(response) => response,
        Err(first_error) => {
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            match fetch_once(client, url).await {
                Ok(response) => response,
                Err(_) => {
                    let message = first_error.to_string();
                    record_log(
                        pool,
                        source,
                        url,
                        &started_at,
                        FetchLogOutcome {
                            status: "ERROR",
                            http_status: None,
                            duration_ms: timer.elapsed().as_millis() as i64,
                            cache_hit: false,
                            observation_count: 0,
                            error_type: Some("NETWORK"),
                            error_message: Some(&message),
                        },
                    )
                    .await?;
                    sqlx::query("UPDATE market_sources SET current_status='ERROR', last_failure_at=?, consecutive_failures=consecutive_failures+1, last_error=?, updated_at=? WHERE id=?")
                        .bind(now()).bind(&message).bind(now()).bind(&source.id).execute(pool).await?;
                    return Ok(preview(&source.id, "ERROR", message, None, vec![]));
                }
            }
        }
    };
    if let Some(reason) = blocked_reason(&response) {
        let cooldown = Utc::now()
            + Duration::seconds(
                response
                    .retry_after_seconds
                    .unwrap_or(86_400)
                    .clamp(3_600, 604_800),
            );
        record_log(
            pool,
            source,
            &response.final_url,
            &started_at,
            FetchLogOutcome {
                status: "BLOCKED",
                http_status: Some(response.http_status as i64),
                duration_ms: timer.elapsed().as_millis() as i64,
                cache_hit: false,
                observation_count: 0,
                error_type: Some("BLOCKED"),
                error_message: Some(&reason),
            },
        )
        .await?;
        sqlx::query("UPDATE market_sources SET current_status='BLOCKED', automation_status='BLOCKED', last_failure_at=?, consecutive_failures=consecutive_failures+1, last_http_status=?, last_error=?, cooldown_until=?, updated_at=? WHERE id=?")
            .bind(now()).bind(response.http_status as i64).bind(&reason).bind(cooldown.to_rfc3339()).bind(now()).bind(&source.id).execute(pool).await?;
        return Ok(preview(
            &source.id,
            "BLOCKED",
            reason,
            Some(response.http_status as i64),
            vec![],
        ));
    }
    if response.http_status >= 400 {
        let message = format!("La fuente respondió HTTP {}.", response.http_status);
        record_log(
            pool,
            source,
            &response.final_url,
            &started_at,
            FetchLogOutcome {
                status: "ERROR",
                http_status: Some(response.http_status as i64),
                duration_ms: timer.elapsed().as_millis() as i64,
                cache_hit: false,
                observation_count: 0,
                error_type: Some("HTTP"),
                error_message: Some(&message),
            },
        )
        .await?;
        sqlx::query("UPDATE market_sources SET current_status='ERROR', last_failure_at=?, consecutive_failures=consecutive_failures+1, last_http_status=?, last_error=?, updated_at=? WHERE id=?")
            .bind(now()).bind(response.http_status as i64).bind(&message).bind(now()).bind(&source.id).execute(pool).await?;
        return Ok(preview(
            &source.id,
            "ERROR",
            message,
            Some(response.http_status as i64),
            vec![],
        ));
    }
    let mut drafts = extract_with_adapter(&response.body, source, context, &response.final_url)?;
    let mut accepted = Vec::new();
    for mut draft in drafts.drain(..) {
        if validate_observation(&mut draft).is_ok() {
            accepted.push(draft);
        }
    }
    if accepted.is_empty() {
        let message = "La página respondió, pero el adapter no encontró datos confirmables. Revisá la configuración; no se inventaron selectores.".to_string();
        record_log(
            pool,
            source,
            &response.final_url,
            &started_at,
            FetchLogOutcome {
                status: "NEEDS_CONFIGURATION",
                http_status: Some(response.http_status as i64),
                duration_ms: timer.elapsed().as_millis() as i64,
                cache_hit: false,
                observation_count: 0,
                error_type: Some("EMPTY_EXTRACTION"),
                error_message: Some(&message),
            },
        )
        .await?;
        sqlx::query("UPDATE market_sources SET current_status='NEEDS_CONFIGURATION', last_failure_at=?, last_http_status=?, last_error=?, updated_at=? WHERE id=?")
            .bind(now()).bind(response.http_status as i64).bind(&message).bind(now()).bind(&source.id).execute(pool).await?;
        return Ok(preview(
            &source.id,
            "NEEDS_CONFIGURATION",
            message,
            Some(response.http_status as i64),
            vec![],
        ));
    }
    let count = accepted.len() as i64;
    if persist {
        for draft in accepted.iter().cloned() {
            let _ = persist_draft(pool, source, draft, "AUTO").await?;
        }
    }
    let cooldown = Utc::now() + Duration::hours(source.cooldown_hours.unwrap_or(24).max(0));
    sqlx::query("UPDATE market_sources SET current_status='SUCCESS', last_success_at=?, consecutive_failures=0, last_http_status=?, last_error=NULL, cooldown_until=?, updated_at=? WHERE id=?")
        .bind(now()).bind(response.http_status as i64).bind(cooldown.to_rfc3339()).bind(now()).bind(&source.id).execute(pool).await?;
    record_log(
        pool,
        source,
        &response.final_url,
        &started_at,
        FetchLogOutcome {
            status: "SUCCESS",
            http_status: Some(response.http_status as i64),
            duration_ms: timer.elapsed().as_millis() as i64,
            cache_hit: false,
            observation_count: count,
            error_type: None,
            error_message: None,
        },
    )
    .await?;
    Ok(preview(
        &source.id,
        "SUCCESS",
        format!("Se detectaron {count} observaciones trazables."),
        Some(response.http_status as i64),
        accepted,
    ))
}

pub async fn test_source(state: &AppState, source_id: &str) -> AppResult<SourceTestResult> {
    let source = source_by_id(&state.pool, source_id).await?;
    let service = serde_json::from_str::<Vec<String>>(&source.supported_services_json)
        .unwrap_or_default()
        .into_iter()
        .next()
        .unwrap_or_else(|| "video-editing".into());
    let context = MarketQueryContext::generic(service, regions(&source));
    let mut testing_source = source.clone();
    testing_source.enabled = true;
    testing_source.acquisition_mode = "auto_http".into();
    testing_source.automation_status = "APPROVED".into();
    let result = process_source(
        &state.pool,
        &state.http,
        &state.market_request_lock,
        &testing_source,
        &context,
        true,
        false,
    )
    .await?;
    if result.status == "SUCCESS" {
        sqlx::query("UPDATE market_sources SET last_success_at=?, current_status=CASE WHEN automation_status='UNREVIEWED' THEN 'NEEDS_CONFIGURATION' ELSE current_status END, updated_at=? WHERE id=?")
            .bind(now()).bind(now()).bind(source_id).execute(&state.pool).await?;
    }
    Ok(result)
}

pub async fn approve_source(state: &AppState, source_id: &str) -> AppResult<()> {
    let source = source_by_id(&state.pool, source_id).await?;
    if source.last_success_at.is_none() {
        return Err(AppError::Validation(
            "Primero probá la fuente y confirmá que detecta datos válidos.".into(),
        ));
    }
    sqlx::query("UPDATE market_sources SET automation_status='APPROVED', acquisition_mode='auto_http', adapter_key=COALESCE(adapter_key,'generic'), current_status='READY', enabled=1, updated_at=? WHERE id=?")
        .bind(now()).bind(source_id).execute(&state.pool).await?;
    Ok(())
}

pub async fn refresh_single_source(
    state: &AppState,
    source_id: &str,
    force: bool,
) -> AppResult<SourceTestResult> {
    let source = source_by_id(&state.pool, source_id).await?;
    let service = serde_json::from_str::<Vec<String>>(&source.supported_services_json)
        .unwrap_or_default()
        .into_iter()
        .next()
        .unwrap_or_else(|| "video-editing".into());
    let context = MarketQueryContext::generic(service, regions(&source));
    process_source(
        &state.pool,
        &state.http,
        &state.market_request_lock,
        &source,
        &context,
        force,
        true,
    )
    .await
}

pub async fn create_manual_observation(
    state: &AppState,
    input: ManualObservationInput,
) -> AppResult<MarketObservation> {
    let source = source_by_id(&state.pool, &input.source_id).await?;
    let source_url = if input.source_url.trim().is_empty() {
        format!("pricing-os://manual/{}", source.id)
    } else {
        validate_public_https(&input.source_url)?.to_string()
    };
    let original = match (
        input.price_min_minor,
        input.price_max_minor,
        input.price_value_minor,
    ) {
        (_, _, Some(value)) => value.to_string(),
        (Some(min), Some(max), _) => format!("{min}–{max}"),
        (Some(min), _, _) => min.to_string(),
        (_, Some(max), _) => max.to_string(),
        _ => String::new(),
    };
    let is_salary = matches!(
        input.price_type.as_str(),
        "MONTHLY_SALARY" | "ANNUAL_SALARY"
    );
    let draft = ObservationDraft {
        service_type: input.service_type,
        subservice: input.subservice,
        category: input.category,
        region: input.region,
        country: input.country,
        currency: input.currency,
        price_type: input.price_type,
        unit: input.unit,
        price_min_minor: input.price_min_minor,
        price_max_minor: input.price_max_minor,
        price_value_minor: input.price_value_minor,
        original_value_text: original,
        experience_level: input.experience_level,
        client_tier: input.client_tier,
        source_url,
        published_at: input.published_at,
        confidence: "HIGH".into(),
        comparison_eligibility: if is_salary {
            "CONTEXT_ONLY"
        } else {
            "ELIGIBLE"
        }
        .into(),
        exclusion_reason: is_salary.then_some("Salario guardado como contexto separado.".into()),
        evidence_snippet: None,
        notes: input.notes,
    };
    let (id, _) = persist_draft(&state.pool, &source, draft, "MANUAL").await?;
    observation_by_id(&state.pool, &id).await
}

async fn observation_by_id(pool: &SqlitePool, id: &str) -> AppResult<MarketObservation> {
    sqlx::query_as::<_, MarketObservation>(&format!("SELECT {OBSERVATION_COLUMNS} FROM market_observations o JOIN market_sources s ON s.id=o.source_id WHERE o.id=?"))
        .bind(id).fetch_optional(pool).await?.ok_or(AppError::NotFound)
}

pub async fn list_observations(
    state: &AppState,
    filter: MarketObservationFilter,
) -> AppResult<Vec<MarketObservation>> {
    let rows = sqlx::query_as::<_, MarketObservation>(&format!("SELECT {OBSERVATION_COLUMNS} FROM market_observations o JOIN market_sources s ON s.id=o.source_id ORDER BY o.retrieved_at DESC LIMIT 1000"))
        .fetch_all(&state.pool).await?;
    let query = filter.query.unwrap_or_default().to_lowercase();
    Ok(rows
        .into_iter()
        .filter(|item| {
            filter
                .service_type
                .as_ref()
                .is_none_or(|value| &item.service_type == value)
                && filter
                    .region
                    .as_ref()
                    .is_none_or(|value| &item.region == value)
                && filter
                    .source_id
                    .as_ref()
                    .is_none_or(|value| &item.source_id == value)
                && filter
                    .price_type
                    .as_ref()
                    .is_none_or(|value| &item.price_type == value)
                && filter
                    .currency
                    .as_ref()
                    .is_none_or(|value| &item.currency == value)
                && (query.is_empty()
                    || item.source_name.to_lowercase().contains(&query)
                    || item
                        .subservice
                        .as_deref()
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains(&query))
        })
        .collect())
}

pub async fn list_snapshots(
    state: &AppState,
    quote_service_id: Option<&str>,
) -> AppResult<Vec<MarketSnapshot>> {
    let sql = "SELECT id, quote_id, quote_service_id, query_context_json, currency, observation_count, comparable_observation_count, source_count, minimum_filtered_minor, p25_minor, market_median_minor, p75_minor, maximum_filtered_minor, confidence_level, calculated_price_minor, suggested_price_minor, final_price_minor_at_creation, summary_json, created_at FROM market_snapshots";
    if let Some(id) = quote_service_id {
        Ok(sqlx::query_as::<_, MarketSnapshot>(&format!(
            "{sql} WHERE quote_service_id=? ORDER BY created_at DESC"
        ))
        .bind(id)
        .fetch_all(&state.pool)
        .await?)
    } else {
        Ok(sqlx::query_as::<_, MarketSnapshot>(&format!(
            "{sql} ORDER BY created_at DESC LIMIT 300"
        ))
        .fetch_all(&state.pool)
        .await?)
    }
}

pub async fn market_overview(
    state: &AppState,
    quote_service_id: &str,
) -> AppResult<MarketOverview> {
    let history = list_snapshots(state, Some(quote_service_id)).await?;
    let latest_snapshot = history.first().cloned();
    let observations = if let Some(snapshot) = &latest_snapshot {
        let snapshot_columns = OBSERVATION_COLUMNS
            .replace("o.converted_value_minor", "so.converted_value_minor")
            .replace("o.converted_currency", "so.converted_currency")
            .replace("o.exchange_rate_micros", "so.exchange_rate_micros")
            .replace("o.exchange_rate_date", "so.exchange_rate_date")
            .replace("o.exchange_rate_source", "so.exchange_rate_source")
            .replace(
                "NULL AS snapshot_included",
                "so.included AS snapshot_included",
            )
            .replace(
                "NULL AS snapshot_exclusion_reason",
                "so.exclusion_reason AS snapshot_exclusion_reason",
            )
            .replace(
                "NULL AS snapshot_normalized_value_minor",
                "so.normalized_value_minor AS snapshot_normalized_value_minor",
            );
        sqlx::query_as::<_, MarketObservation>(&format!("SELECT {snapshot_columns} FROM market_observations o JOIN market_sources s ON s.id=o.source_id JOIN market_snapshot_observations so ON so.observation_id=o.id WHERE so.snapshot_id=? ORDER BY so.included DESC, s.name, o.price_value_minor"))
            .bind(&snapshot.id).fetch_all(&state.pool).await?
    } else {
        vec![]
    };
    Ok(MarketOverview {
        latest_snapshot,
        observations,
        history,
    })
}

fn parse_duration_minutes(raw: Option<&str>) -> Option<f64> {
    let (minutes, seconds) = raw?.split_once(':')?;
    Some(minutes.parse::<f64>().ok()? + seconds.parse::<f64>().ok()? / 60.0)
}

fn query_context(
    service_type: &str,
    configuration_json: &str,
    market_scope: Option<&str>,
) -> MarketQueryContext {
    let json: Value = serde_json::from_str(configuration_json).unwrap_or(Value::Null);
    let data = json.get("data").unwrap_or(&Value::Null);
    let values = if service_type == "programming" {
        data.get("parameterValues").unwrap_or(data)
    } else {
        data
    };
    let subtype = values
        .get(if service_type == "video-editing" {
            "pieceType"
        } else {
            "projectType"
        })
        .and_then(Value::as_str)
        .map(str::to_string);
    let level = values
        .get(if service_type == "video-editing" {
            "editingLevel"
        } else {
            "complexity"
        })
        .and_then(Value::as_str)
        .map(str::to_string);
    let duration_minutes =
        parse_duration_minutes(values.get("finalDuration").and_then(Value::as_str));
    let quantity = values.get("quantity").and_then(Value::as_f64);
    let estimated_hours = values.get("estimatedHours").and_then(Value::as_f64);
    let regions = match market_scope {
        Some("argentina") => vec!["AR".into()],
        Some("international") => vec!["INTERNATIONAL".into()],
        _ => vec!["AR".into(), "INTERNATIONAL".into(), "LATAM".into()],
    };
    let mut features = Vec::new();
    let allowed_features: &[&str] = if service_type == "video-editing" {
        &[
            "resolution",
            "editingLevel",
            "revisions",
            "urgency",
            "formats",
            "color",
            "audio",
            "subtitles",
            "videoAi",
            "voiceAi",
            "soundAi",
            "backgroundRemoval",
            "motion",
            "broll",
            "additionalVersions",
        ]
    } else {
        &[
            "projectType",
            "frontend",
            "backend",
            "database",
            "auth",
            "integrations",
            "screens",
            "responsive",
            "deploy",
            "ai",
            "complexity",
        ]
    };
    if let Some(object) = values.as_object() {
        for (key, value) in object {
            if !allowed_features.contains(&key.as_str()) {
                continue;
            }
            let meaningful = value.as_bool().unwrap_or(false)
                || value
                    .as_str()
                    .is_some_and(|value| !matches!(value, "" | "none" | "basic" | "normal"))
                || value.as_array().is_some_and(|values| !values.is_empty());
            if meaningful {
                features.push(key.clone());
            }
        }
    }
    MarketQueryContext {
        service: service_type.into(),
        subtype,
        region_targets: regions,
        level,
        duration_minutes,
        quantity,
        estimated_hours,
        features,
    }
}

async fn update_job(state: &AppState, id: &str, update: impl FnOnce(&mut MarketResearchJob)) {
    if let Some(job) = state.market_jobs.lock().await.get_mut(id) {
        update(job);
    }
}

pub async fn start_job_record(
    state: &AppState,
    quote_service_id: &str,
) -> AppResult<MarketResearchJob> {
    if state
        .market_jobs
        .lock()
        .await
        .values()
        .any(|job| job.status == "RUNNING")
    {
        return Err(AppError::Validation(
            "Ya hay una actualización de mercado en curso. Evitamos consultas duplicadas.".into(),
        ));
    }
    let service_type: String = sqlx::query_scalar(
        "SELECT service_type FROM quote_services WHERE id=? AND deleted_at IS NULL",
    )
    .bind(quote_service_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;
    let sources = enabled_sources(&state.pool, &service_type).await?;
    let id = Uuid::new_v4().to_string();
    let job = MarketResearchJob {
        id: id.clone(),
        quote_service_id: quote_service_id.into(),
        status: "RUNNING".into(),
        completed: 0,
        total: sources.len() as i64,
        cancel_requested: false,
        items: sources
            .into_iter()
            .map(|source| MarketResearchJobItem {
                source_id: source.id,
                source_name: source.name,
                status: "READY".into(),
                message: None,
                observation_count: 0,
            })
            .collect(),
        snapshot_id: None,
        error: None,
        started_at: now(),
        finished_at: None,
    };
    state.market_jobs.lock().await.insert(id, job.clone());
    Ok(job)
}

pub async fn get_job(state: &AppState, id: &str) -> AppResult<MarketResearchJob> {
    state
        .market_jobs
        .lock()
        .await
        .get(id)
        .cloned()
        .ok_or(AppError::NotFound)
}

pub async fn cancel_job(state: &AppState, id: &str) -> AppResult<MarketResearchJob> {
    let mut jobs = state.market_jobs.lock().await;
    let job = jobs.get_mut(id).ok_or(AppError::NotFound)?;
    job.cancel_requested = true;
    Ok(job.clone())
}

pub async fn run_research_job(state: AppState, job_id: String, force: bool) {
    if let Err(error) = run_research_inner(&state, &job_id, force).await {
        update_job(&state, &job_id, |job| {
            job.status = "ERROR".into();
            job.error = Some(error.to_string());
            job.finished_at = Some(now());
        })
        .await;
    }
}

async fn run_research_inner(state: &AppState, job_id: &str, force: bool) -> AppResult<()> {
    let job = get_job(state, job_id).await?;
    let row = sqlx::query("SELECT qs.quote_id, qs.service_type, qs.configuration_json, qs.calculated_subtotal_minor, qs.final_subtotal_minor, qs.has_override, q.currency, p.market_scope FROM quote_services qs JOIN quotes q ON q.id=qs.quote_id JOIN projects p ON p.id=q.project_id WHERE qs.id=?")
        .bind(&job.quote_service_id).fetch_one(&state.pool).await?;
    let quote_id: String = row.try_get("quote_id")?;
    let service_type: String = row.try_get("service_type")?;
    let configuration_json: String = row.try_get("configuration_json")?;
    let calculated: Option<i64> = row.try_get("calculated_subtotal_minor")?;
    let final_price: Option<i64> = row.try_get("final_subtotal_minor")?;
    let has_override: bool = row.try_get("has_override")?;
    let currency: String = row.try_get("currency")?;
    let market_scope: Option<String> = row.try_get("market_scope")?;
    let context = query_context(&service_type, &configuration_json, market_scope.as_deref());
    let sources = enabled_sources(&state.pool, &service_type).await?;
    for source in &sources {
        if get_job(state, job_id).await?.cancel_requested {
            update_job(state, job_id, |job| {
                job.status = "CANCELLED".into();
                job.finished_at = Some(now());
            })
            .await;
            return Ok(());
        }
        update_job(state, job_id, |job| {
            if let Some(item) = job
                .items
                .iter_mut()
                .find(|item| item.source_id == source.id)
            {
                item.status = "FETCHING".into();
            }
        })
        .await;
        let result = process_source(
            &state.pool,
            &state.http,
            &state.market_request_lock,
            source,
            &context,
            force,
            true,
        )
        .await
        .unwrap_or_else(|error| preview(&source.id, "ERROR", error.to_string(), None, vec![]));
        update_job(state, job_id, |job| {
            if let Some(item) = job
                .items
                .iter_mut()
                .find(|item| item.source_id == source.id)
            {
                item.status = result.status.clone();
                item.message = Some(result.message.clone());
                item.observation_count = result.observations.len() as i64;
            }
            job.completed += 1;
        })
        .await;
    }
    let finished_job = get_job(state, job_id).await?;
    let live_result_count = finished_job
        .items
        .iter()
        .filter(|item| matches!(item.status.as_str(), "SUCCESS" | "CACHED"))
        .count();
    let live_failure_count = finished_job
        .items
        .iter()
        .filter(|item| {
            matches!(
                item.status.as_str(),
                "ERROR" | "BLOCKED" | "NEEDS_CONFIGURATION"
            )
        })
        .count();
    if live_result_count == 0 && live_failure_count > 0 {
        if let Some(previous) = list_snapshots(state, Some(&job.quote_service_id))
            .await?
            .into_iter()
            .next()
        {
            update_job(state, job_id, |job| {
                job.status = "COMPLETED".into();
                job.snapshot_id = Some(previous.id);
                job.error = Some(format!(
                    "Sin actualización en vivo. Se conserva el último snapshot del {}.",
                    previous.created_at
                ));
                job.finished_at = Some(now());
            })
            .await;
            return Ok(());
        }
    }
    let enabled_source_ids = sources
        .iter()
        .map(|source| source.id.clone())
        .collect::<HashSet<_>>();
    let participating_source_ids =
        participating_sources(&state.pool, &service_type, &sources).await?;
    let observations = list_observations(
        state,
        MarketObservationFilter {
            service_type: Some(service_type.clone()),
            ..Default::default()
        },
    )
    .await?
    .into_iter()
    .filter(|observation| enabled_source_ids.contains(&observation.source_id))
    .collect::<Vec<_>>();
    let official_rate: Option<(i64, String, String)> = sqlx::query_as("SELECT rate_micros, rate_date, source_url FROM market_fx_rates WHERE base_currency='USD' AND quote_currency='ARS' ORDER BY rate_date DESC, retrieved_at DESC LIMIT 1").fetch_optional(&state.pool).await?;
    let manual_rate: Option<i64> =
        sqlx::query_scalar("SELECT usd_to_ars_micros FROM app_settings WHERE id=1")
            .fetch_one(&state.pool)
            .await?;
    let (rate, rate_date, rate_source) = if let Some((rate, date, source)) = official_rate {
        (Some(rate), Some(date), Some(source))
    } else if let Some(rate) = manual_rate {
        (
            Some(rate),
            Some(Utc::now().date_naive().to_string()),
            Some("Configuración manual de Pricing OS".to_string()),
        )
    } else {
        (None, None, None)
    };
    let (compared, summary) = compare_market(
        &observations,
        &context,
        &currency,
        rate,
        &participating_source_ids,
    );
    let settings =
        sqlx::query("SELECT suggestions_enabled, suggestion_strategy FROM app_settings WHERE id=1")
            .fetch_one(&state.pool)
            .await?;
    let suggestions_enabled: bool = settings.try_get("suggestions_enabled")?;
    let strategy: String = settings.try_get("suggestion_strategy")?;
    let suggested = suggestions_enabled
        .then(|| suggested_with_market(calculated, &summary, &strategy))
        .flatten();
    let snapshot_id = Uuid::new_v4().to_string();
    let summary_json = json!({
        "schemaVersion": 1, "explanations": summary.explanations, "strategy": strategy,
        "marketSufficient": summary.confidence_level != "INSUFFICIENT",
        "suggestionExplanation": if suggested.is_some() { "Combina 40% del cálculo interno y 60% de la zona de mercado elegida, sin bajar del cálculo." } else { "Sugerencia sin referencia externa suficiente." },
        "finalPriceProtected": true,
        "fxRateMicros": rate,
        "fxRateDate": rate_date,
        "fxRateSource": rate_source,
        "usedCachedObservations": live_result_count == 0,
    }).to_string();
    let timestamp = now();
    let mut tx = state.pool.begin().await?;
    sqlx::query("INSERT INTO market_snapshots (id, quote_id, quote_service_id, query_context_json, currency, observation_count, comparable_observation_count, source_count, minimum_filtered_minor, p25_minor, market_median_minor, p75_minor, maximum_filtered_minor, confidence_level, calculated_price_minor, suggested_price_minor, final_price_minor_at_creation, summary_json, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&snapshot_id).bind(&quote_id).bind(&job.quote_service_id).bind(serde_json::to_string(&context)?)
        .bind(&currency).bind(observations.len() as i64).bind(summary.comparable_count).bind(summary.source_count)
        .bind(summary.minimum_filtered_minor).bind(summary.p25_minor).bind(summary.median_minor).bind(summary.p75_minor).bind(summary.maximum_filtered_minor)
        .bind(&summary.confidence_level).bind(calculated).bind(suggested).bind(final_price).bind(&summary_json).bind(&timestamp)
        .execute(&mut *tx).await?;
    for item in &compared {
        let observation = observations
            .iter()
            .find(|observation| observation.id == item.observation_id)
            .ok_or_else(|| AppError::Validation("Observación de snapshot inexistente.".into()))?;
        let cross_currency = observation.currency != currency;
        let converted = converted_midpoint(observation, &currency, rate);
        sqlx::query("INSERT INTO market_snapshot_observations (snapshot_id, observation_id, included, exclusion_reason, normalized_value_minor, converted_value_minor, converted_currency, exchange_rate_micros, exchange_rate_date, exchange_rate_source) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(&snapshot_id).bind(&item.observation_id).bind(item.included).bind(&item.reason)
            .bind(item.normalized_value_minor).bind(cross_currency.then_some(converted).flatten())
            .bind(cross_currency.then_some(currency.as_str())).bind(cross_currency.then_some(rate).flatten())
            .bind(cross_currency.then_some(rate_date.as_deref()).flatten())
            .bind(cross_currency.then_some(rate_source.as_deref()).flatten())
            .execute(&mut *tx).await?;
    }
    if suggestions_enabled {
        if let Some(market_suggestion) = suggested {
            let mut pricing_snapshot: Option<String> =
                sqlx::query_scalar("SELECT pricing_snapshot_json FROM quote_services WHERE id=?")
                    .bind(&job.quote_service_id)
                    .fetch_one(&mut *tx)
                    .await?;
            if let Some(raw) = pricing_snapshot.take() {
                if let Ok(mut json) = serde_json::from_str::<Value>(&raw) {
                    if let Some(slot) = json.pointer_mut("/result/suggestedSubtotalMinor") {
                        *slot = Value::from(market_suggestion);
                    }
                    if let Some(slot) = json.pointer_mut("/result/finalSubtotalMinor") {
                        *slot = final_price.map(Value::from).unwrap_or(Value::Null);
                    }
                    sqlx::query("UPDATE quote_services SET suggested_subtotal_minor=?, pricing_snapshot_json=?, updated_at=? WHERE id=?")
                        .bind(market_suggestion).bind(json.to_string()).bind(&timestamp).bind(&job.quote_service_id).execute(&mut *tx).await?;
                } else {
                    sqlx::query("UPDATE quote_services SET suggested_subtotal_minor=?, updated_at=? WHERE id=?").bind(market_suggestion).bind(&timestamp).bind(&job.quote_service_id).execute(&mut *tx).await?;
                }
            } else {
                sqlx::query(
                    "UPDATE quote_services SET suggested_subtotal_minor=?, updated_at=? WHERE id=?",
                )
                .bind(market_suggestion)
                .bind(&timestamp)
                .bind(&job.quote_service_id)
                .execute(&mut *tx)
                .await?;
            }
        }
    }
    // El precio final y su override nunca se actualizan durante investigación de mercado.
    let protected_final: (Option<i64>, bool) =
        sqlx::query_as("SELECT final_subtotal_minor, has_override FROM quote_services WHERE id=?")
            .bind(&job.quote_service_id)
            .fetch_one(&mut *tx)
            .await?;
    if protected_final != (final_price, has_override) {
        return Err(AppError::Validation(
            "La protección del precio final detectó una mutación inesperada.".into(),
        ));
    }
    tx.commit().await?;
    update_job(state, job_id, |job| {
        job.status = "COMPLETED".into();
        job.snapshot_id = Some(snapshot_id);
        job.finished_at = Some(now());
        if live_result_count == 0 && live_failure_count > 0 {
            job.error = Some(
                "No hubo respuesta en vivo; el snapshot usa observaciones trazables guardadas."
                    .into(),
            );
        }
    })
    .await;
    Ok(())
}

pub fn open_source(raw_url: &str) -> AppResult<()> {
    let url = validate_public_https(raw_url)?.to_string();
    #[cfg(target_os = "windows")]
    Command::new("rundll32")
        .args(["url.dll,FileProtocolHandler", &url])
        .spawn()?;
    #[cfg(target_os = "macos")]
    Command::new("open").arg(&url).spawn()?;
    #[cfg(all(unix, not(target_os = "macos")))]
    Command::new("xdg-open").arg(&url).spawn()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_context_contains_only_abstract_service_information() {
        let json = r#"{"data":{"pieceType":"youtube","finalDuration":"10:30","estimatedHours":8,"quantity":2,"clientEmail":"private@example.com","subtitles":"designed"}}"#;
        let context = query_context("video-editing", json, Some("both"));
        let serialized = serde_json::to_string(&context).unwrap();
        assert_eq!(context.duration_minutes, Some(10.5));
        assert!(!serialized.contains("private@example.com"));
        assert!(!serialized.contains("clientEmail"));
    }

    #[test]
    fn cooldown_recognizes_future_and_expired_dates() {
        let future = (Utc::now() + Duration::hours(2)).to_rfc3339();
        let expired = (Utc::now() - Duration::hours(2)).to_rfc3339();
        assert!(cooldown_active(Some(&future)));
        assert!(!cooldown_active(Some(&expired)));
        assert!(!cooldown_active(None));
        assert!(should_use_cache(Some(&future), false));
        assert!(!should_use_cache(Some(&future), true));
    }
}
