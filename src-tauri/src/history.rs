use chrono::{SecondsFormat, Utc};
use serde_json::{json, Value};
use sqlx::{FromRow, Sqlite, SqlitePool, Transaction};
use tauri::State;
use uuid::Uuid;

use crate::{
    commands::workspace,
    db::AppState,
    error::{command_error, AppError, AppResult},
    models::{
        DuplicateQuoteInput, QuoteHistoryDetail, QuoteHistoryItem, QuoteService,
        QuoteSnapshotRevision, SaveQuoteSnapshotInput, UpdateQuoteAdminInput, Workspace,
    },
    phase6::assign_quote_number_in_transaction,
};

#[derive(Debug, FromRow)]
struct QuoteContext {
    quote_id: String,
    project_id: String,
    project_name: String,
    market_scope: Option<String>,
    client_id: String,
    client_name: String,
    client_company: Option<String>,
    currency: String,
    status: String,
    version: i64,
    notes: Option<String>,
}

#[derive(Debug, FromRow)]
struct SnapshotRow {
    revision: i64,
    reason: String,
    project_name: String,
    client_name: String,
    currency: String,
    selected_price_kind: String,
    selected_price_minor: Option<i64>,
    floor_total_minor: Option<i64>,
    recommended_total_minor: Option<i64>,
    premium_total_minor: Option<i64>,
    total_hours_micros: i64,
    external_costs_minor: i64,
    effective_hourly_minor: Option<i64>,
    margin_micros: Option<i64>,
    snapshot_json: String,
    created_at: String,
}

fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim().to_string();
        (!trimmed.is_empty()).then_some(trimmed)
    })
}

fn valid_status(status: &str) -> bool {
    matches!(
        status,
        "draft" | "sent" | "accepted" | "rejected" | "archived"
    )
}

fn valid_price_kind(kind: &str) -> bool {
    matches!(kind, "floor" | "recommended" | "premium" | "custom")
}

fn validate_money(value: Option<i64>, label: &str) -> AppResult<()> {
    if value.is_some_and(|value| value < 0) {
        Err(AppError::Validation(format!(
            "{label} no puede ser negativo."
        )))
    } else {
        Ok(())
    }
}

fn selected_price(
    kind: &str,
    custom: Option<i64>,
    floor: Option<i64>,
    recommended: Option<i64>,
    premium: Option<i64>,
) -> AppResult<Option<i64>> {
    if !valid_price_kind(kind) {
        return Err(AppError::Validation(
            "La opción de precio no es válida.".into(),
        ));
    }
    let selected = match kind {
        "floor" => floor,
        "recommended" => recommended,
        "premium" => premium,
        "custom" => Some(custom.ok_or_else(|| {
            AppError::Validation("Ingresá el importe del precio personalizado.".into())
        })?),
        _ => unreachable!(),
    };
    validate_money(selected, "El precio elegido")?;
    Ok(selected)
}

async fn quote_context<'a, E>(executor: E, quote_id: &str) -> AppResult<QuoteContext>
where
    E: sqlx::Executor<'a, Database = Sqlite>,
{
    sqlx::query_as::<_, QuoteContext>(
        "SELECT q.id AS quote_id, p.id AS project_id, p.name AS project_name,
                p.market_scope, c.id AS client_id, c.name AS client_name,
                c.company AS client_company, q.currency, q.status, q.version, q.notes
         FROM quotes q
         JOIN projects p ON p.id=q.project_id
         JOIN clients c ON c.id=p.client_id
         WHERE q.id=?",
    )
    .bind(quote_id)
    .fetch_optional(executor)
    .await?
    .ok_or(AppError::NotFound)
}

async fn canonical_services(
    tx: &mut Transaction<'_, Sqlite>,
    quote_id: &str,
) -> AppResult<Vec<QuoteService>> {
    Ok(sqlx::query_as::<_, QuoteService>(
        "SELECT id, quote_id, service_type, title, sort_order, configuration_version,
                configuration_json, calculated_subtotal_minor, suggested_subtotal_minor,
                final_subtotal_minor, has_override, manual_subtotal_minor, manual_reason,
                pricing_snapshot_json, service_definition_version, row_revision, deleted_at,
                created_at, updated_at
         FROM quote_services WHERE quote_id=? AND deleted_at IS NULL ORDER BY sort_order",
    )
    .bind(quote_id)
    .fetch_all(&mut **tx)
    .await?)
}

fn parse_json(raw: &str) -> Value {
    serde_json::from_str(raw).unwrap_or_else(|_| Value::String(raw.to_string()))
}

async fn source_references(
    tx: &mut Transaction<'_, Sqlite>,
    service: &QuoteService,
) -> AppResult<Value> {
    let assigned: Vec<String> = sqlx::query_scalar(
        "SELECT json_object(
            'id', ms.id, 'name', ms.name, 'url', ms.base_url,
            'sourceType', ms.business_source_type, 'country', ms.market_country,
            'currency', ms.source_currency, 'updatedAt', ms.source_updated_at,
            'contribution', ms.data_contribution, 'role', pes.role,
            'preference', pes.preference, 'assignmentUpdatedAt', pes.updated_at)
         FROM pricing_engine_sources pes
         JOIN pricing_engines pe ON pe.id=pes.engine_id
         JOIN market_sources ms ON ms.id=pes.source_id
         WHERE pe.engine_key=? AND pes.preference!='excluded'
         ORDER BY CASE pes.preference WHEN 'preferred' THEN 0 ELSE 1 END, ms.priority, ms.name",
    )
    .bind(&service.service_type)
    .fetch_all(&mut **tx)
    .await?;
    let market_snapshot: Option<String> = sqlx::query_scalar(
        "SELECT json_object(
            'id', id, 'createdAt', created_at, 'currency', currency,
            'medianMinor', market_median_minor, 'p25Minor', p25_minor,
            'p75Minor', p75_minor, 'confidence', confidence_level,
            'observationCount', observation_count, 'sourceCount', source_count)
         FROM market_snapshots WHERE quote_service_id=? ORDER BY created_at DESC LIMIT 1",
    )
    .bind(&service.id)
    .fetch_optional(&mut **tx)
    .await?;
    let observations: Vec<String> = if let Some(snapshot) = market_snapshot.as_ref() {
        let snapshot_id = parse_json(snapshot)
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        sqlx::query_scalar(
            "SELECT json_object(
                'sourceId', ms.id, 'sourceName', ms.name, 'sourceUrl', mo.source_url,
                'currency', mo.currency, 'valueMinor', mo.price_value_minor,
                'minimumMinor', mo.price_min_minor, 'maximumMinor', mo.price_max_minor,
                'unit', mo.unit, 'priceType', mo.price_type, 'publishedAt', mo.published_at,
                'retrievedAt', mo.retrieved_at, 'included', mso.included,
                'exclusionReason', mso.exclusion_reason)
             FROM market_snapshot_observations mso
             JOIN market_observations mo ON mo.id=mso.observation_id
             JOIN market_sources ms ON ms.id=mo.source_id
             WHERE mso.snapshot_id=? ORDER BY mso.included DESC, ms.name",
        )
        .bind(snapshot_id)
        .fetch_all(&mut **tx)
        .await?
    } else {
        Vec::new()
    };
    Ok(json!({
        "assigned": assigned.iter().map(|item| parse_json(item)).collect::<Vec<_>>(),
        "marketSnapshot": market_snapshot.as_deref().map(parse_json),
        "observations": observations.iter().map(|item| parse_json(item)).collect::<Vec<_>>()
    }))
}

async fn build_snapshot(
    tx: &mut Transaction<'_, Sqlite>,
    context: &QuoteContext,
    input: &SaveQuoteSnapshotInput,
    revision: i64,
    selected_minor: Option<i64>,
    timestamp: &str,
) -> AppResult<Value> {
    let services = canonical_services(tx, &context.quote_id).await?;
    let mut service_values = Vec::with_capacity(services.len());
    for service in services {
        let sources = source_references(tx, &service).await?;
        service_values.push(json!({
            "id": service.id,
            "serviceType": service.service_type,
            "title": service.title,
            "sortOrder": service.sort_order,
            "configurationVersion": service.configuration_version,
            "configuration": parse_json(&service.configuration_json),
            "calculatedSubtotalMinor": service.calculated_subtotal_minor,
            "suggestedSubtotalMinor": service.suggested_subtotal_minor,
            "finalSubtotalMinor": service.final_subtotal_minor,
            "hasOverride": service.has_override,
            "manualSubtotalMinor": service.manual_subtotal_minor,
            "manualReason": service.manual_reason,
            "serviceDefinitionVersion": service.service_definition_version,
            "pricingSnapshot": service.pricing_snapshot_json.as_deref().map(parse_json),
            "sources": sources
        }));
    }
    Ok(json!({
        "schemaVersion": 1,
        "savedAt": timestamp,
        "revision": revision,
        "quote": {
            "id": context.quote_id,
            "version": context.version,
            "status": context.status,
            "currency": context.currency,
            "notes": clean_optional(input.notes.clone()),
            "selectedPriceKind": input.selected_price_kind,
            "selectedPriceMinor": selected_minor
        },
        "project": {
            "id": context.project_id,
            "name": context.project_name,
            "marketScope": context.market_scope
        },
        "client": {
            "id": context.client_id,
            "name": context.client_name,
            "company": context.client_company
        },
        "services": service_values,
        "totals": {
            "floorMinor": input.floor_total_minor,
            "recommendedMinor": input.recommended_total_minor,
            "premiumMinor": input.premium_total_minor,
            "selectedMinor": selected_minor,
            "totalHoursMicros": input.total_hours_micros,
            "externalCostsMinor": input.external_costs_minor,
            "effectiveHourlyMinor": input.effective_hourly_minor,
            "marginMicros": input.margin_micros
        }
    }))
}

async fn history_item(pool: &SqlitePool, quote_id: &str) -> AppResult<QuoteHistoryItem> {
    sqlx::query_as::<_, QuoteHistoryItem>(
        "SELECT q.id, p.id AS project_id, p.name AS project_name, c.id AS client_id,
                c.name AS client_name, snap.currency, q.status, q.notes,
                q.selected_price_kind, q.selected_price_minor, q.floor_total_minor,
                q.recommended_total_minor, q.premium_total_minor, q.snapshot_revision,
                q.saved_at, q.updated_at, COUNT(snapshot_service.value) AS service_count,
                COALESCE(GROUP_CONCAT(json_extract(snapshot_service.value,'$.title'), ' · '), '') AS service_titles,
                COALESCE(GROUP_CONCAT(json_extract(snapshot_service.value,'$.serviceType'), '|'), '') AS service_types
         FROM quotes q
         JOIN projects p ON p.id=q.project_id
         JOIN clients c ON c.id=p.client_id
         JOIN quote_snapshots snap ON snap.quote_id=q.id AND snap.revision=q.snapshot_revision
         LEFT JOIN json_each(snap.snapshot_json,'$.services') snapshot_service
         WHERE q.id=? AND q.saved_at IS NOT NULL
         GROUP BY q.id, p.id, p.name, c.id, c.name, snap.currency, q.status, q.notes,
                  q.selected_price_kind, q.selected_price_minor, q.floor_total_minor,
                  q.recommended_total_minor, q.premium_total_minor, q.snapshot_revision,
                  q.saved_at, q.updated_at",
    )
    .bind(quote_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

pub(crate) async fn save_snapshot_in_pool(
    pool: &SqlitePool,
    mut input: SaveQuoteSnapshotInput,
) -> AppResult<QuoteHistoryDetail> {
    validate_money(input.floor_total_minor, "El piso")?;
    validate_money(input.recommended_total_minor, "El precio recomendado")?;
    validate_money(input.premium_total_minor, "El precio premium")?;
    validate_money(input.effective_hourly_minor, "El valor efectivo por hora")?;
    if input.total_hours_micros < 0 || input.external_costs_minor < 0 {
        return Err(AppError::Validation(
            "Las horas y los costos no pueden ser negativos.".into(),
        ));
    }
    let selected_minor = selected_price(
        &input.selected_price_kind,
        input.selected_price_minor,
        input.floor_total_minor,
        input.recommended_total_minor,
        input.premium_total_minor,
    )?;
    input.selected_price_minor = selected_minor;
    let reason = input.reason.as_deref().unwrap_or("manual_save");
    if !matches!(reason, "manual_save" | "calculation_update") {
        return Err(AppError::Validation(
            "El motivo del snapshot no es válido.".into(),
        ));
    }
    let mut tx = pool.begin().await?;
    let context = quote_context(&mut *tx, &input.quote_id).await?;
    if context.status == "archived" {
        return Err(AppError::Validation(
            "Restaurá la cotización antes de guardar una revisión.".into(),
        ));
    }
    let revision: i64 = sqlx::query_scalar("SELECT snapshot_revision + 1 FROM quotes WHERE id=?")
        .bind(&input.quote_id)
        .fetch_one(&mut *tx)
        .await?;
    let timestamp = now();
    let snapshot = build_snapshot(
        &mut tx,
        &context,
        &input,
        revision,
        selected_minor,
        &timestamp,
    )
    .await?;
    sqlx::query(
        "INSERT INTO quote_snapshots
         (id,quote_id,revision,schema_version,reason,project_name,client_name,currency,
          selected_price_kind,selected_price_minor,floor_total_minor,recommended_total_minor,
          premium_total_minor,total_hours_micros,external_costs_minor,effective_hourly_minor,
          margin_micros,snapshot_json,created_at)
         VALUES (?,?,?,1,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&input.quote_id)
    .bind(revision)
    .bind(reason)
    .bind(&context.project_name)
    .bind(&context.client_name)
    .bind(&context.currency)
    .bind(&input.selected_price_kind)
    .bind(selected_minor)
    .bind(input.floor_total_minor)
    .bind(input.recommended_total_minor)
    .bind(input.premium_total_minor)
    .bind(input.total_hours_micros)
    .bind(input.external_costs_minor)
    .bind(input.effective_hourly_minor)
    .bind(input.margin_micros)
    .bind(snapshot.to_string())
    .bind(&timestamp)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE quotes SET notes=?,selected_price_kind=?,selected_price_minor=?,
            floor_total_minor=?,recommended_total_minor=?,premium_total_minor=?,
            snapshot_revision=?,saved_at=?,updated_at=? WHERE id=?",
    )
    .bind(clean_optional(input.notes))
    .bind(&input.selected_price_kind)
    .bind(selected_minor)
    .bind(input.floor_total_minor)
    .bind(input.recommended_total_minor)
    .bind(input.premium_total_minor)
    .bind(revision)
    .bind(&timestamp)
    .bind(&timestamp)
    .bind(&input.quote_id)
    .execute(&mut *tx)
    .await?;
    assign_quote_number_in_transaction(&mut tx, &input.quote_id, &timestamp).await?;
    tx.commit().await?;
    get_detail_from_pool(pool, &input.quote_id, Some(revision)).await
}

async fn get_detail_from_pool(
    pool: &SqlitePool,
    quote_id: &str,
    revision: Option<i64>,
) -> AppResult<QuoteHistoryDetail> {
    let quote = history_item(pool, quote_id).await?;
    let snapshot = if let Some(revision) = revision {
        sqlx::query_as::<_, SnapshotRow>(
            "SELECT revision,reason,project_name,client_name,currency,selected_price_kind,
                    selected_price_minor,floor_total_minor,recommended_total_minor,
                    premium_total_minor,total_hours_micros,external_costs_minor,
                    effective_hourly_minor,margin_micros,snapshot_json,created_at
             FROM quote_snapshots WHERE quote_id=? AND revision=?",
        )
        .bind(quote_id)
        .bind(revision)
        .fetch_optional(pool)
        .await?
    } else {
        sqlx::query_as::<_, SnapshotRow>(
            "SELECT revision,reason,project_name,client_name,currency,selected_price_kind,
                    selected_price_minor,floor_total_minor,recommended_total_minor,
                    premium_total_minor,total_hours_micros,external_costs_minor,
                    effective_hourly_minor,margin_micros,snapshot_json,created_at
             FROM quote_snapshots WHERE quote_id=? ORDER BY revision DESC LIMIT 1",
        )
        .bind(quote_id)
        .fetch_optional(pool)
        .await?
    }
    .ok_or(AppError::NotFound)?;
    // Reading these typed columns is deliberate: SQLx validates the whole immutable
    // record even though the UI consumes the self-contained JSON representation.
    let _integrity = (
        &snapshot.reason,
        &snapshot.project_name,
        &snapshot.client_name,
        &snapshot.currency,
        &snapshot.selected_price_kind,
        snapshot.selected_price_minor,
        snapshot.floor_total_minor,
        snapshot.recommended_total_minor,
        snapshot.premium_total_minor,
        snapshot.total_hours_micros,
        snapshot.external_costs_minor,
        snapshot.effective_hourly_minor,
        snapshot.margin_micros,
    );
    let revisions = sqlx::query_as::<_, QuoteSnapshotRevision>(
        "SELECT revision,reason,created_at FROM quote_snapshots WHERE quote_id=? ORDER BY revision DESC",
    )
    .bind(quote_id)
    .fetch_all(pool)
    .await?;
    Ok(QuoteHistoryDetail {
        quote,
        snapshot_json: snapshot.snapshot_json,
        snapshot_created_at: snapshot.created_at,
        displayed_revision: snapshot.revision,
        revisions,
    })
}

#[tauri::command]
pub async fn list_quote_history(
    state: State<'_, AppState>,
) -> Result<Vec<QuoteHistoryItem>, String> {
    sqlx::query_as::<_, QuoteHistoryItem>(
        "SELECT q.id, p.id AS project_id, p.name AS project_name, c.id AS client_id,
                c.name AS client_name, snap.currency, q.status, q.notes,
                q.selected_price_kind, q.selected_price_minor, q.floor_total_minor,
                q.recommended_total_minor, q.premium_total_minor, q.snapshot_revision,
                q.saved_at, q.updated_at, COUNT(snapshot_service.value) AS service_count,
                COALESCE(GROUP_CONCAT(json_extract(snapshot_service.value,'$.title'), ' · '), '') AS service_titles,
                COALESCE(GROUP_CONCAT(json_extract(snapshot_service.value,'$.serviceType'), '|'), '') AS service_types
         FROM quotes q
         JOIN projects p ON p.id=q.project_id
         JOIN clients c ON c.id=p.client_id
         JOIN quote_snapshots snap ON snap.quote_id=q.id AND snap.revision=q.snapshot_revision
         LEFT JOIN json_each(snap.snapshot_json,'$.services') snapshot_service
         WHERE q.saved_at IS NOT NULL
         GROUP BY q.id, p.id, p.name, c.id, c.name, snap.currency, q.status, q.notes,
                  q.selected_price_kind, q.selected_price_minor, q.floor_total_minor,
                  q.recommended_total_minor, q.premium_total_minor, q.snapshot_revision,
                  q.saved_at, q.updated_at
         ORDER BY q.saved_at DESC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(AppError::from)
    .map_err(command_error)
}

#[tauri::command]
pub async fn get_quote_history(
    quote_id: String,
    revision: Option<i64>,
    state: State<'_, AppState>,
) -> Result<QuoteHistoryDetail, String> {
    get_detail_from_pool(&state.pool, &quote_id, revision)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn save_quote_snapshot(
    input: SaveQuoteSnapshotInput,
    state: State<'_, AppState>,
) -> Result<QuoteHistoryDetail, String> {
    save_snapshot_in_pool(&state.pool, input)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn update_quote_admin(
    input: UpdateQuoteAdminInput,
    state: State<'_, AppState>,
) -> Result<QuoteHistoryItem, String> {
    async {
        let name = input.project_name.trim();
        if name.is_empty() {
            return Err(AppError::Validation("El nombre del proyecto es obligatorio.".into()));
        }
        if !valid_status(&input.status) {
            return Err(AppError::Validation("El estado no es válido.".into()));
        }
        let mut tx = state.pool.begin().await?;
        let context = quote_context(&mut *tx, &input.quote_id).await?;
        let active_client: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM clients WHERE id=? AND status='active'")
                .bind(&input.client_id)
                .fetch_one(&mut *tx)
                .await?;
        if active_client == 0 {
            return Err(AppError::Validation("Seleccioná un cliente activo.".into()));
        }
        let totals: (Option<i64>, Option<i64>, Option<i64>) = sqlx::query_as(
            "SELECT floor_total_minor,recommended_total_minor,premium_total_minor FROM quotes WHERE id=?",
        )
        .bind(&input.quote_id)
        .fetch_one(&mut *tx)
        .await?;
        let selected = selected_price(
            &input.selected_price_kind,
            input.selected_price_minor,
            totals.0,
            totals.1,
            totals.2,
        )?;
        let timestamp = now();
        sqlx::query("UPDATE projects SET name=?,client_id=?,updated_at=? WHERE id=?")
            .bind(name)
            .bind(&input.client_id)
            .bind(&timestamp)
            .bind(&context.project_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "UPDATE quotes SET notes=?,status=?,selected_price_kind=?,selected_price_minor=?,
             archived_at=?,updated_at=? WHERE id=?",
        )
        .bind(clean_optional(input.notes))
        .bind(&input.status)
        .bind(&input.selected_price_kind)
        .bind(selected)
        .bind((input.status == "archived").then_some(timestamp.clone()))
        .bind(&timestamp)
        .bind(&input.quote_id)
        .execute(&mut *tx)
        .await?;
        if input.status == "archived" {
            sqlx::query("UPDATE app_settings SET active_project_id=NULL,updated_at=? WHERE active_project_id=?")
                .bind(&timestamp)
                .bind(&context.project_id)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        history_item(&state.pool, &input.quote_id).await
    }
    .await
    .map_err(command_error)
}

pub(crate) async fn duplicate_quote_in_pool(
    pool: &SqlitePool,
    input: DuplicateQuoteInput,
) -> AppResult<String> {
    let mut tx = pool.begin().await?;
    let context = quote_context(&mut *tx, &input.quote_id).await?;
    let original = if let Some(revision) = input.revision {
        sqlx::query_as::<_, SnapshotRow>(
            "SELECT revision,reason,project_name,client_name,currency,selected_price_kind,
                    selected_price_minor,floor_total_minor,recommended_total_minor,
                    premium_total_minor,total_hours_micros,external_costs_minor,
                    effective_hourly_minor,margin_micros,snapshot_json,created_at
             FROM quote_snapshots WHERE quote_id=? AND revision=?",
        )
        .bind(&input.quote_id)
        .bind(revision)
        .fetch_optional(&mut *tx)
        .await?
    } else {
        sqlx::query_as::<_, SnapshotRow>(
            "SELECT revision,reason,project_name,client_name,currency,selected_price_kind,
                    selected_price_minor,floor_total_minor,recommended_total_minor,
                    premium_total_minor,total_hours_micros,external_costs_minor,
                    effective_hourly_minor,margin_micros,snapshot_json,created_at
             FROM quote_snapshots WHERE quote_id=? ORDER BY revision DESC LIMIT 1",
        )
        .bind(&input.quote_id)
        .fetch_optional(&mut *tx)
        .await?
    }
    .ok_or_else(|| {
        AppError::Validation("Guardá la cotización antes de usarla como base.".into())
    })?;
    let client_id = input.client_id.unwrap_or_else(|| context.client_id.clone());
    let client: (String, Option<String>) =
        sqlx::query_as("SELECT name,company FROM clients WHERE id=? AND status='active'")
            .bind(&client_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| AppError::Validation("Seleccioná un cliente activo.".into()))?;
    let project_name = input
        .project_name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("Copia de {}", context.project_name));
    let project_id = Uuid::new_v4().to_string();
    let quote_id = Uuid::new_v4().to_string();
    let timestamp = now();
    let mut snapshot = parse_json(&original.snapshot_json);
    let snapshot_services = snapshot
        .get("services")
        .and_then(Value::as_array)
        .cloned()
        .ok_or_else(|| {
            AppError::Validation("El snapshot no contiene módulos reutilizables.".into())
        })?;
    sqlx::query(
            "INSERT INTO projects (id,client_id,name,currency,market_scope,status,created_at,updated_at)
             VALUES (?,?,?,?,?,'active',?,?)",
        )
        .bind(&project_id)
        .bind(&client_id)
        .bind(&project_name)
        .bind(&original.currency)
        .bind(&context.market_scope)
        .bind(&timestamp)
        .bind(&timestamp)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "INSERT INTO quotes
             (id,project_id,version,status,currency,notes,selected_price_kind,selected_price_minor,
              floor_total_minor,recommended_total_minor,premium_total_minor,snapshot_revision,
              saved_at,created_at,updated_at)
             VALUES (?,?,1,'draft',?,?,?,?,?,?,?,1,?,?,?)",
    )
    .bind(&quote_id)
    .bind(&project_id)
    .bind(&original.currency)
    .bind(&context.notes)
    .bind(&original.selected_price_kind)
    .bind(original.selected_price_minor)
    .bind(original.floor_total_minor)
    .bind(original.recommended_total_minor)
    .bind(original.premium_total_minor)
    .bind(&timestamp)
    .bind(&timestamp)
    .bind(&timestamp)
    .execute(&mut *tx)
    .await?;
    let mut service_ids = std::collections::HashMap::new();
    for service in snapshot_services {
        let service = service.as_object().ok_or_else(|| {
            AppError::Validation("Un módulo del snapshot histórico no es válido.".into())
        })?;
        let old_id = service.get("id").and_then(Value::as_str).ok_or_else(|| {
            AppError::Validation("Un módulo histórico no tiene identificador.".into())
        })?;
        let service_type = service
            .get("serviceType")
            .and_then(Value::as_str)
            .ok_or_else(|| AppError::Validation("Un módulo histórico no tiene tipo.".into()))?;
        let title = service
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or(service_type);
        let configuration =
            serde_json::to_string(service.get("configuration").unwrap_or(&Value::Null))?;
        let pricing_snapshot = service
            .get("pricingSnapshot")
            .filter(|value| !value.is_null())
            .map(serde_json::to_string)
            .transpose()?;
        let new_id = Uuid::new_v4().to_string();
        service_ids.insert(old_id.to_string(), new_id.clone());
        sqlx::query(
                "INSERT INTO quote_services
                 (id,quote_id,service_type,title,sort_order,configuration_version,configuration_json,
                  calculated_subtotal_minor,suggested_subtotal_minor,final_subtotal_minor,has_override,
                  manual_subtotal_minor,manual_reason,pricing_snapshot_json,service_definition_version,
                  row_revision,created_at,updated_at)
                 VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,0,?,?)",
            )
            .bind(new_id)
            .bind(&quote_id)
            .bind(service_type)
            .bind(title)
            .bind(service.get("sortOrder").and_then(Value::as_i64).unwrap_or(0))
            .bind(service.get("configurationVersion").and_then(Value::as_i64).unwrap_or(1))
            .bind(configuration)
            .bind(service.get("calculatedSubtotalMinor").and_then(Value::as_i64))
            .bind(service.get("suggestedSubtotalMinor").and_then(Value::as_i64))
            .bind(service.get("finalSubtotalMinor").and_then(Value::as_i64))
            .bind(service.get("hasOverride").and_then(Value::as_bool).unwrap_or(false))
            .bind(service.get("manualSubtotalMinor").and_then(Value::as_i64))
            .bind(service.get("manualReason").and_then(Value::as_str))
            .bind(pricing_snapshot)
            .bind(service.get("serviceDefinitionVersion").and_then(Value::as_i64))
            .bind(&timestamp)
            .bind(&timestamp)
            .execute(&mut *tx)
            .await?;
    }
    if let Some(root) = snapshot.as_object_mut() {
        root.insert("savedAt".into(), json!(timestamp));
        root.insert("revision".into(), json!(1));
        root.insert(
            "duplicatedFrom".into(),
            json!({"quoteId": input.quote_id, "revision": original.revision}),
        );
        if let Some(quote) = root.get_mut("quote").and_then(Value::as_object_mut) {
            quote.insert("id".into(), json!(quote_id));
            quote.insert("status".into(), json!("draft"));
        }
        if let Some(project) = root.get_mut("project").and_then(Value::as_object_mut) {
            project.insert("id".into(), json!(project_id));
            project.insert("name".into(), json!(project_name));
        }
        if let Some(client_value) = root.get_mut("client").and_then(Value::as_object_mut) {
            client_value.insert("id".into(), json!(client_id));
            client_value.insert("name".into(), json!(client.0));
            client_value.insert("company".into(), json!(client.1));
        }
        if let Some(services) = root.get_mut("services").and_then(Value::as_array_mut) {
            for service in services {
                if let Some(item) = service.as_object_mut() {
                    if let Some(old_id) = item.get("id").and_then(Value::as_str) {
                        if let Some(new_id) = service_ids.get(old_id) {
                            item.insert("id".into(), json!(new_id));
                        }
                    }
                }
            }
        }
    }
    sqlx::query(
        "INSERT INTO quote_snapshots
             (id,quote_id,revision,schema_version,reason,project_name,client_name,currency,
              selected_price_kind,selected_price_minor,floor_total_minor,recommended_total_minor,
              premium_total_minor,total_hours_micros,external_costs_minor,effective_hourly_minor,
              margin_micros,snapshot_json,created_at)
             VALUES (?,?,1,1,'duplicate',?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&quote_id)
    .bind(&project_name)
    .bind(&client.0)
    .bind(&original.currency)
    .bind(&original.selected_price_kind)
    .bind(original.selected_price_minor)
    .bind(original.floor_total_minor)
    .bind(original.recommended_total_minor)
    .bind(original.premium_total_minor)
    .bind(original.total_hours_micros)
    .bind(original.external_costs_minor)
    .bind(original.effective_hourly_minor)
    .bind(original.margin_micros)
    .bind(snapshot.to_string())
    .bind(&timestamp)
    .execute(&mut *tx)
    .await?;
    assign_quote_number_in_transaction(&mut tx, &quote_id, &timestamp).await?;
    sqlx::query("UPDATE app_settings SET active_project_id=?,updated_at=? WHERE id=1")
        .bind(&project_id)
        .bind(&timestamp)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(project_id)
}

#[tauri::command]
pub async fn duplicate_quote(
    input: DuplicateQuoteInput,
    state: State<'_, AppState>,
) -> Result<Workspace, String> {
    async {
        let project_id = duplicate_quote_in_pool(&state.pool, input).await?;
        workspace(&state.pool, &project_id).await
    }
    .await
    .map_err(command_error)
}

#[tauri::command]
pub async fn delete_quote_permanently(
    quote_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    async {
        let mut tx = state.pool.begin().await?;
        let context = quote_context(&mut *tx, &quote_id).await?;
        if context.status != "archived" {
            return Err(AppError::Validation(
                "Archivá la cotización antes de eliminarla definitivamente.".into(),
            ));
        }
        sqlx::query("DELETE FROM quote_services WHERE quote_id=?")
            .bind(&quote_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM quotes WHERE id=?")
            .bind(&quote_id)
            .execute(&mut *tx)
            .await?;
        let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM quotes WHERE project_id=?")
            .bind(&context.project_id)
            .fetch_one(&mut *tx)
            .await?;
        if remaining == 0 {
            sqlx::query("UPDATE app_settings SET active_project_id=NULL WHERE active_project_id=?")
                .bind(&context.project_id)
                .execute(&mut *tx)
                .await?;
            sqlx::query("DELETE FROM projects WHERE id=?")
                .bind(&context.project_id)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }
    .await
    .map_err(command_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::str::FromStr;

    async fn seed_pool(pool: SqlitePool) -> SqlitePool {
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrations");
        sqlx::query("INSERT INTO clients (id,name,status,created_at,updated_at) VALUES ('client-test','Cliente','active','now','now')")
            .execute(&pool).await.expect("client");
        sqlx::query("INSERT INTO projects (id,client_id,name,currency,market_scope,status,created_at,updated_at) VALUES ('project-test','client-test','Proyecto','USD','international','active','now','now')")
            .execute(&pool).await.expect("project");
        sqlx::query("INSERT INTO quotes (id,project_id,version,status,currency,created_at,updated_at) VALUES ('quote-test','project-test',1,'draft','USD','now','now')")
            .execute(&pool).await.expect("quote");
        sqlx::query("INSERT INTO quote_services (id,quote_id,service_type,title,sort_order,configuration_version,configuration_json,calculated_subtotal_minor,suggested_subtotal_minor,final_subtotal_minor,has_override,row_revision,created_at,updated_at) VALUES ('service-test','quote-test','video-editing','Video',0,1,'{}',90000,120000,120000,0,0,'now','now')")
            .execute(&pool).await.expect("service");
        pool
    }

    async fn seeded_pool() -> SqlitePool {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .expect("options")
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("pool");
        seed_pool(pool).await
    }

    fn input(recommended: i64) -> SaveQuoteSnapshotInput {
        SaveQuoteSnapshotInput {
            quote_id: "quote-test".into(),
            notes: Some("Alcance original".into()),
            selected_price_kind: "recommended".into(),
            selected_price_minor: Some(recommended),
            floor_total_minor: Some(90_000),
            recommended_total_minor: Some(recommended),
            premium_total_minor: Some(145_000),
            total_hours_micros: 10_000_000,
            external_costs_minor: 5_000,
            effective_hourly_minor: Some(11_500),
            margin_micros: Some(250_000),
            reason: None,
        }
    }

    #[test]
    fn snapshots_are_append_only_and_keep_old_prices() {
        tauri::async_runtime::block_on(async {
            let pool = seeded_pool().await;
            save_snapshot_in_pool(&pool, input(120_000))
                .await
                .expect("first");
            sqlx::query("UPDATE quote_services SET suggested_subtotal_minor=180000,final_subtotal_minor=180000")
                .execute(&pool).await.expect("new engine values");
            let mut second = input(180_000);
            second.reason = Some("calculation_update".into());
            save_snapshot_in_pool(&pool, second).await.expect("second");
            let values: Vec<(i64, i64)> = sqlx::query_as(
                "SELECT revision,recommended_total_minor FROM quote_snapshots WHERE quote_id='quote-test' ORDER BY revision",
            )
            .fetch_all(&pool).await.expect("snapshots");
            assert_eq!(values, vec![(1, 120_000), (2, 180_000)]);
            let first = get_detail_from_pool(&pool, "quote-test", Some(1))
                .await
                .expect("old detail");
            assert!(first.snapshot_json.contains("120000"));
            let tamper = sqlx::query(
                "UPDATE quote_snapshots SET recommended_total_minor=1 WHERE quote_id='quote-test' AND revision=1",
            )
            .execute(&pool)
            .await;
            assert!(tamper.is_err(), "un snapshot existente nunca se modifica");
        });
    }

    #[test]
    fn duplicate_keeps_values_and_creates_an_independent_draft() {
        tauri::async_runtime::block_on(async {
            let pool = seeded_pool().await;
            save_snapshot_in_pool(&pool, input(120_000))
                .await
                .expect("snapshot");
            sqlx::query("UPDATE quote_services SET final_subtotal_minor=999999,configuration_json='{\"changedAfterSnapshot\":true}' WHERE id='service-test'")
                .execute(&pool)
                .await
                .expect("live draft changed after snapshot");
            let project_id = duplicate_quote_in_pool(
                &pool,
                DuplicateQuoteInput {
                    quote_id: "quote-test".into(),
                    project_name: None,
                    client_id: None,
                    revision: Some(1),
                },
            )
            .await
            .expect("duplicate");
            let duplicate: (String, String, Option<i64>, i64) = sqlx::query_as(
                "SELECT q.id,q.status,q.selected_price_minor,q.snapshot_revision
                 FROM quotes q WHERE q.project_id=?",
            )
            .bind(project_id)
            .fetch_one(&pool)
            .await
            .expect("duplicated quote");
            assert_ne!(duplicate.0, "quote-test");
            assert_eq!(duplicate.1, "draft");
            assert_eq!(duplicate.2, Some(120_000));
            assert_eq!(duplicate.3, 1);
            let copied_service: (String, Option<i64>, String) = sqlx::query_as(
                "SELECT id,final_subtotal_minor,configuration_json FROM quote_services WHERE quote_id=?",
            )
            .bind(&duplicate.0)
            .fetch_one(&pool)
            .await
            .expect("copied service");
            assert_ne!(copied_service.0, "service-test");
            assert_eq!(copied_service.1, Some(120_000));
            assert_eq!(copied_service.2, "{}");
            sqlx::query("UPDATE quote_services SET final_subtotal_minor=777777 WHERE quote_id=?")
                .bind(&duplicate.0)
                .execute(&pool)
                .await
                .expect("edit copy");
            let original: Option<i64> = sqlx::query_scalar(
                "SELECT final_subtotal_minor FROM quote_services WHERE id='service-test'",
            )
            .fetch_one(&pool)
            .await
            .expect("original value");
            assert_eq!(original, Some(999_999));
        });
    }

    #[test]
    fn saved_quote_history_persists_after_reopening_database() {
        tauri::async_runtime::block_on(async {
            let path =
                std::env::temp_dir().join(format!("pricing-os-history-{}.sqlite3", Uuid::new_v4()));
            let connection = format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"));
            let options = SqliteConnectOptions::from_str(&connection)
                .expect("options")
                .create_if_missing(true)
                .foreign_keys(true);
            let pool = SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(options.clone())
                .await
                .expect("first connection");
            let pool = seed_pool(pool).await;
            save_snapshot_in_pool(&pool, input(120_000))
                .await
                .expect("saved snapshot");
            pool.close().await;
            // On Windows a closed SqlitePool can retain its file handle until
            // the pool value is dropped. Release it before reopening/removing
            // the temporary database so the persistence test is deterministic.
            drop(pool);

            let reopened = SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(options)
                .await
                .expect("second connection");
            let detail = get_detail_from_pool(&reopened, "quote-test", None)
                .await
                .expect("persisted snapshot");
            assert_eq!(detail.quote.snapshot_revision, 1);
            assert_eq!(detail.quote.recommended_total_minor, Some(120_000));
            assert!(detail.snapshot_json.contains("Alcance original"));
            reopened.close().await;
            drop(reopened);
            let mut remove_error = None;
            for attempt in 0..5 {
                match std::fs::remove_file(&path) {
                    Ok(()) => {
                        remove_error = None;
                        break;
                    }
                    Err(error) => {
                        remove_error = Some(error);
                        if attempt < 4 {
                            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
                        }
                    }
                }
            }
            if let Some(error) = remove_error {
                panic!("remove test database: {error}");
            }
        });
    }
}
