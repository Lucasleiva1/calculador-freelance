use chrono::{SecondsFormat, Utc};
use serde_json::Value;
use sqlx::{Sqlite, SqlitePool, Transaction};
use tauri::State;
use uuid::Uuid;

use crate::{
    db::AppState,
    error::{command_error, AppError, AppResult},
    models::{
        AppSettings, Bootstrap, Client, ClientInput, CreateProjectInput, EconomicProfile,
        EconomicProfileInput, MarketSource, MarketSourceInput, ParameterOption,
        ParameterOptionInput, Preset, PresetInput, PricingConfiguration, PricingRule,
        PricingRuleInput, ProjectSummary, Quote, QuoteService, SaveServiceInput, ServiceDefinition,
        ServiceDefinitionInput, ServiceParameter, ServiceParameterInput, SettingsInput, Workspace,
    },
};

fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim().to_string();
        (!trimmed.is_empty()).then_some(trimmed)
    })
}

fn validate_currency(currency: &str) -> AppResult<()> {
    if matches!(currency, "ARS" | "USD") {
        Ok(())
    } else {
        Err(AppError::Validation("Moneda inválida.".into()))
    }
}

fn validate_non_negative(value: Option<i64>, field: &str) -> AppResult<()> {
    if value.is_some_and(|value| value < 0) {
        Err(AppError::Validation(format!(
            "{field} no puede ser negativo."
        )))
    } else {
        Ok(())
    }
}

async fn clients(pool: &SqlitePool) -> AppResult<Vec<Client>> {
    Ok(sqlx::query_as::<_, Client>(
        "SELECT id, name, company, email, whatsapp, country, notes, status, created_at, updated_at
         FROM clients ORDER BY status ASC, name COLLATE NOCASE ASC",
    )
    .fetch_all(pool)
    .await?)
}

async fn projects(pool: &SqlitePool) -> AppResult<Vec<ProjectSummary>> {
    Ok(sqlx::query_as::<_, ProjectSummary>(
        "SELECT p.id, p.client_id, c.name AS client_name, p.name, p.currency, p.market_scope,
                p.status,
                CASE WHEN COUNT(qs.id) = 0 OR SUM(CASE WHEN COALESCE(qs.final_subtotal_minor, qs.manual_subtotal_minor, qs.calculated_subtotal_minor) IS NOT NULL THEN 1 ELSE 0 END) = 0 THEN NULL
                     ELSE SUM(COALESCE(qs.final_subtotal_minor, qs.manual_subtotal_minor, qs.calculated_subtotal_minor, 0)) END AS total_minor,
                SUM(CASE WHEN qs.id IS NOT NULL AND qs.final_subtotal_minor IS NULL
                              AND qs.manual_subtotal_minor IS NULL AND qs.calculated_subtotal_minor IS NULL THEN 1 ELSE 0 END) AS unpriced_count,
                p.updated_at
         FROM projects p
         JOIN clients c ON c.id = p.client_id
         LEFT JOIN quotes q ON q.project_id = p.id
            AND q.version = (SELECT MAX(q2.version) FROM quotes q2 WHERE q2.project_id = p.id)
         LEFT JOIN quote_services qs ON qs.quote_id = q.id AND qs.deleted_at IS NULL
         GROUP BY p.id, p.client_id, c.name, p.name, p.currency, p.market_scope, p.status, p.updated_at
         ORDER BY CASE p.status WHEN 'active' THEN 0 ELSE 1 END, p.updated_at DESC",
    )
    .fetch_all(pool)
    .await?)
}

async fn settings(pool: &SqlitePool) -> AppResult<AppSettings> {
    Ok(sqlx::query_as::<_, AppSettings>(
        "SELECT theme, hourly_rate_ars_minor, hourly_rate_usd_minor, usd_to_ars_micros,
                active_project_id, suggestions_enabled, suggestion_strategy, base_currency,
                updated_at FROM app_settings WHERE id = 1",
    )
    .fetch_one(pool)
    .await?)
}

async fn presets(pool: &SqlitePool) -> AppResult<Vec<Preset>> {
    Ok(sqlx::query_as::<_, Preset>(
        "SELECT id, service_type, name, origin, system_key, configuration_version,
                definition_version, configuration_json, created_at, updated_at
         FROM service_presets ORDER BY CASE origin WHEN 'system' THEN 0 ELSE 1 END, name COLLATE NOCASE",
    )
    .fetch_all(pool)
    .await?)
}

async fn pricing_configuration(pool: &SqlitePool) -> AppResult<PricingConfiguration> {
    let definitions = sqlx::query_as::<_, ServiceDefinition>(
        "SELECT id, service_type, name, description, version, enabled, suggestions_enabled,
                default_strategy, competitive_margin_micros, balanced_margin_micros,
                premium_margin_micros, created_at, updated_at
         FROM service_definitions ORDER BY name COLLATE NOCASE",
    )
    .fetch_all(pool)
    .await?;
    let parameters = sqlx::query_as::<_, ServiceParameter>(
        "SELECT id, service_definition_id, parameter_key, name, label, parameter_type,
                description, required, sort_order, enabled, default_value_json,
                suggestion_enabled, is_system, ui_managed, version, created_at, updated_at
         FROM service_parameters ORDER BY service_definition_id, sort_order, name COLLATE NOCASE",
    )
    .fetch_all(pool)
    .await?;
    let options = sqlx::query_as::<_, ParameterOption>(
        "SELECT id, parameter_id, label, value, sort_order, enabled, created_at, updated_at
         FROM parameter_options ORDER BY parameter_id, sort_order, label COLLATE NOCASE",
    )
    .fetch_all(pool)
    .await?;
    let rules = sqlx::query_as::<_, PricingRule>(
        "SELECT id, service_definition_id, parameter_id, option_id, quantity_parameter_id,
                name, rule_type, numeric_value_micros, amount_ars_minor, amount_usd_minor,
                sort_order, enabled, version, created_at, updated_at
         FROM pricing_rules ORDER BY service_definition_id, sort_order, name COLLATE NOCASE",
    )
    .fetch_all(pool)
    .await?;
    let economic_profiles = sqlx::query_as::<_, EconomicProfile>(
        "SELECT currency, monthly_income_target_minor, monthly_expenses_minor,
                billable_hours_micros, reserve_tax_micros, desired_margin_micros,
                default_urgency_micros, work_days, vacation_weeks, manual_hourly_rate_minor,
                updated_at FROM economic_profiles ORDER BY currency",
    )
    .fetch_all(pool)
    .await?;
    let market_sources = sqlx::query_as::<_, MarketSource>(
        "SELECT id, name, base_url, source_type, regions_json, supported_services_json,
                priority, enabled, usage_mode, acquisition_mode, cooldown_hours, notes,
                is_system_source, system_key, default_data_json, created_at, updated_at
         FROM market_sources ORDER BY priority, name COLLATE NOCASE",
    )
    .fetch_all(pool)
    .await?;
    Ok(PricingConfiguration {
        definitions,
        parameters,
        options,
        rules,
        economic_profiles,
        market_sources,
    })
}

async fn project_by_id(pool: &SqlitePool, project_id: &str) -> AppResult<ProjectSummary> {
    projects(pool)
        .await?
        .into_iter()
        .find(|project| project.id == project_id)
        .ok_or(AppError::NotFound)
}

async fn workspace(pool: &SqlitePool, project_id: &str) -> AppResult<Workspace> {
    let project = project_by_id(pool, project_id).await?;
    let quote = sqlx::query_as::<_, Quote>(
        "SELECT id, project_id, version, status, currency, created_at, updated_at
         FROM quotes WHERE project_id = ? ORDER BY version DESC LIMIT 1",
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;
    let services = sqlx::query_as::<_, QuoteService>(
        "SELECT id, quote_id, service_type, title, sort_order, configuration_version,
                configuration_json, calculated_subtotal_minor, suggested_subtotal_minor,
                final_subtotal_minor, has_override, manual_subtotal_minor,
                manual_reason, pricing_snapshot_json, service_definition_version,
                row_revision, deleted_at, created_at, updated_at
         FROM quote_services WHERE quote_id = ? AND deleted_at IS NULL ORDER BY sort_order ASC",
    )
    .bind(&quote.id)
    .fetch_all(pool)
    .await?;
    Ok(Workspace {
        project,
        quote,
        services,
    })
}

async fn insert_client(tx: &mut Transaction<'_, Sqlite>, input: ClientInput) -> AppResult<String> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(AppError::Validation(
            "El nombre del cliente es obligatorio.".into(),
        ));
    }
    let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let timestamp = now();
    sqlx::query(
        "INSERT INTO clients (id, name, company, email, whatsapp, country, notes, status, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, 'active', ?, ?)",
    )
    .bind(&id)
    .bind(name)
    .bind(clean_optional(input.company))
    .bind(clean_optional(input.email))
    .bind(clean_optional(input.whatsapp))
    .bind(clean_optional(input.country))
    .bind(clean_optional(input.notes))
    .bind(&timestamp)
    .bind(&timestamp)
    .execute(&mut **tx)
    .await?;
    Ok(id)
}

#[tauri::command]
pub async fn bootstrap_app(state: State<'_, AppState>) -> Result<Bootstrap, String> {
    async {
        Ok(Bootstrap {
            clients: clients(&state.pool).await?,
            projects: projects(&state.pool).await?,
            presets: presets(&state.pool).await?,
            settings: settings(&state.pool).await?,
            pricing: pricing_configuration(&state.pool).await?,
        })
    }
    .await
    .map_err(command_error)
}

#[tauri::command]
pub async fn load_workspace(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Workspace, String> {
    async {
        let result = workspace(&state.pool, &project_id).await?;
        sqlx::query("UPDATE app_settings SET active_project_id = ?, updated_at = ? WHERE id = 1")
            .bind(&project_id)
            .bind(now())
            .execute(&state.pool)
            .await?;
        Ok(result)
    }
    .await
    .map_err(command_error)
}

#[tauri::command]
pub async fn create_project(
    input: CreateProjectInput,
    state: State<'_, AppState>,
) -> Result<Workspace, String> {
    async {
        let project_name = input.name.trim();
        if project_name.is_empty() {
            return Err(AppError::Validation("El nombre del proyecto es obligatorio.".into()));
        }
        validate_currency(&input.currency)?;
        if !matches!(input.market_scope.as_str(), "argentina" | "international" | "both") {
            return Err(AppError::Validation("Mercado de referencia inválido.".into()));
        }
        let mut tx = state.pool.begin().await?;
        let client_id = match (input.client_id, input.new_client) {
            (Some(client_id), None) => {
                let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clients WHERE id = ? AND status = 'active'")
                    .bind(&client_id)
                    .fetch_one(&mut *tx)
                    .await?;
                if exists == 0 { return Err(AppError::Validation("Seleccioná un cliente activo.".into())); }
                client_id
            }
            (None, Some(client)) => insert_client(&mut tx, client).await?,
            _ => return Err(AppError::Validation("Seleccioná o creá un cliente.".into())),
        };
        let project_id = Uuid::new_v4().to_string();
        let quote_id = Uuid::new_v4().to_string();
        let timestamp = now();
        sqlx::query(
            "INSERT INTO projects (id, client_id, name, currency, market_scope, status, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, 'active', ?, ?)",
        )
        .bind(&project_id).bind(&client_id).bind(project_name).bind(&input.currency)
        .bind(&input.market_scope).bind(&timestamp).bind(&timestamp)
        .execute(&mut *tx).await?;
        sqlx::query(
            "INSERT INTO quotes (id, project_id, version, status, currency, created_at, updated_at)
             VALUES (?, ?, 1, 'draft', ?, ?, ?)",
        )
        .bind(&quote_id).bind(&project_id).bind(&input.currency).bind(&timestamp).bind(&timestamp)
        .execute(&mut *tx).await?;
        sqlx::query("UPDATE app_settings SET active_project_id = ?, updated_at = ? WHERE id = 1")
            .bind(&project_id).bind(&timestamp).execute(&mut *tx).await?;
        tx.commit().await?;
        workspace(&state.pool, &project_id).await
    }
    .await
    .map_err(command_error)
}

#[tauri::command]
pub async fn save_client(input: ClientInput, state: State<'_, AppState>) -> Result<Client, String> {
    async {
        let name = input.name.trim();
        if name.is_empty() { return Err(AppError::Validation("El nombre del cliente es obligatorio.".into())); }
        let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let timestamp = now();
        sqlx::query(
            "INSERT INTO clients (id, name, company, email, whatsapp, country, notes, status, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, 'active', ?, ?)
             ON CONFLICT(id) DO UPDATE SET name=excluded.name, company=excluded.company,
               email=excluded.email, whatsapp=excluded.whatsapp, country=excluded.country,
               notes=excluded.notes, updated_at=excluded.updated_at",
        )
        .bind(&id).bind(name).bind(clean_optional(input.company)).bind(clean_optional(input.email))
        .bind(clean_optional(input.whatsapp)).bind(clean_optional(input.country)).bind(clean_optional(input.notes))
        .bind(&timestamp).bind(&timestamp).execute(&state.pool).await?;
        sqlx::query_as::<_, Client>(
            "SELECT id, name, company, email, whatsapp, country, notes, status, created_at, updated_at FROM clients WHERE id = ?",
        ).bind(&id).fetch_one(&state.pool).await.map_err(AppError::from)
    }.await.map_err(command_error)
}

#[tauri::command]
pub async fn set_client_archived(
    id: String,
    archived: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    async {
        let result = sqlx::query("UPDATE clients SET status = ?, updated_at = ? WHERE id = ?")
            .bind(if archived { "archived" } else { "active" })
            .bind(now())
            .bind(id)
            .execute(&state.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }
    .await
    .map_err(command_error)
}

#[tauri::command]
pub async fn set_project_archived(
    id: String,
    archived: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    async {
        let mut tx = state.pool.begin().await?;
        let result = sqlx::query("UPDATE projects SET status = ?, updated_at = ? WHERE id = ?")
            .bind(if archived { "archived" } else { "active" }).bind(now()).bind(&id)
            .execute(&mut *tx).await?;
        if result.rows_affected() == 0 { return Err(AppError::NotFound); }
        if archived {
            sqlx::query("UPDATE app_settings SET active_project_id = NULL, updated_at = ? WHERE active_project_id = ?")
                .bind(now()).bind(&id).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }.await.map_err(command_error)
}

fn default_configuration(service_type: &str) -> AppResult<(String, String)> {
    let (title, value) = match service_type {
        "video-editing" => (
            "Edición de video",
            serde_json::json!({
                "schemaVersion": 1,
                "serviceType": "video-editing",
                "data": {
                    "pieceType": "", "quantity": 1, "rawMinutes": null, "finalDuration": "",
                    "resolution": "1080p", "editingLevel": "basic", "revisions": 1,
                    "urgency": "normal", "urgencyFeeMinor": 0, "formats": [], "estimatedHours": null,
                    "color": "none", "audio": "basic", "subtitles": "none", "videoAi": "none",
                    "voiceAi": false, "soundAi": false, "backgroundRemoval": false, "motion": "none",
                    "broll": "client", "additionalVersions": 0, "externalCosts": []
                }
            }),
        ),
        "programming" => (
            "Programación",
            serde_json::json!({
                "schemaVersion": 2, "serviceType": "programming",
                "data": { "parameterValues": {}, "externalCosts": [], "notes": "" }
            }),
        ),
        _ => {
            return Err(AppError::Validation(
                "Tipo de servicio no disponible.".into(),
            ))
        }
    };
    Ok((title.into(), value.to_string()))
}

#[tauri::command]
pub async fn add_quote_service(
    quote_id: String,
    service_type: String,
    state: State<'_, AppState>,
) -> Result<QuoteService, String> {
    async {
        let (base_title, configuration) = default_configuration(&service_type)?;
        let mut tx = state.pool.begin().await?;
        let quote_exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM quotes WHERE id = ? AND status = 'draft'")
            .bind(&quote_id).fetch_one(&mut *tx).await?;
        if quote_exists == 0 { return Err(AppError::Validation("La cotización no está disponible para editar.".into())); }
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM quote_services WHERE quote_id = ? AND service_type = ? AND deleted_at IS NULL")
            .bind(&quote_id).bind(&service_type).fetch_one(&mut *tx).await?;
        let order: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(sort_order), -1) + 1 FROM quote_services WHERE quote_id = ? AND deleted_at IS NULL")
            .bind(&quote_id).fetch_one(&mut *tx).await?;
        let id = Uuid::new_v4().to_string();
        let title = if count == 0 { base_title } else { format!("{} {}", base_title, count + 1) };
        let timestamp = now();
        sqlx::query(
            "INSERT INTO quote_services (id, quote_id, service_type, title, sort_order, configuration_version,
             configuration_json, row_revision, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 1, ?, 0, ?, ?)",
        ).bind(&id).bind(&quote_id).bind(&service_type).bind(&title).bind(order).bind(&configuration)
         .bind(&timestamp).bind(&timestamp).execute(&mut *tx).await?;
        sqlx::query("UPDATE quotes SET updated_at = ? WHERE id = ?").bind(&timestamp).bind(&quote_id).execute(&mut *tx).await?;
        tx.commit().await?;
        sqlx::query_as::<_, QuoteService>(
            "SELECT id, quote_id, service_type, title, sort_order, configuration_version, configuration_json,
             calculated_subtotal_minor, suggested_subtotal_minor, final_subtotal_minor, has_override,
             manual_subtotal_minor, manual_reason, pricing_snapshot_json, service_definition_version,
             row_revision, deleted_at, created_at, updated_at
             FROM quote_services WHERE id = ?",
        ).bind(&id).fetch_one(&state.pool).await.map_err(AppError::from)
    }.await.map_err(command_error)
}

#[tauri::command]
pub async fn save_quote_service(
    input: SaveServiceInput,
    state: State<'_, AppState>,
) -> Result<QuoteService, String> {
    async {
        if input.title.trim().is_empty() { return Err(AppError::Validation("El título del servicio es obligatorio.".into())); }
        validate_non_negative(input.calculated_subtotal_minor, "El subtotal calculado")?;
        validate_non_negative(input.suggested_subtotal_minor, "El subtotal sugerido")?;
        validate_non_negative(input.final_subtotal_minor, "El precio final")?;
        validate_non_negative(input.manual_subtotal_minor, "El precio manual")?;
        let _: Value = serde_json::from_str(&input.configuration_json)?;
        if let Some(snapshot) = &input.pricing_snapshot_json {
            let _: Value = serde_json::from_str(snapshot)?;
        }
        let timestamp = now();
        let result = sqlx::query(
            "UPDATE quote_services SET title = ?, configuration_version = ?, configuration_json = ?,
             calculated_subtotal_minor = ?, suggested_subtotal_minor = ?, final_subtotal_minor = ?,
             has_override = ?, manual_subtotal_minor = ?, manual_reason = ?, pricing_snapshot_json = ?,
             service_definition_version = ?,
             row_revision = row_revision + 1, updated_at = ?
             WHERE id = ? AND row_revision = ? AND deleted_at IS NULL",
        ).bind(input.title.trim()).bind(input.configuration_version).bind(&input.configuration_json)
         .bind(input.calculated_subtotal_minor).bind(input.suggested_subtotal_minor)
         .bind(input.final_subtotal_minor).bind(input.has_override).bind(input.manual_subtotal_minor)
         .bind(clean_optional(input.manual_reason)).bind(input.pricing_snapshot_json)
         .bind(input.service_definition_version).bind(&timestamp).bind(&input.id).bind(input.expected_revision)
         .execute(&state.pool).await?;
        if result.rows_affected() == 0 {
            let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM quote_services WHERE id = ? AND deleted_at IS NULL")
                .bind(&input.id).fetch_one(&state.pool).await?;
            return Err(if exists == 0 { AppError::NotFound } else { AppError::RevisionConflict });
        }
        sqlx::query_as::<_, QuoteService>(
            "SELECT id, quote_id, service_type, title, sort_order, configuration_version, configuration_json,
             calculated_subtotal_minor, suggested_subtotal_minor, final_subtotal_minor, has_override,
             manual_subtotal_minor, manual_reason, pricing_snapshot_json, service_definition_version,
             row_revision, deleted_at, created_at, updated_at
             FROM quote_services WHERE id = ?",
        ).bind(&input.id).fetch_one(&state.pool).await.map_err(AppError::from)
    }.await.map_err(command_error)
}

#[tauri::command]
pub async fn set_service_deleted(
    id: String,
    deleted: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    async {
        let result = sqlx::query("UPDATE quote_services SET deleted_at = ?, row_revision = row_revision + 1, updated_at = ? WHERE id = ?")
            .bind(if deleted { Some(now()) } else { None::<String> }).bind(now()).bind(id)
            .execute(&state.pool).await?;
        if result.rows_affected() == 0 { return Err(AppError::NotFound); }
        Ok(())
    }.await.map_err(command_error)
}

#[tauri::command]
pub async fn reorder_quote_services(
    quote_id: String,
    ordered_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    async {
        let mut tx = state.pool.begin().await?;
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM quote_services WHERE quote_id = ? AND deleted_at IS NULL")
            .bind(&quote_id).fetch_one(&mut *tx).await?;
        if count != ordered_ids.len() as i64 { return Err(AppError::Validation("El orden de servicios está incompleto.".into())); }
        for (index, id) in ordered_ids.iter().enumerate() {
            let result = sqlx::query("UPDATE quote_services SET sort_order = ?, row_revision = row_revision + 1, updated_at = ? WHERE id = ? AND quote_id = ? AND deleted_at IS NULL")
                .bind(index as i64).bind(now()).bind(id).bind(&quote_id).execute(&mut *tx).await?;
            if result.rows_affected() == 0 { return Err(AppError::NotFound); }
        }
        tx.commit().await?;
        Ok(())
    }.await.map_err(command_error)
}

#[tauri::command]
pub async fn update_settings(
    input: SettingsInput,
    state: State<'_, AppState>,
) -> Result<AppSettings, String> {
    async {
        if !matches!(input.theme.as_str(), "warm" | "dark") { return Err(AppError::Validation("Tema inválido.".into())); }
        if !matches!(input.suggestion_strategy.as_str(), "competitive" | "balanced" | "premium") {
            return Err(AppError::Validation("Estrategia de sugerencias inválida.".into()));
        }
        validate_currency(&input.base_currency)?;
        validate_non_negative(input.hourly_rate_ars_minor, "La tarifa ARS")?;
        validate_non_negative(input.hourly_rate_usd_minor, "La tarifa USD")?;
        if input.usd_to_ars_micros.is_some_and(|value| value <= 0) { return Err(AppError::Validation("El cambio debe ser mayor que cero.".into())); }
        sqlx::query("UPDATE app_settings SET theme = ?, hourly_rate_ars_minor = ?, hourly_rate_usd_minor = ?, usd_to_ars_micros = ?, suggestions_enabled = ?, suggestion_strategy = ?, base_currency = ?, updated_at = ? WHERE id = 1")
            .bind(input.theme).bind(input.hourly_rate_ars_minor).bind(input.hourly_rate_usd_minor)
            .bind(input.usd_to_ars_micros).bind(input.suggestions_enabled).bind(input.suggestion_strategy)
            .bind(input.base_currency).bind(now()).execute(&state.pool).await?;
        sqlx::query("UPDATE economic_profiles SET manual_hourly_rate_minor = CASE currency WHEN 'ARS' THEN ? ELSE ? END, updated_at = ? WHERE currency IN ('ARS','USD')")
            .bind(input.hourly_rate_ars_minor).bind(input.hourly_rate_usd_minor).bind(now()).execute(&state.pool).await?;
        settings(&state.pool).await
    }.await.map_err(command_error)
}

fn convert_minor(amount: i64, from: &str, to: &str, rate_micros: i64) -> i64 {
    if from == to {
        return amount;
    }
    if from == "USD" {
        ((amount as i128 * rate_micros as i128 + 5_000) / 10_000) as i64
    } else {
        ((amount as i128 * 10_000 + (rate_micros as i128 / 2)) / rate_micros as i128) as i64
    }
}

#[tauri::command]
pub async fn change_project_currency(
    project_id: String,
    currency: String,
    state: State<'_, AppState>,
) -> Result<Workspace, String> {
    async {
        validate_currency(&currency)?;
        let current: String = sqlx::query_scalar("SELECT currency FROM projects WHERE id = ?")
            .bind(&project_id).fetch_optional(&state.pool).await?.ok_or(AppError::NotFound)?;
        if current == currency { return workspace(&state.pool, &project_id).await; }
        let app_settings = settings(&state.pool).await?;
        let rate = app_settings.usd_to_ars_micros;
        let mut tx = state.pool.begin().await?;
        let rows: Vec<(String, String, Option<i64>)> = sqlx::query_as(
            "SELECT qs.id, qs.configuration_json, qs.manual_subtotal_minor
             FROM quote_services qs JOIN quotes q ON q.id = qs.quote_id
             WHERE q.project_id = ? AND qs.deleted_at IS NULL",
        ).bind(&project_id).fetch_all(&mut *tx).await?;
        let has_currency_values = rows.iter().any(|(_, json, manual)| {
            let urgency = serde_json::from_str::<Value>(json).ok()
                .and_then(|value| value.pointer("/data/urgencyFeeMinor").and_then(Value::as_i64)).unwrap_or(0);
            manual.unwrap_or(0) > 0 || urgency > 0
        });
        if has_currency_values && rate.is_none() {
            return Err(AppError::Validation("Configurá el cambio USD/ARS antes de cambiar la moneda de este proyecto.".into()));
        }
        for (id, json, manual) in rows {
            let mut value: Value = serde_json::from_str(&json)?;
            if let Some(urgency) = value.pointer("/data/urgencyFeeMinor").and_then(Value::as_i64) {
                if let Some(slot) = value.pointer_mut("/data/urgencyFeeMinor") {
                    *slot = Value::from(convert_minor(urgency, &current, &currency, rate.unwrap_or(10_000)));
                }
            }
            let converted_manual = manual.map(|amount| convert_minor(amount, &current, &currency, rate.unwrap_or(10_000)));
            sqlx::query("UPDATE quote_services SET configuration_json = ?, manual_subtotal_minor = ?, final_subtotal_minor = CASE WHEN has_override = 1 THEN ? ELSE NULL END, calculated_subtotal_minor = NULL, suggested_subtotal_minor = NULL, pricing_snapshot_json = NULL, service_definition_version = NULL, row_revision = row_revision + 1, updated_at = ? WHERE id = ?")
                .bind(value.to_string()).bind(converted_manual).bind(converted_manual).bind(now()).bind(id).execute(&mut *tx).await?;
        }
        sqlx::query("UPDATE projects SET currency = ?, updated_at = ? WHERE id = ?")
            .bind(&currency).bind(now()).bind(&project_id).execute(&mut *tx).await?;
        sqlx::query("UPDATE quotes SET currency = ?, updated_at = ? WHERE project_id = ?")
            .bind(&currency).bind(now()).bind(&project_id).execute(&mut *tx).await?;
        tx.commit().await?;
        workspace(&state.pool, &project_id).await
    }.await.map_err(command_error)
}

#[tauri::command]
pub async fn save_preset(input: PresetInput, state: State<'_, AppState>) -> Result<Preset, String> {
    async {
        if input.name.trim().is_empty() { return Err(AppError::Validation("El nombre del preset es obligatorio.".into())); }
        let _: Value = serde_json::from_str(&input.configuration_json)?;
        let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let timestamp = now();
        sqlx::query(
            "INSERT INTO service_presets (id, service_type, name, origin, configuration_version, definition_version, configuration_json, created_at, updated_at)
             VALUES (?, ?, ?, 'user', ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET name=excluded.name, configuration_version=excluded.configuration_version,
              definition_version=excluded.definition_version, configuration_json=excluded.configuration_json, updated_at=excluded.updated_at",
        ).bind(&id).bind(&input.service_type).bind(input.name.trim()).bind(input.configuration_version)
         .bind(input.definition_version.unwrap_or(1)).bind(&input.configuration_json).bind(&timestamp).bind(&timestamp).execute(&state.pool).await?;
        sqlx::query_as::<_, Preset>(
            "SELECT id, service_type, name, origin, system_key, configuration_version, definition_version, configuration_json, created_at, updated_at FROM service_presets WHERE id = ?",
        ).bind(&id).fetch_one(&state.pool).await.map_err(AppError::from)
    }.await.map_err(command_error)
}

#[tauri::command]
pub async fn delete_user_preset(id: String, state: State<'_, AppState>) -> Result<(), String> {
    async {
        let result = sqlx::query("DELETE FROM service_presets WHERE id = ? AND origin = 'user'")
            .bind(id)
            .execute(&state.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::Validation(
                "Sólo se pueden eliminar presets creados por el usuario.".into(),
            ));
        }
        Ok(())
    }
    .await
    .map_err(command_error)
}

#[tauri::command]
pub async fn restore_system_preset(
    id: String,
    state: State<'_, AppState>,
) -> Result<Preset, String> {
    async {
        let result = sqlx::query("UPDATE service_presets SET configuration_json = default_configuration_json, updated_at = ? WHERE id = ? AND origin = 'system'")
            .bind(now()).bind(&id).execute(&state.pool).await?;
        if result.rows_affected() == 0 { return Err(AppError::NotFound); }
        sqlx::query_as::<_, Preset>(
            "SELECT id, service_type, name, origin, system_key, configuration_version, definition_version, configuration_json, created_at, updated_at FROM service_presets WHERE id = ?",
        ).bind(&id).fetch_one(&state.pool).await.map_err(AppError::from)
    }.await.map_err(command_error)
}

fn validate_margin(value: Option<i64>, field: &str) -> AppResult<()> {
    if value.is_some_and(|value| !(0..1_000_000).contains(&value)) {
        Err(AppError::Validation(format!(
            "{field} debe estar entre 0% y menos de 100%."
        )))
    } else {
        Ok(())
    }
}

async fn bump_definition_version(
    tx: &mut Transaction<'_, Sqlite>,
    definition_id: &str,
    timestamp: &str,
) -> AppResult<()> {
    let result = sqlx::query(
        "UPDATE service_definitions SET version = version + 1, updated_at = ? WHERE id = ?",
    )
    .bind(timestamp)
    .bind(definition_id)
    .execute(&mut **tx)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

#[tauri::command]
pub async fn load_pricing_configuration(
    state: State<'_, AppState>,
) -> Result<PricingConfiguration, String> {
    pricing_configuration(&state.pool)
        .await
        .map_err(command_error)
}

#[tauri::command]
pub async fn save_service_definition(
    input: ServiceDefinitionInput,
    state: State<'_, AppState>,
) -> Result<PricingConfiguration, String> {
    async {
        if input.name.trim().is_empty() { return Err(AppError::Validation("El nombre es obligatorio.".into())); }
        if !matches!(input.default_strategy.as_str(), "competitive" | "balanced" | "premium") {
            return Err(AppError::Validation("Estrategia inválida.".into()));
        }
        validate_margin(input.competitive_margin_micros, "Margen competitivo")?;
        validate_margin(input.balanced_margin_micros, "Margen equilibrado")?;
        validate_margin(input.premium_margin_micros, "Margen premium")?;
        let result = sqlx::query(
            "UPDATE service_definitions SET name = ?, description = ?, enabled = ?, suggestions_enabled = ?,
             default_strategy = ?, competitive_margin_micros = ?, balanced_margin_micros = ?,
             premium_margin_micros = ?, version = version + 1, updated_at = ? WHERE id = ?",
        ).bind(input.name.trim()).bind(clean_optional(input.description)).bind(input.enabled)
         .bind(input.suggestions_enabled).bind(input.default_strategy)
         .bind(input.competitive_margin_micros).bind(input.balanced_margin_micros)
         .bind(input.premium_margin_micros).bind(now()).bind(input.id).execute(&state.pool).await?;
        if result.rows_affected() == 0 { return Err(AppError::NotFound); }
        pricing_configuration(&state.pool).await
    }.await.map_err(command_error)
}

#[tauri::command]
pub async fn save_service_parameter(
    input: ServiceParameterInput,
    state: State<'_, AppState>,
) -> Result<PricingConfiguration, String> {
    async {
        let allowed = ["single_select", "multi_select", "boolean", "number", "duration", "currency", "percentage", "text"];
        if !allowed.contains(&input.parameter_type.as_str()) { return Err(AppError::Validation("Tipo de parámetro inválido.".into())); }
        let key = input.parameter_key.trim();
        if key.is_empty() || input.name.trim().is_empty() || input.label.trim().is_empty() {
            return Err(AppError::Validation("Clave, nombre y etiqueta son obligatorios.".into()));
        }
        if let Some(value) = &input.default_value_json { let _: Value = serde_json::from_str(value)?; }
        let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let timestamp = now();
        let mut tx = state.pool.begin().await?;
        sqlx::query(
            "INSERT INTO service_parameters (id, service_definition_id, parameter_key, name, label,
             parameter_type, description, required, sort_order, enabled, default_value_json,
             suggestion_enabled, is_system, ui_managed, version, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 0, 1, ?, ?)
             ON CONFLICT(id) DO UPDATE SET parameter_key=excluded.parameter_key, name=excluded.name,
             label=excluded.label, parameter_type=excluded.parameter_type, description=excluded.description,
             required=excluded.required, sort_order=excluded.sort_order, enabled=excluded.enabled,
             default_value_json=excluded.default_value_json, suggestion_enabled=excluded.suggestion_enabled,
             version=service_parameters.version+1, updated_at=excluded.updated_at",
        ).bind(&id).bind(&input.service_definition_id).bind(key).bind(input.name.trim())
         .bind(input.label.trim()).bind(input.parameter_type).bind(clean_optional(input.description))
         .bind(input.required).bind(input.sort_order).bind(input.enabled).bind(input.default_value_json)
         .bind(input.suggestion_enabled).bind(&timestamp).bind(&timestamp).execute(&mut *tx).await?;
        bump_definition_version(&mut tx, &input.service_definition_id, &timestamp).await?;
        tx.commit().await?;
        pricing_configuration(&state.pool).await
    }.await.map_err(command_error)
}

#[tauri::command]
pub async fn delete_service_parameter(
    id: String,
    state: State<'_, AppState>,
) -> Result<PricingConfiguration, String> {
    async {
        let parameter: (String, bool) = sqlx::query_as(
            "SELECT service_definition_id, is_system FROM service_parameters WHERE id = ?",
        )
        .bind(&id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;
        if parameter.1 {
            return Err(AppError::Validation(
                "Un parámetro del sistema se desactiva; no se elimina.".into(),
            ));
        }
        let timestamp = now();
        let mut tx = state.pool.begin().await?;
        sqlx::query("DELETE FROM service_parameters WHERE id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        bump_definition_version(&mut tx, &parameter.0, &timestamp).await?;
        tx.commit().await?;
        pricing_configuration(&state.pool).await
    }
    .await
    .map_err(command_error)
}

#[tauri::command]
pub async fn save_parameter_option(
    input: ParameterOptionInput,
    state: State<'_, AppState>,
) -> Result<PricingConfiguration, String> {
    async {
        if input.label.trim().is_empty() || input.value.trim().is_empty() {
            return Err(AppError::Validation("Etiqueta y valor son obligatorios.".into()));
        }
        let definition_id: String = sqlx::query_scalar("SELECT service_definition_id FROM service_parameters WHERE id = ?")
            .bind(&input.parameter_id).fetch_optional(&state.pool).await?.ok_or(AppError::NotFound)?;
        let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let timestamp = now();
        let mut tx = state.pool.begin().await?;
        sqlx::query(
            "INSERT INTO parameter_options (id, parameter_id, label, value, sort_order, enabled, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET label=excluded.label, value=excluded.value,
             sort_order=excluded.sort_order, enabled=excluded.enabled, updated_at=excluded.updated_at",
        ).bind(id).bind(input.parameter_id).bind(input.label.trim()).bind(input.value.trim())
         .bind(input.sort_order).bind(input.enabled).bind(&timestamp).bind(&timestamp).execute(&mut *tx).await?;
        bump_definition_version(&mut tx, &definition_id, &timestamp).await?;
        tx.commit().await?;
        pricing_configuration(&state.pool).await
    }.await.map_err(command_error)
}

#[tauri::command]
pub async fn delete_parameter_option(
    id: String,
    state: State<'_, AppState>,
) -> Result<PricingConfiguration, String> {
    async {
        let definition_id: String = sqlx::query_scalar(
            "SELECT p.service_definition_id FROM parameter_options o JOIN service_parameters p ON p.id=o.parameter_id WHERE o.id=?",
        ).bind(&id).fetch_optional(&state.pool).await?.ok_or(AppError::NotFound)?;
        let timestamp = now();
        let mut tx = state.pool.begin().await?;
        sqlx::query("DELETE FROM parameter_options WHERE id=?").bind(id).execute(&mut *tx).await?;
        bump_definition_version(&mut tx, &definition_id, &timestamp).await?;
        tx.commit().await?;
        pricing_configuration(&state.pool).await
    }.await.map_err(command_error)
}

#[tauri::command]
pub async fn save_pricing_rule(
    input: PricingRuleInput,
    state: State<'_, AppState>,
) -> Result<PricingConfiguration, String> {
    async {
        let allowed = ["fixed_amount", "hours", "per_unit", "percentage", "multiplier", "external_cost"];
        if !allowed.contains(&input.rule_type.as_str()) { return Err(AppError::Validation("Tipo de regla inválido.".into())); }
        if input.name.trim().is_empty() { return Err(AppError::Validation("El nombre es obligatorio.".into())); }
        validate_non_negative(input.amount_ars_minor, "Importe ARS")?;
        validate_non_negative(input.amount_usd_minor, "Importe USD")?;
        if matches!(input.rule_type.as_str(), "hours" | "percentage" | "multiplier" | "per_unit")
            && input.numeric_value_micros.is_none() {
            return Err(AppError::Validation("La regla requiere un valor numérico.".into()));
        }
        if input.numeric_value_micros.is_some_and(|value| value < 0) {
            return Err(AppError::Validation("El valor de la regla no puede ser negativo.".into()));
        }
        let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let timestamp = now();
        let mut tx = state.pool.begin().await?;
        sqlx::query(
            "INSERT INTO pricing_rules (id, service_definition_id, parameter_id, option_id,
             quantity_parameter_id, name, rule_type, numeric_value_micros, amount_ars_minor,
             amount_usd_minor, sort_order, enabled, version, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1, ?, ?)
             ON CONFLICT(id) DO UPDATE SET parameter_id=excluded.parameter_id, option_id=excluded.option_id,
             quantity_parameter_id=excluded.quantity_parameter_id, name=excluded.name,
             rule_type=excluded.rule_type, numeric_value_micros=excluded.numeric_value_micros,
             amount_ars_minor=excluded.amount_ars_minor, amount_usd_minor=excluded.amount_usd_minor,
             sort_order=excluded.sort_order, enabled=excluded.enabled, version=pricing_rules.version+1,
             updated_at=excluded.updated_at",
        ).bind(id).bind(&input.service_definition_id).bind(input.parameter_id).bind(input.option_id)
         .bind(input.quantity_parameter_id).bind(input.name.trim()).bind(input.rule_type)
         .bind(input.numeric_value_micros).bind(input.amount_ars_minor).bind(input.amount_usd_minor)
         .bind(input.sort_order).bind(input.enabled).bind(&timestamp).bind(&timestamp).execute(&mut *tx).await?;
        bump_definition_version(&mut tx, &input.service_definition_id, &timestamp).await?;
        tx.commit().await?;
        pricing_configuration(&state.pool).await
    }.await.map_err(command_error)
}

#[tauri::command]
pub async fn delete_pricing_rule(
    id: String,
    state: State<'_, AppState>,
) -> Result<PricingConfiguration, String> {
    async {
        let definition_id: String =
            sqlx::query_scalar("SELECT service_definition_id FROM pricing_rules WHERE id=?")
                .bind(&id)
                .fetch_optional(&state.pool)
                .await?
                .ok_or(AppError::NotFound)?;
        let timestamp = now();
        let mut tx = state.pool.begin().await?;
        sqlx::query("DELETE FROM pricing_rules WHERE id=?")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        bump_definition_version(&mut tx, &definition_id, &timestamp).await?;
        tx.commit().await?;
        pricing_configuration(&state.pool).await
    }
    .await
    .map_err(command_error)
}

#[tauri::command]
pub async fn save_economic_profile(
    input: EconomicProfileInput,
    state: State<'_, AppState>,
) -> Result<PricingConfiguration, String> {
    async {
        validate_currency(&input.currency)?;
        validate_non_negative(input.monthly_income_target_minor, "Objetivo mensual")?;
        validate_non_negative(input.monthly_expenses_minor, "Gastos mensuales")?;
        validate_non_negative(input.manual_hourly_rate_minor, "Tarifa manual")?;
        validate_margin(input.reserve_tax_micros, "Reserva e impuestos")?;
        validate_margin(input.desired_margin_micros, "Margen deseado")?;
        if input.billable_hours_micros.is_some_and(|v| v <= 0) { return Err(AppError::Validation("Las horas facturables deben ser mayores que cero.".into())); }
        sqlx::query(
            "INSERT INTO economic_profiles (currency, monthly_income_target_minor, monthly_expenses_minor,
             billable_hours_micros, reserve_tax_micros, desired_margin_micros, default_urgency_micros,
             work_days, vacation_weeks, manual_hourly_rate_minor, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(currency) DO UPDATE SET monthly_income_target_minor=excluded.monthly_income_target_minor,
             monthly_expenses_minor=excluded.monthly_expenses_minor, billable_hours_micros=excluded.billable_hours_micros,
             reserve_tax_micros=excluded.reserve_tax_micros, desired_margin_micros=excluded.desired_margin_micros,
             default_urgency_micros=excluded.default_urgency_micros, work_days=excluded.work_days,
             vacation_weeks=excluded.vacation_weeks, manual_hourly_rate_minor=excluded.manual_hourly_rate_minor,
             updated_at=excluded.updated_at",
        ).bind(&input.currency).bind(input.monthly_income_target_minor).bind(input.monthly_expenses_minor)
         .bind(input.billable_hours_micros).bind(input.reserve_tax_micros).bind(input.desired_margin_micros)
         .bind(input.default_urgency_micros).bind(input.work_days).bind(input.vacation_weeks)
         .bind(input.manual_hourly_rate_minor).bind(now()).execute(&state.pool).await?;
        if input.currency == "ARS" {
            sqlx::query("UPDATE app_settings SET hourly_rate_ars_minor=?, updated_at=? WHERE id=1")
                .bind(input.manual_hourly_rate_minor).bind(now()).execute(&state.pool).await?;
        } else {
            sqlx::query("UPDATE app_settings SET hourly_rate_usd_minor=?, updated_at=? WHERE id=1")
                .bind(input.manual_hourly_rate_minor).bind(now()).execute(&state.pool).await?;
        }
        pricing_configuration(&state.pool).await
    }.await.map_err(command_error)
}

#[tauri::command]
pub async fn save_market_source(
    input: MarketSourceInput,
    state: State<'_, AppState>,
) -> Result<PricingConfiguration, String> {
    async {
        if input.name.trim().is_empty() {
            return Err(AppError::Validation(
                "El nombre de la fuente es obligatorio.".into(),
            ));
        }
        let _: Value = serde_json::from_str(&input.regions_json)?;
        let _: Value = serde_json::from_str(&input.supported_services_json)?;
        let allowed_types = [
            "freelance_marketplace",
            "rate_benchmark",
            "professional_tariff",
            "salary",
            "job_board",
            "agency_pricing",
            "methodology",
            "currency",
            "other",
        ];
        let allowed_usage = [
            "market_price",
            "salary_context",
            "rate_methodology",
            "currency",
            "context_only",
        ];
        let allowed_acquisition = ["auto_http", "auto_browser", "manual", "disabled"];
        if !allowed_types.contains(&input.source_type.as_str())
            || !allowed_usage.contains(&input.usage_mode.as_str())
            || !allowed_acquisition.contains(&input.acquisition_mode.as_str())
        {
            return Err(AppError::Validation(
                "Clasificación de fuente inválida.".into(),
            ));
        }
        let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let timestamp = now();
        sqlx::query(
            "INSERT INTO market_sources (id, name, base_url, source_type, regions_json,
             supported_services_json, priority, enabled, usage_mode, acquisition_mode,
             cooldown_hours, notes, is_system_source, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?, ?)
             ON CONFLICT(id) DO UPDATE SET name=excluded.name, base_url=excluded.base_url,
             source_type=excluded.source_type, regions_json=excluded.regions_json,
             supported_services_json=excluded.supported_services_json, priority=excluded.priority,
             enabled=excluded.enabled, usage_mode=excluded.usage_mode,
             acquisition_mode=excluded.acquisition_mode, cooldown_hours=excluded.cooldown_hours,
             notes=excluded.notes, updated_at=excluded.updated_at",
        )
        .bind(id)
        .bind(input.name.trim())
        .bind(clean_optional(input.base_url))
        .bind(input.source_type)
        .bind(input.regions_json)
        .bind(input.supported_services_json)
        .bind(input.priority)
        .bind(input.enabled)
        .bind(input.usage_mode)
        .bind(input.acquisition_mode)
        .bind(input.cooldown_hours)
        .bind(clean_optional(input.notes))
        .bind(&timestamp)
        .bind(&timestamp)
        .execute(&state.pool)
        .await?;
        pricing_configuration(&state.pool).await
    }
    .await
    .map_err(command_error)
}

#[tauri::command]
pub async fn delete_market_source(
    id: String,
    state: State<'_, AppState>,
) -> Result<PricingConfiguration, String> {
    async {
        let result = sqlx::query("DELETE FROM market_sources WHERE id=? AND is_system_source=0")
            .bind(id)
            .execute(&state.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::Validation(
                "Las fuentes del catálogo se restauran; no se eliminan.".into(),
            ));
        }
        pricing_configuration(&state.pool).await
    }
    .await
    .map_err(command_error)
}

#[tauri::command]
pub async fn restore_market_source(
    id: String,
    state: State<'_, AppState>,
) -> Result<PricingConfiguration, String> {
    async {
        let default_json: String = sqlx::query_scalar(
            "SELECT default_data_json FROM market_sources WHERE id=? AND is_system_source=1",
        ).bind(&id).fetch_optional(&state.pool).await?.flatten().ok_or(AppError::NotFound)?;
        let value: Value = serde_json::from_str(&default_json)?;
        let enabled = value.get("enabled").and_then(Value::as_bool).unwrap_or(false);
        let priority = value.get("priority").and_then(Value::as_i64).unwrap_or(0);
        let acquisition = value.get("acquisitionMode").and_then(Value::as_str).unwrap_or("disabled");
        sqlx::query("UPDATE market_sources SET enabled=?, priority=?, acquisition_mode=?, updated_at=? WHERE id=?")
            .bind(enabled).bind(priority).bind(acquisition).bind(now()).bind(id).execute(&state.pool).await?;
        pricing_configuration(&state.pool).await
    }.await.map_err(command_error)
}

#[cfg(test)]
mod tests {
    use super::convert_minor;

    #[test]
    fn currency_conversion_round_trip_is_stable() {
        let ars = convert_minor(10_000, "USD", "ARS", 13_205_000);
        assert_eq!(ars, 13_205_000);
        assert_eq!(convert_minor(ars, "ARS", "USD", 13_205_000), 10_000);
    }
}
