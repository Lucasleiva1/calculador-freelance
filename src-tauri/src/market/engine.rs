use std::{collections::HashSet, process::Command, time::Instant};

use chrono::{DateTime, Duration, SecondsFormat, Utc};
use reqwest::Client;
use serde_json::{json, Value};
use sqlx::{Row, Sqlite, SqlitePool, Transaction};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    db::AppState,
    error::{AppError, AppResult},
    models::{
        ManualObservationInput, MarketObservation, MarketObservationFilter,
        MarketObservationPreview, MarketOverview, MarketResearchBaseline, MarketResearchJob,
        MarketResearchJobItem, MarketSnapshot, MarketSource, SourceTestResult,
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
    let assignment_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pricing_engine_sources pes
         JOIN pricing_engines pe ON pe.id=pes.engine_id
         WHERE pe.engine_key=? AND pes.preference<>'excluded'",
    )
    .bind(service_type)
    .fetch_one(pool)
    .await?;
    let assigned: Vec<String> = sqlx::query_scalar(
        "SELECT pes.source_id FROM pricing_engine_sources pes
         JOIN pricing_engines pe ON pe.id=pes.engine_id
         JOIN market_sources ms ON ms.id=pes.source_id
         WHERE pe.engine_key=? AND pes.preference<>'excluded'
           AND pes.role='reference'
           AND pes.participates_in_suggestions=1
           AND ms.enabled=1 AND ms.archived_at IS NULL
           AND ms.usage_mode='market_price'
           AND ms.source_type NOT IN ('salary','job_board','methodology','currency')
           AND ms.participates_in_suggestions=1",
    )
    .bind(service_type)
    .fetch_all(pool)
    .await?;
    if assignment_count > 0 {
        return Ok(assigned.into_iter().collect());
    }
    Ok(sources
        .iter()
        .filter(|source| source_supports_suggestions(source))
        .map(|source| source.id.clone())
        .collect())
}

fn source_supports_suggestions(source: &MarketSource) -> bool {
    source.enabled
        && source.participates_in_suggestions
        && source_kind_supports_suggestions(&source.usage_mode, &source.source_type)
}

fn source_kind_supports_suggestions(usage_mode: &str, source_type: &str) -> bool {
    usage_mode == "market_price"
        && !matches!(
            source_type,
            "salary" | "job_board" | "methodology" | "currency"
        )
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

fn source_request_url(source: &MarketSource, context: &MarketQueryContext) -> AppResult<String> {
    if source.adapter_key.as_deref() == Some("upwork") {
        return Ok(match context.service.as_str() {
            "video-editing" => "https://www.upwork.com/hire/video-editors/cost/",
            "programming" => "https://www.upwork.com/hire/software-developers/cost/",
            "print-design" => "https://www.upwork.com/hire/graphic-designers/cost/",
            _ => {
                return Err(AppError::Validation(
                    "Upwork no tiene una especialidad aprobada para este motor.".into(),
                ))
            }
        }
        .into());
    }
    source
        .base_url
        .clone()
        .ok_or_else(|| AppError::Validation("La fuente no tiene URL base.".into()))
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
    if source.acquisition_mode == "disabled" || !source.enabled {
        return Ok(preview(
            &source.id,
            "DISABLED",
            "La fuente está desactivada.".into(),
            None,
            vec![],
        ));
    }
    if source.acquisition_mode == "manual" {
        let message = if source_supports_suggestions(source) {
            "Fuente manual: cargá evidencia verificada para que pueda participar como referencia."
        } else {
            "Fuente de contexto manual: no se consulta automáticamente ni genera sugerencias de precio."
        };
        return Ok(preview(&source.id, "MANUAL", message.into(), None, vec![]));
    }
    let url = source_request_url(source, context)?;
    validate_public_https(&url)?;
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
            &url,
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
    let response = match fetch_once(client, &url).await {
        Ok(response) => response,
        Err(first_error) => {
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            match fetch_once(client, &url).await {
                Ok(response) => response,
                Err(_) => {
                    let message = first_error.to_string();
                    record_log(
                        pool,
                        source,
                        &url,
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
    let sql = "SELECT id, quote_id, quote_service_id, query_context_json, currency, observation_count, comparable_observation_count, source_count, minimum_filtered_minor, p25_minor, market_median_minor, p75_minor, maximum_filtered_minor, confidence_level, calculated_price_minor, suggested_price_minor, final_price_minor_at_creation, base_service_revision, suggestion_update_status, suggestion_update_message, summary_json, created_at FROM market_snapshots";
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
    let values = data.get("parameterValues").unwrap_or(data);
    let subtype = values
        .get(match service_type {
            "video-editing" => "pieceType",
            "print-design" => "productType",
            _ => "projectType",
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
    let client_tier = if service_type == "print-design" {
        values.get("clientTier").and_then(Value::as_str).and_then(|tier| match tier {
            "small" | "C" => Some("C".to_string()),
            "medium" | "B" => Some("B".to_string()),
            "large" | "A" => Some("A".to_string()),
            _ => None,
        })
    } else { None };
    let work_class = if service_type == "print-design" {
        let tasks = values.get("workTasks").and_then(Value::as_array);
        let has_task = |wanted: &str| tasks.is_some_and(|items| items.iter().any(|item| item.as_str() == Some(wanted)));
        if values.get("hasReference").and_then(Value::as_bool) == Some(false) || has_task("design-from-scratch") {
            Some("original".to_string())
        } else if ["adapt-composition", "grunge-borders", "ai-elements", "reconstruct-image"].iter().any(|task| has_task(task)) {
            Some("adaptation".to_string())
        } else { Some("preparation".to_string()) }
    } else { None };
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
    } else if service_type == "print-design" {
        &[
            "hasReference",
            "materialType",
            "clientTier",
            "productType",
            "garmentTone",
            "printSystem",
            "sublimationFitsA4",
            "workTasks",
            "complexity",
            "deliveryExtras",
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
        client_tier,
        work_class,
    }
}

async fn update_job(state: &AppState, id: &str, update: impl FnOnce(&mut MarketResearchJob)) {
    if let Some(job) = state.market_jobs.lock().await.get_mut(id) {
        update(job);
    }
}

/// Persiste una sugerencia solamente si el borrador sigue siendo exactamente el
/// que inició la investigación. No incrementa `row_revision`: una referencia de
/// mercado no es una edición del usuario y un autosave local posterior debe poder
/// reemplazar una sugerencia que ya quedó desactualizada.
async fn apply_market_suggestion_if_current(
    tx: &mut Transaction<'_, Sqlite>,
    job: &MarketResearchJob,
    suggestion: i64,
    timestamp: &str,
) -> AppResult<bool> {
    let baseline = &job.baseline;
    let updated = sqlx::query(
        "UPDATE quote_services
         SET suggested_subtotal_minor=?, updated_at=?
         WHERE id=? AND row_revision=? AND configuration_json=?
           AND final_subtotal_minor IS ? AND has_override=?
           AND deleted_at IS NULL",
    )
    .bind(suggestion)
    .bind(timestamp)
    .bind(&job.quote_service_id)
    .bind(job.base_service_revision)
    .bind(&baseline.configuration_json)
    .bind(baseline.final_price_minor)
    .bind(baseline.has_override)
    .execute(&mut **tx)
    .await?
    .rows_affected()
        == 1;
    if updated {
        sqlx::query("UPDATE quotes SET updated_at=? WHERE id=?")
            .bind(timestamp)
            .bind(&baseline.quote_id)
            .execute(&mut **tx)
            .await?;
    }
    Ok(updated)
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
    let row = sqlx::query(
        "SELECT qs.quote_id, qs.service_type, qs.configuration_json,
                qs.calculated_subtotal_minor, qs.final_subtotal_minor, qs.has_override,
                qs.row_revision, q.currency, p.market_scope
         FROM quote_services qs
         JOIN quotes q ON q.id=qs.quote_id
         JOIN projects p ON p.id=q.project_id
         WHERE qs.id=? AND qs.deleted_at IS NULL",
    )
    .bind(quote_service_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;
    let baseline = MarketResearchBaseline {
        quote_id: row.try_get("quote_id")?,
        service_type: row.try_get("service_type")?,
        configuration_json: row.try_get("configuration_json")?,
        calculated_price_minor: row.try_get("calculated_subtotal_minor")?,
        final_price_minor: row.try_get("final_subtotal_minor")?,
        has_override: row.try_get("has_override")?,
        currency: row.try_get("currency")?,
        market_scope: row.try_get("market_scope")?,
    };
    let base_service_revision: i64 = row.try_get("row_revision")?;
    let sources = enabled_sources(&state.pool, &baseline.service_type).await?;
    let id = Uuid::new_v4().to_string();
    let job = MarketResearchJob {
        id: id.clone(),
        quote_service_id: quote_service_id.into(),
        base_service_revision,
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
        suggestion_update_status: "PENDING".into(),
        suggestion_update_message: None,
        error: None,
        started_at: now(),
        finished_at: None,
        baseline,
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
    let baseline = &job.baseline;
    let quote_id = &baseline.quote_id;
    let service_type = &baseline.service_type;
    let calculated = baseline.calculated_price_minor;
    let final_price = baseline.final_price_minor;
    let currency = &baseline.currency;
    let context = query_context(
        service_type,
        &baseline.configuration_json,
        baseline.market_scope.as_deref(),
    );
    let sources = enabled_sources(&state.pool, service_type).await?;
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
        participating_sources(&state.pool, service_type, &sources).await?;
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
    let (rate, rate_date, rate_source) = if let Some((rate, date, source)) = official_rate {
        (Some(rate), Some(date), Some(source))
    } else {
        (None, None, None)
    };
    let mut local_context = context.clone();
    local_context.region_targets = vec!["AR".into()];
    let mut international_context = context.clone();
    international_context.region_targets = vec!["INTERNATIONAL".into()];
    let (local_compared, mut local_summary) = compare_market(
        &observations,
        &local_context,
        currency,
        rate,
        &participating_source_ids,
    );
    let (international_compared, mut international_summary) = compare_market(
        &observations,
        &international_context,
        currency,
        rate,
        &participating_source_ids,
    );
    if participating_source_ids.is_empty() {
        local_summary.explanations.push(
            "No hay fuentes de precio de mercado verificadas para sugerir. Las de moneda, salarios y metodología quedan sólo como contexto.".into(),
        );
        international_summary
            .explanations
            .push("No hay fuentes internacionales verificadas para sugerir.".into());
    }
    let settings =
        sqlx::query("SELECT suggestions_enabled, suggestion_strategy FROM app_settings WHERE id=1")
            .fetch_one(&state.pool)
            .await?;
    let suggestions_enabled: bool = settings.try_get("suggestions_enabled")?;
    let strategy: String = settings.try_get("suggestion_strategy")?;
    // Mercado e internacional son automáticos e independientes del precio
    // local/sostenible. Nunca se mezclan con la tarifa manual.
    let suggested = suggestions_enabled
        .then(|| suggested_with_market(None, &local_summary, &strategy))
        .flatten();
    let international_suggested = suggestions_enabled
        .then(|| suggested_with_market(None, &international_summary, &strategy))
        .flatten();
    let snapshot_id = Uuid::new_v4().to_string();
    let summary_json = json!({
        "schemaVersion": 2, "explanations": local_summary.explanations.clone(), "strategy": strategy,
        "marketSufficient": local_summary.confidence_level != "INSUFFICIENT",
        "suggestionExplanation": if suggested.is_some() { "Mediana o percentil del mercado argentino comparable, sin mezclar la tarifa manual ni fuentes globales." } else { "Sugerencia argentina sin referencia externa suficiente." },
        "pricingOptions": {
            "market": { "summary": local_summary, "suggestedPriceMinor": suggested, "region": "AR" },
            "international": { "summary": international_summary, "suggestedPriceMinor": international_suggested, "region": "INTERNATIONAL" }
        },
        "finalPriceProtected": true,
        "fxRateMicros": rate,
        "fxRateDate": rate_date,
        "fxRateSource": rate_source,
        "usedCachedObservations": live_result_count == 0,
    }).to_string();
    let timestamp = now();
    let mut tx = state.pool.begin().await?;
    let (suggestion_update_status, suggestion_update_message) = if !suggestions_enabled {
        (
            "DISABLED",
            Some(
                "Las sugerencias de mercado estan desactivadas; se guardo solo la evidencia."
                    .to_string(),
            ),
        )
    } else if let Some(market_suggestion) = suggested {
        if apply_market_suggestion_if_current(&mut tx, &job, market_suggestion, &timestamp).await? {
            (
                "APPLIED",
                Some("Se actualizó el Precio de mercado Argentina. El precio local, el internacional y el precio final no cambiaron.".to_string()),
            )
        } else {
            (
                "SKIPPED_DRAFT_CHANGED",
                Some("La cotizacion cambio mientras se investigaba. Se guardo la evidencia, sin modificar tus parametros ni tu precio final.".to_string()),
            )
        }
    } else {
        (
            "INSUFFICIENT",
            Some("No hay referencias argentinas suficientes para proponer el precio de mercado. El precio internacional se calculó por separado cuando hubo evidencia.".to_string()),
        )
    };
    sqlx::query("INSERT INTO market_snapshots (id, quote_id, quote_service_id, query_context_json, currency, observation_count, comparable_observation_count, source_count, minimum_filtered_minor, p25_minor, market_median_minor, p75_minor, maximum_filtered_minor, confidence_level, calculated_price_minor, suggested_price_minor, final_price_minor_at_creation, base_service_revision, suggestion_update_status, suggestion_update_message, summary_json, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&snapshot_id).bind(&quote_id).bind(&job.quote_service_id).bind(serde_json::to_string(&context)?)
        .bind(&currency).bind(observations.len() as i64).bind(local_summary.comparable_count).bind(local_summary.source_count)
        .bind(local_summary.minimum_filtered_minor).bind(local_summary.p25_minor).bind(local_summary.median_minor).bind(local_summary.p75_minor).bind(local_summary.maximum_filtered_minor)
        .bind(&local_summary.confidence_level).bind(calculated).bind(suggested).bind(final_price)
        .bind(job.base_service_revision).bind(suggestion_update_status).bind(&suggestion_update_message)
        .bind(&summary_json).bind(&timestamp)
        .execute(&mut *tx).await?;
    for observation in &observations {
        let local = local_compared
            .iter()
            .find(|item| item.observation_id == observation.id);
        let international = international_compared
            .iter()
            .find(|item| item.observation_id == observation.id);
        let included_local = local.is_some_and(|item| item.included);
        let included_international = international.is_some_and(|item| item.included);
        let included = included_local || included_international;
        let normalized = local
            .and_then(|item| item.normalized_value_minor)
            .or_else(|| international.and_then(|item| item.normalized_value_minor));
        let reason = if included_local {
            Some("Incluida en Precio de mercado Argentina".to_string())
        } else if included_international {
            Some("Incluida en Precio internacional".to_string())
        } else {
            local
                .and_then(|item| item.reason.clone())
                .or_else(|| international.and_then(|item| item.reason.clone()))
        };
        let cross_currency = observation.currency != currency.as_str();
        let converted = converted_midpoint(observation, currency, rate);
        sqlx::query("INSERT INTO market_snapshot_observations (snapshot_id, observation_id, included, exclusion_reason, normalized_value_minor, converted_value_minor, converted_currency, exchange_rate_micros, exchange_rate_date, exchange_rate_source) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(&snapshot_id).bind(&observation.id).bind(included).bind(&reason)
            .bind(normalized).bind(cross_currency.then_some(converted).flatten())
            .bind(cross_currency.then_some(currency.as_str())).bind(cross_currency.then_some(rate).flatten())
            .bind(cross_currency.then_some(rate_date.as_deref()).flatten())
            .bind(cross_currency.then_some(rate_source.as_deref()).flatten())
            .execute(&mut *tx).await?;
    }
    tx.commit().await?;
    update_job(state, job_id, |job| {
        job.status = "COMPLETED".into();
        job.snapshot_id = Some(snapshot_id);
        job.suggestion_update_status = suggestion_update_status.into();
        job.suggestion_update_message = suggestion_update_message;
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
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::str::FromStr;

    async fn suggestion_test_pool() -> SqlitePool {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .expect("valid sqlite options")
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("pool");
        sqlx::query("CREATE TABLE quotes (id TEXT PRIMARY KEY, updated_at TEXT NOT NULL)")
            .execute(&pool)
            .await
            .expect("quotes table");
        sqlx::query("INSERT INTO quotes (id,updated_at) VALUES ('quote','before')")
            .execute(&pool)
            .await
            .expect("quote");
        sqlx::query("CREATE TABLE quote_services (id TEXT PRIMARY KEY, suggested_subtotal_minor INTEGER, final_subtotal_minor INTEGER, has_override INTEGER NOT NULL, row_revision INTEGER NOT NULL, configuration_json TEXT NOT NULL, deleted_at TEXT, updated_at TEXT NOT NULL)")
            .execute(&pool)
            .await
            .expect("services table");
        pool
    }

    fn market_job(
        revision: i64,
        configuration: &str,
        final_price: Option<i64>,
        has_override: bool,
    ) -> MarketResearchJob {
        MarketResearchJob {
            id: "job".into(),
            quote_service_id: "service".into(),
            base_service_revision: revision,
            status: "RUNNING".into(),
            completed: 0,
            total: 0,
            cancel_requested: false,
            items: vec![],
            snapshot_id: None,
            suggestion_update_status: "PENDING".into(),
            suggestion_update_message: None,
            error: None,
            started_at: "before".into(),
            finished_at: None,
            baseline: MarketResearchBaseline {
                quote_id: "quote".into(),
                service_type: "video-editing".into(),
                configuration_json: configuration.into(),
                calculated_price_minor: Some(100_000),
                final_price_minor: final_price,
                has_override,
                currency: "USD".into(),
                market_scope: Some("international".into()),
            },
        }
    }

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
    fn print_design_context_uses_its_own_professional_parameters() {
        let json = r#"{"data":{"parameterValues":{"hasReference":true,"clientTier":"medium","productType":"shirt","complexity":"complex","printSystem":"dtf","workTasks":["adapt-composition"],"deliveryExtras":["ai-vector"],"estimatedHours":3.5,"projectType":"must-not-leak"}}}"#;
        let context = query_context("print-design", json, Some("both"));
        assert_eq!(context.service, "print-design");
        assert_eq!(context.subtype.as_deref(), Some("shirt"));
        assert_eq!(context.level.as_deref(), Some("complex"));
        assert_eq!(context.client_tier.as_deref(), Some("B"));
        assert_eq!(context.work_class.as_deref(), Some("adaptation"));
        assert_eq!(context.estimated_hours, Some(3.5));
        assert!(context.features.contains(&"printSystem".into()));
        assert!(context.features.contains(&"deliveryExtras".into()));
        assert!(!context.features.contains(&"projectType".into()));
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

    #[test]
    fn context_and_salary_sources_never_qualify_for_suggestions() {
        assert!(source_kind_supports_suggestions(
            "market_price",
            "professional_tariff"
        ));
        assert!(!source_kind_supports_suggestions(
            "context_only",
            "rate_benchmark"
        ));
        assert!(!source_kind_supports_suggestions("market_price", "salary"));
        assert!(!source_kind_supports_suggestions("currency", "currency"));
    }

    #[test]
    fn market_suggestion_preserves_configuration_final_price_and_revision() {
        tauri::async_runtime::block_on(async {
            let pool = suggestion_test_pool().await;
            let configuration = r#"{"data":{"estimatedHours":36}}"#;
            sqlx::query("INSERT INTO quote_services (id,suggested_subtotal_minor,final_subtotal_minor,has_override,row_revision,configuration_json,updated_at) VALUES ('service',110000,175000,1,7,?,'before')")
                .bind(configuration)
                .execute(&pool)
                .await
                .expect("service");
            let job = market_job(7, configuration, Some(175_000), true);
            let mut tx = pool.begin().await.expect("transaction");
            assert!(
                apply_market_suggestion_if_current(&mut tx, &job, 140_000, "after")
                    .await
                    .expect("conditional update")
            );
            tx.commit().await.expect("commit");
            let row: (String, Option<i64>, Option<i64>, bool, i64) = sqlx::query_as(
                "SELECT configuration_json,suggested_subtotal_minor,final_subtotal_minor,has_override,row_revision FROM quote_services WHERE id='service'",
            )
            .fetch_one(&pool)
            .await
            .expect("saved row");
            assert_eq!(row.0, configuration);
            assert_eq!(row.1, Some(140_000));
            assert_eq!(row.2, Some(175_000));
            assert!(row.3);
            assert_eq!(row.4, 7, "market evidence is not a user edit");
        });
    }

    #[test]
    fn market_suggestion_is_skipped_when_the_draft_changed() {
        tauri::async_runtime::block_on(async {
            let pool = suggestion_test_pool().await;
            let original = r#"{"data":{"estimatedHours":36}}"#;
            sqlx::query("INSERT INTO quote_services (id,suggested_subtotal_minor,final_subtotal_minor,has_override,row_revision,configuration_json,updated_at) VALUES ('service',110000,175000,1,7,?,'before')")
                .bind(original)
                .execute(&pool)
                .await
                .expect("service");
            sqlx::query("UPDATE quote_services SET configuration_json=?, final_subtotal_minor=220000, row_revision=8, updated_at='user-save' WHERE id='service'")
                .bind(r#"{"data":{"estimatedHours":48}}"#)
                .execute(&pool)
                .await
                .expect("user edit");
            let job = market_job(7, original, Some(175_000), true);
            let mut tx = pool.begin().await.expect("transaction");
            assert!(
                !apply_market_suggestion_if_current(&mut tx, &job, 140_000, "after")
                    .await
                    .expect("conditional update")
            );
            tx.commit().await.expect("commit");
            let row: (String, Option<i64>, Option<i64>, i64) = sqlx::query_as(
                "SELECT configuration_json,suggested_subtotal_minor,final_subtotal_minor,row_revision FROM quote_services WHERE id='service'",
            )
            .fetch_one(&pool)
            .await
            .expect("saved row");
            assert_eq!(row.0, r#"{"data":{"estimatedHours":48}}"#);
            assert_eq!(row.1, Some(110_000));
            assert_eq!(row.2, Some(220_000));
            assert_eq!(row.3, 8);
        });
    }

    #[test]
    fn catalog_migration_enables_verified_live_sources_and_never_restores_tarifario_url() {
        tauri::async_runtime::block_on(async {
            let options = SqliteConnectOptions::from_str("sqlite::memory:")
                .expect("valid sqlite options")
                .create_if_missing(true)
                .foreign_keys(true);
            let pool = SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(options)
                .await
                .expect("pool");
            sqlx::migrate!("./migrations")
                .run(&pool)
                .await
                .expect("migrations");
            let tarifario: (Option<String>, String, String, String, bool, String) = sqlx::query_as(
                "SELECT base_url,usage_mode,acquisition_mode,automation_status,participates_in_suggestions,default_data_json FROM market_sources WHERE system_key='tarifario'",
            )
            .fetch_one(&pool)
            .await
            .expect("tarifario");
            assert_eq!(tarifario.0, None);
            assert_eq!(tarifario.1, "context_only");
            assert_eq!(tarifario.2, "manual");
            assert_eq!(tarifario.3, "MANUAL_ONLY");
            assert!(!tarifario.4);
            assert!(tarifario.5.contains("\"baseUrl\":null"));

            let bcra: (String, String, String, bool, String) = sqlx::query_as(
                "SELECT usage_mode,acquisition_mode,automation_status,participates_in_suggestions,adapter_key FROM market_sources WHERE system_key='bcra'",
            )
            .fetch_one(&pool)
            .await
            .expect("bcra");
            assert_eq!(bcra.0, "currency");
            assert_eq!(bcra.1, "auto_http");
            assert_eq!(bcra.2, "APPROVED");
            assert!(!bcra.3);
            assert_eq!(bcra.4, "bcra");

            let automatic: Vec<(String, bool)> = sqlx::query_as(
                "SELECT system_key,participates_in_suggestions FROM market_sources WHERE acquisition_mode='auto_http' AND automation_status='APPROVED' ORDER BY system_key",
            )
            .fetch_all(&pool)
            .await
            .expect("automatic sources");
            assert_eq!(
                automatic,
                vec![
                    ("ardg-print-design".into(), true),
                    ("bcra".into(), false),
                    ("freelancerateiq-print-design".into(), true),
                    ("golance".into(), true),
                    ("indexdev".into(), true),
                    ("prolatam-programming-ar".into(), true),
                    ("prolatam-video-ar".into(), true),
                    ("reelrate".into(), true),
                    ("remoteok".into(), false),
                    ("solopricing".into(), true),
                    ("twine-print-design".into(), true),
                    ("upwork-print-design".into(), true),
                ]
            );
        });
    }
}
