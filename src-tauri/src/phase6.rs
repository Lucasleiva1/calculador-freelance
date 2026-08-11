use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Datelike, NaiveDate, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use sqlx::{Row, Sqlite, SqlitePool, Transaction};
use tauri::{AppHandle, Manager, State};

use crate::{
    db::AppState,
    error::{command_error, AppError, AppResult},
};

const BACKUP_SCHEMA_VERSION: i64 = 1;
const BACKUP_TABLES: &[&str] = &[
    "clients",
    "projects",
    "quotes",
    "quote_services",
    "quote_snapshots",
    "quote_client_details",
    "service_presets",
    "app_settings",
    "professional_profile",
    "quote_number_counters",
    "service_definitions",
    "service_parameters",
    "parameter_options",
    "pricing_rules",
    "economic_profiles",
    "market_sources",
    "engine_categories",
    "pricing_engines",
    "pricing_engine_sources",
    "market_observations",
    "market_snapshots",
    "market_snapshot_observations",
    // These records reference market_sources with RESTRICT. They must travel
    // with the source catalog and be removed before it during a restore.
    "market_fetch_logs",
    "market_fx_rates",
    // Classification is persisted user data as well: restoring a backup must
    // not keep aliases or audit records from the database being replaced.
    "classification_aliases",
    "classification_runs",
];

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ProfessionalProfile {
    pub display_name: String,
    pub business_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub location: Option<String>,
    #[serde(skip_serializing)]
    pub logo_path: Option<String>,
    pub default_currency: String,
    pub default_quote_validity_days: Option<i64>,
    pub default_client_terms: Option<String>,
    pub document_theme: String,
    pub updated_at: String,
    #[sqlx(default)]
    pub logo_data_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfessionalProfileInput {
    pub display_name: String,
    pub business_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub location: Option<String>,
    pub logo_data_url: Option<String>,
    pub remove_logo: bool,
    pub default_currency: String,
    pub default_quote_validity_days: Option<i64>,
    pub default_client_terms: Option<String>,
    pub document_theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientDocumentConfig {
    pub quote_id: String,
    /// Se usa solamente para seleccionar un corte histórico al renderizar. Los
    /// detalles públicos se siguen guardando por `quote_id`.
    #[serde(default)]
    pub snapshot_revision: Option<i64>,
    pub presentation_mode: String,
    pub scope: Option<String>,
    pub revisions: Option<String>,
    pub estimated_timeline: Option<String>,
    pub client_notes: Option<String>,
    pub valid_until: Option<String>,
    pub service_descriptions: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientQuoteLine {
    pub title: String,
    pub description: Option<String>,
    pub quantity: Option<String>,
    pub price_minor: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientQuoteDocument {
    pub quote_number: String,
    pub issue_date: String,
    pub valid_until: Option<String>,
    pub currency: String,
    pub profile: ClientDocumentProfile,
    pub client_name: String,
    pub project_name: String,
    pub presentation_mode: String,
    pub lines: Vec<ClientQuoteLine>,
    pub total_minor: i64,
    pub scope: Option<String>,
    pub revisions: Option<String>,
    pub estimated_timeline: Option<String>,
    pub client_notes: Option<String>,
    pub document_theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientDocumentProfile {
    pub display_name: String,
    pub business_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub location: Option<String>,
    pub logo_data_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedFile {
    pub path: String,
    pub filename: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupSummary {
    pub schema_version: i64,
    pub exported_at: String,
    pub clients: usize,
    pub quotes: usize,
    pub sources: usize,
    pub has_profile_logo: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreResult {
    pub automatic_backup_path: String,
    pub summary: BackupSummary,
}

fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}
fn clean(value: Option<String>) -> Option<String> {
    value.and_then(|v| {
        let v = v.trim().to_string();
        (!v.is_empty()).then_some(v)
    })
}

fn profile_dir(app: &AppHandle) -> AppResult<PathBuf> {
    let directory = app
        .path()
        .app_data_dir()
        .map_err(|error| {
            AppError::Validation(format!("No se pudo localizar la carpeta local: {error}"))
        })?
        .join("profile");
    fs::create_dir_all(&directory)?;
    Ok(directory)
}

fn b64_value(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

fn decode_base64(input: &str) -> AppResult<Vec<u8>> {
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len() * 3 / 4);
    let mut chunk = [0_u8; 4];
    let mut count = 0;
    for byte in bytes.iter().copied().filter(|b| !b.is_ascii_whitespace()) {
        if byte == b'=' {
            chunk[count] = 64;
            count += 1;
        } else if let Some(value) = b64_value(byte) {
            chunk[count] = value;
            count += 1;
        } else {
            return Err(AppError::Validation(
                "El logo no contiene Base64 válido.".into(),
            ));
        }
        if count == 4 {
            if chunk[0] == 64 || chunk[1] == 64 {
                return Err(AppError::Validation(
                    "El logo no contiene Base64 válido.".into(),
                ));
            }
            output.push((chunk[0] << 2) | (chunk[1] >> 4));
            if chunk[2] != 64 {
                output.push((chunk[1] << 4) | (chunk[2] >> 2));
            }
            if chunk[3] != 64 && chunk[2] != 64 {
                output.push((chunk[2] << 6) | chunk[3]);
            }
            count = 0;
        }
    }
    if count != 0 {
        return Err(AppError::Validation(
            "El logo no contiene Base64 completo.".into(),
        ));
    }
    Ok(output)
}

fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for chunk in bytes.chunks(3) {
        let a = chunk[0];
        let b = *chunk.get(1).unwrap_or(&0);
        let c = *chunk.get(2).unwrap_or(&0);
        output.push(TABLE[(a >> 2) as usize] as char);
        output.push(TABLE[(((a & 3) << 4) | (b >> 4)) as usize] as char);
        output.push(if chunk.len() > 1 {
            TABLE[(((b & 15) << 2) | (c >> 6)) as usize] as char
        } else {
            '='
        });
        output.push(if chunk.len() > 2 {
            TABLE[(c & 63) as usize] as char
        } else {
            '='
        });
    }
    output
}

fn parse_logo_data_url(data_url: &str) -> AppResult<(String, Vec<u8>)> {
    let (header, data) = data_url
        .split_once(',')
        .ok_or_else(|| AppError::Validation("El formato del logo es inválido.".into()))?;
    let mime = header
        .strip_prefix("data:")
        .and_then(|v| v.strip_suffix(";base64"))
        .ok_or_else(|| AppError::Validation("El logo debe ser PNG o JPEG.".into()))?;
    if !matches!(mime, "image/png" | "image/jpeg") {
        return Err(AppError::Validation("El logo debe ser PNG o JPEG.".into()));
    }
    let bytes = decode_base64(data)?;
    if bytes.is_empty() || bytes.len() > 3_000_000 {
        return Err(AppError::Validation(
            "El logo debe pesar entre 1 byte y 3 MB.".into(),
        ));
    }
    let valid = (mime == "image/png" && bytes.starts_with(b"\x89PNG\r\n\x1a\n"))
        || (mime == "image/jpeg" && bytes.starts_with(&[0xff, 0xd8]));
    if !valid {
        return Err(AppError::Validation(
            "El contenido del logo no coincide con su formato.".into(),
        ));
    }
    Ok((mime.to_string(), bytes))
}

async fn profile(pool: &SqlitePool, app: &AppHandle) -> AppResult<ProfessionalProfile> {
    let mut item = sqlx::query_as::<_, ProfessionalProfile>(
        "SELECT display_name,business_name,email,phone,website,location,logo_path,default_currency,default_quote_validity_days,default_client_terms,document_theme,updated_at FROM professional_profile WHERE id=1",
    ).fetch_one(pool).await?;
    item.logo_data_url = item
        .logo_path
        .as_ref()
        .and_then(|path| fs::read(path).ok())
        .and_then(|bytes| {
            let mime = if bytes.starts_with(b"\x89PNG") {
                "image/png"
            } else if bytes.starts_with(&[0xff, 0xd8]) {
                "image/jpeg"
            } else {
                return None;
            };
            Some(format!("data:{mime};base64,{}", encode_base64(&bytes)))
        });
    if item.logo_data_url.is_none() {
        item.logo_path = None;
    }
    let _ = app;
    Ok(item)
}

#[tauri::command]
pub async fn get_professional_profile(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<ProfessionalProfile, String> {
    profile(&state.pool, &app).await.map_err(command_error)
}

#[tauri::command]
pub async fn save_professional_profile(
    input: ProfessionalProfileInput,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<ProfessionalProfile, String> {
    async {
        if !matches!(input.default_currency.as_str(), "ARS" | "USD") { return Err(AppError::Validation("La moneda predeterminada es inválida.".into())); }
        if !matches!(input.default_quote_validity_days, None | Some(7 | 15 | 30)) { return Err(AppError::Validation("La validez predeterminada no es válida.".into())); }
        if !matches!(input.document_theme.as_str(), "light" | "dark") { return Err(AppError::Validation("El modo del documento no es válido.".into())); }
        let display_name = input.display_name.trim().to_string();
        if display_name.len() > 120 { return Err(AppError::Validation("El nombre profesional es demasiado largo.".into())); }
        let existing: Option<String> = sqlx::query_scalar("SELECT logo_path FROM professional_profile WHERE id=1").fetch_one(&state.pool).await?;
        let mut logo_path = existing.clone();
        if input.remove_logo { logo_path = None; }
        if let Some(data_url) = input.logo_data_url.as_deref().filter(|v| !v.is_empty()) {
            let (mime, bytes) = parse_logo_data_url(data_url)?;
            let extension = if mime == "image/png" { "png" } else { "jpg" };
            let destination = profile_dir(&app)?.join(format!("logo-{}.{}", Utc::now().timestamp_millis(), extension));
            fs::write(&destination, bytes)?;
            logo_path = Some(destination.to_string_lossy().to_string());
        }
        sqlx::query("UPDATE professional_profile SET display_name=?,business_name=?,email=?,phone=?,website=?,location=?,logo_path=?,default_currency=?,default_quote_validity_days=?,default_client_terms=?,document_theme=?,updated_at=? WHERE id=1")
            .bind(display_name).bind(clean(input.business_name)).bind(clean(input.email)).bind(clean(input.phone)).bind(clean(input.website)).bind(clean(input.location)).bind(&logo_path).bind(input.default_currency).bind(input.default_quote_validity_days).bind(clean(input.default_client_terms)).bind(input.document_theme).bind(now()).execute(&state.pool).await?;
        if (input.remove_logo || input.logo_data_url.is_some()) && existing.as_ref().is_some_and(|path| Some(path) != logo_path.as_ref()) {
            if let Some(path) = existing { let _ = fs::remove_file(path); }
        }
        profile(&state.pool, &app).await
    }.await.map_err(command_error)
}

fn config_from_row(
    quote_id: String,
    row: Option<(
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        String,
    )>,
    default_days: Option<i64>,
    default_terms: Option<String>,
    saved_at: Option<String>,
) -> ClientDocumentConfig {
    let default_valid_until = default_days.and_then(|days| {
        saved_at
            .and_then(|date| DateTime::parse_from_rfc3339(&date).ok())
            .map(|date| {
                (date + chrono::Duration::days(days))
                    .date_naive()
                    .to_string()
            })
    });
    match row {
        Some((
            presentation_mode,
            scope,
            revisions,
            estimated_timeline,
            client_notes,
            valid_until,
            descriptions,
        )) => ClientDocumentConfig {
            quote_id,
            snapshot_revision: None,
            presentation_mode,
            scope,
            revisions,
            estimated_timeline,
            client_notes,
            valid_until,
            service_descriptions: serde_json::from_str(&descriptions).unwrap_or_default(),
        },
        None => ClientDocumentConfig {
            quote_id,
            snapshot_revision: None,
            presentation_mode: "itemized".into(),
            scope: None,
            revisions: None,
            estimated_timeline: None,
            client_notes: clean(default_terms),
            valid_until: default_valid_until,
            service_descriptions: HashMap::new(),
        },
    }
}

#[tauri::command]
pub async fn get_client_document_config(
    quote_id: String,
    state: State<'_, AppState>,
) -> Result<ClientDocumentConfig, String> {
    async {
        let profile_defaults: (Option<i64>, Option<String>) = sqlx::query_as("SELECT default_quote_validity_days,default_client_terms FROM professional_profile WHERE id=1").fetch_one(&state.pool).await?;
        let saved_at: Option<String> = sqlx::query_scalar("SELECT saved_at FROM quotes WHERE id=?").bind(&quote_id).fetch_optional(&state.pool).await?.ok_or(AppError::NotFound)?;
        let row = sqlx::query_as::<_, (String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, String)>("SELECT presentation_mode,scope,revisions,estimated_timeline,client_notes,valid_until,service_descriptions_json FROM quote_client_details WHERE quote_id=?").bind(&quote_id).fetch_optional(&state.pool).await?;
        Ok(config_from_row(quote_id, row, profile_defaults.0, profile_defaults.1, saved_at))
    }.await.map_err(command_error)
}

fn validate_document_config(config: &ClientDocumentConfig) -> AppResult<()> {
    if config.quote_id.trim().is_empty() {
        return Err(AppError::Validation("La cotización no es válida.".into()));
    }
    if config
        .snapshot_revision
        .is_some_and(|revision| revision <= 0)
    {
        return Err(AppError::Validation(
            "La revisión histórica no es válida.".into(),
        ));
    }
    if !matches!(config.presentation_mode.as_str(), "global" | "itemized") {
        return Err(AppError::Validation(
            "La forma de mostrar el precio no es válida.".into(),
        ));
    }
    if let Some(date) = config.valid_until.as_deref() {
        NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .map_err(|_| AppError::Validation("La fecha de validez no es válida.".into()))?;
    }
    for field in [
        &config.scope,
        &config.revisions,
        &config.estimated_timeline,
        &config.client_notes,
    ] {
        if field.as_ref().is_some_and(|value| value.len() > 4000) {
            return Err(AppError::Validation(
                "Un campo público es demasiado largo.".into(),
            ));
        }
    }
    Ok(())
}

async fn save_config_in_pool(pool: &SqlitePool, config: &ClientDocumentConfig) -> AppResult<()> {
    validate_document_config(config)?;
    let exists: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM quotes WHERE id=? AND saved_at IS NOT NULL")
            .bind(&config.quote_id)
            .fetch_one(pool)
            .await?;
    if exists == 0 {
        return Err(AppError::Validation(
            "Guardá la cotización antes de preparar el documento para cliente.".into(),
        ));
    }
    sqlx::query("INSERT INTO quote_client_details (quote_id,presentation_mode,scope,revisions,estimated_timeline,client_notes,valid_until,service_descriptions_json,updated_at) VALUES (?,?,?,?,?,?,?,?,?) ON CONFLICT(quote_id) DO UPDATE SET presentation_mode=excluded.presentation_mode,scope=excluded.scope,revisions=excluded.revisions,estimated_timeline=excluded.estimated_timeline,client_notes=excluded.client_notes,valid_until=excluded.valid_until,service_descriptions_json=excluded.service_descriptions_json,updated_at=excluded.updated_at")
        .bind(&config.quote_id).bind(&config.presentation_mode).bind(clean(config.scope.clone())).bind(clean(config.revisions.clone())).bind(clean(config.estimated_timeline.clone())).bind(clean(config.client_notes.clone())).bind(&config.valid_until).bind(serde_json::to_string(&config.service_descriptions)?).bind(now()).execute(pool).await?;
    Ok(())
}

#[tauri::command]
pub async fn save_client_document_config(
    config: ClientDocumentConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    save_config_in_pool(&state.pool, &config)
        .await
        .map_err(command_error)
}

async fn ensure_quote_number(
    tx: &mut Transaction<'_, Sqlite>,
    quote_id: &str,
    saved_at: &str,
) -> AppResult<String> {
    if let Some(number) =
        sqlx::query_scalar::<_, Option<String>>("SELECT quote_number FROM quotes WHERE id=?")
            .bind(quote_id)
            .fetch_one(&mut **tx)
            .await?
            .filter(|value| !value.is_empty())
    {
        return Ok(number);
    }
    let year = DateTime::parse_from_rfc3339(saved_at)
        .map(|date| date.year())
        .unwrap_or_else(|_| Utc::now().year());
    sqlx::query("INSERT OR IGNORE INTO quote_number_counters (year,next_sequence) VALUES (?,1)")
        .bind(year)
        .execute(&mut **tx)
        .await?;
    let sequence: i64 =
        sqlx::query_scalar("SELECT next_sequence FROM quote_number_counters WHERE year=?")
            .bind(year)
            .fetch_one(&mut **tx)
            .await?;
    sqlx::query("UPDATE quote_number_counters SET next_sequence=next_sequence+1 WHERE year=?")
        .bind(year)
        .execute(&mut **tx)
        .await?;
    let number = format!("PR-{year}-{sequence:04}");
    sqlx::query("UPDATE quotes SET quote_number=? WHERE id=?")
        .bind(&number)
        .bind(quote_id)
        .execute(&mut **tx)
        .await?;
    Ok(number)
}

pub(crate) async fn assign_quote_number_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    quote_id: &str,
    saved_at: &str,
) -> AppResult<String> {
    ensure_quote_number(tx, quote_id, saved_at).await
}

fn allocate_total(total: i64, services: &[(String, Option<i64>)]) -> Vec<i64> {
    if services.is_empty() {
        return Vec::new();
    }
    let weights: Vec<i64> = services
        .iter()
        .map(|(_, amount)| amount.unwrap_or(0).max(0))
        .collect();
    let sum: i64 = weights.iter().sum();
    let weights = if sum > 0 {
        weights
    } else {
        vec![1; services.len()]
    };
    let divisor: i64 = weights.iter().sum();
    let mut allocated: Vec<i64> = weights
        .iter()
        .map(|weight| total.saturating_mul(*weight) / divisor)
        .collect();
    let remainder = total - allocated.iter().sum::<i64>();
    for value in allocated.iter_mut().take(remainder.max(0) as usize) {
        *value += 1;
    }
    allocated
}

fn display_date(value: &str) -> String {
    DateTime::parse_from_rfc3339(value)
        .map(|date| date.format("%d/%m/%Y").to_string())
        .or_else(|_| {
            NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .map(|date| date.format("%d/%m/%Y").to_string())
        })
        .unwrap_or_else(|_| value.to_string())
}

fn document_from_snapshot(
    snapshot: &Value,
    quote_number: String,
    profile: &ProfessionalProfile,
    config: &ClientDocumentConfig,
) -> AppResult<ClientQuoteDocument> {
    let quote = snapshot
        .get("quote")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            AppError::Validation("El snapshot no tiene datos de cotización válidos.".into())
        })?;
    let project = snapshot
        .get("project")
        .and_then(Value::as_object)
        .ok_or_else(|| AppError::Validation("El snapshot no tiene proyecto válido.".into()))?;
    let client = snapshot
        .get("client")
        .and_then(Value::as_object)
        .ok_or_else(|| AppError::Validation("El snapshot no tiene cliente válido.".into()))?;
    let totals = snapshot
        .get("totals")
        .and_then(Value::as_object)
        .ok_or_else(|| AppError::Validation("El snapshot no tiene totales válidos.".into()))?;
    let selected = totals
        .get("selectedMinor")
        .and_then(Value::as_i64)
        .filter(|amount| *amount >= 0)
        .ok_or_else(|| {
            AppError::Validation("Elegí un precio final válido antes de exportar.".into())
        })?;
    let currency = quote
        .get("currency")
        .and_then(Value::as_str)
        .filter(|value| matches!(*value, "ARS" | "USD"))
        .ok_or_else(|| AppError::Validation("La moneda de la cotización no es válida.".into()))?
        .to_string();
    let service_values = snapshot
        .get("services")
        .and_then(Value::as_array)
        .filter(|items| !items.is_empty())
        .ok_or_else(|| {
            AppError::Validation("Agregá al menos un servicio antes de exportar.".into())
        })?;
    let services: Vec<(String, String, Option<i64>)> = service_values
        .iter()
        .map(|service| {
            let id = service
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| AppError::Validation("Un servicio histórico no es válido.".into()))?
                .to_string();
            let title = service
                .get("title")
                .and_then(Value::as_str)
                .filter(|v| !v.trim().is_empty())
                .unwrap_or("Servicio")
                .to_string();
            // Se usa sólo como peso para distribuir el precio FINAL elegido; nunca se publica como tal.
            let weight = service
                .get("finalSubtotalMinor")
                .and_then(Value::as_i64)
                .or_else(|| {
                    service
                        .get("suggestedSubtotalMinor")
                        .and_then(Value::as_i64)
                })
                .or_else(|| {
                    service
                        .get("calculatedSubtotalMinor")
                        .and_then(Value::as_i64)
                });
            Ok((id, title, weight))
        })
        .collect::<AppResult<_>>()?;
    let lines = if config.presentation_mode == "global" {
        vec![ClientQuoteLine {
            title: "Proyecto completo".into(),
            description: None,
            quantity: None,
            price_minor: selected,
        }]
    } else {
        let weights = services
            .iter()
            .map(|(id, _, amount)| (id.clone(), *amount))
            .collect::<Vec<_>>();
        allocate_total(selected, &weights)
            .into_iter()
            .zip(services.iter())
            .map(|(price_minor, (id, title, _))| ClientQuoteLine {
                title: title.clone(),
                description: config
                    .service_descriptions
                    .get(id)
                    .cloned()
                    .and_then(|value| clean(Some(value))),
                quantity: None,
                price_minor,
            })
            .collect()
    };
    Ok(ClientQuoteDocument {
        quote_number,
        issue_date: display_date(
            snapshot
                .get("savedAt")
                .and_then(Value::as_str)
                .unwrap_or(""),
        ),
        valid_until: config.valid_until.as_deref().map(display_date),
        currency,
        profile: ClientDocumentProfile {
            display_name: profile.display_name.clone(),
            business_name: profile.business_name.clone(),
            email: profile.email.clone(),
            phone: profile.phone.clone(),
            website: profile.website.clone(),
            location: profile.location.clone(),
            logo_data_url: profile.logo_data_url.clone(),
        },
        client_name: client
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("Cliente")
            .to_string(),
        project_name: project
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("Proyecto")
            .to_string(),
        presentation_mode: config.presentation_mode.clone(),
        lines,
        total_minor: selected,
        scope: clean(config.scope.clone()),
        revisions: clean(config.revisions.clone()),
        estimated_timeline: clean(config.estimated_timeline.clone()),
        client_notes: clean(config.client_notes.clone()),
        document_theme: profile.document_theme.clone(),
    })
}

async fn client_document(
    pool: &SqlitePool,
    app: &AppHandle,
    config: &ClientDocumentConfig,
) -> AppResult<ClientQuoteDocument> {
    validate_document_config(config)?;
    let mut tx = pool.begin().await?;
    let row = document_snapshot_row(&mut tx, &config.quote_id, config.snapshot_revision).await?;
    let (quote_number, snapshot_json, saved_at) = row.ok_or_else(|| {
        AppError::Validation("Guardá una cotización antes de preparar el PDF.".into())
    })?;
    let quote_number = match quote_number.filter(|value| !value.trim().is_empty()) {
        Some(value) => value,
        None => ensure_quote_number(&mut tx, &config.quote_id, &saved_at).await?,
    };
    tx.commit().await?;
    let snapshot: Value = serde_json::from_str(&snapshot_json)
        .map_err(|_| AppError::Validation("El snapshot de esta cotización está dañado.".into()))?;
    document_from_snapshot(&snapshot, quote_number, &profile(pool, app).await?, config)
}

async fn document_snapshot_row(
    tx: &mut Transaction<'_, Sqlite>,
    quote_id: &str,
    snapshot_revision: Option<i64>,
) -> AppResult<Option<(Option<String>, String, String)>> {
    Ok(sqlx::query_as(
        "SELECT q.quote_number,s.snapshot_json,q.saved_at
         FROM quotes q
         JOIN quote_snapshots s ON s.quote_id=q.id
           AND s.revision=COALESCE(?,q.snapshot_revision)
         WHERE q.id=? AND q.saved_at IS NOT NULL",
    )
    .bind(snapshot_revision)
    .bind(quote_id)
    .fetch_optional(&mut **tx)
    .await?)
}

#[tauri::command]
pub async fn create_client_quote_document(
    config: ClientDocumentConfig,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<ClientQuoteDocument, String> {
    client_document(&state.pool, &app, &config)
        .await
        .map_err(command_error)
}

fn money(amount_minor: i64, currency: &str) -> String {
    let major = amount_minor / 100;
    let cents = (amount_minor % 100).abs();
    let digits = major.abs().to_string();
    let mut grouped = String::new();
    for (index, character) in digits.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            grouped.push('.');
        }
        grouped.push(character);
    }
    let number: String = grouped.chars().rev().collect();
    if cents == 0 {
        format!(
            "{currency} {}{number}",
            if amount_minor < 0 { "-" } else { "" }
        )
    } else {
        format!(
            "{currency} {}{number},{cents:02}",
            if amount_minor < 0 { "-" } else { "" }
        )
    }
}

fn pdf_text_bytes(value: &str) -> Vec<u8> {
    value
        .chars()
        .map(|character| match character {
            '€' => 0x80,
            '‚' => 0x82,
            'ƒ' => 0x83,
            '„' => 0x84,
            '…' => 0x85,
            '†' => 0x86,
            '‡' => 0x87,
            'ˆ' => 0x88,
            '‰' => 0x89,
            'Š' => 0x8a,
            '‹' => 0x8b,
            'Œ' => 0x8c,
            'Ž' => 0x8e,
            '‘' => 0x91,
            '’' => 0x92,
            '“' => 0x93,
            '”' => 0x94,
            '•' => 0x95,
            '–' => 0x96,
            '—' => 0x97,
            '™' => 0x99,
            'š' => 0x9a,
            '›' => 0x9b,
            'œ' => 0x9c,
            'ž' => 0x9e,
            'Ÿ' => 0x9f,
            c if (c as u32) <= 255 => c as u8,
            _ => b'?',
        })
        .collect()
}

fn pdf_literal(value: &str) -> String {
    let mut output = String::new();
    for byte in pdf_text_bytes(value) {
        match byte {
            b'(' | b')' | b'\\' => {
                output.push('\\');
                output.push(byte as char);
            }
            0..=31 | 127..=255 => output.push_str(&format!("\\{:03o}", byte)),
            _ => output.push(byte as char),
        }
    }
    output
}

fn wrap(value: &str, size: f32, width: f32) -> Vec<String> {
    let max = (width / (size * 0.5)).max(8.0) as usize;
    value
        .split_whitespace()
        .fold(Vec::<String>::new(), |mut lines, word| {
            if let Some(last) = lines.last_mut() {
                if last.chars().count() + 1 + word.chars().count() <= max {
                    last.push(' ');
                    last.push_str(word);
                    return lines;
                }
            }
            lines.push(word.to_string());
            lines
        })
        .into_iter()
        .flat_map(|line| {
            if line.chars().count() <= max {
                vec![line]
            } else {
                line.chars()
                    .collect::<Vec<_>>()
                    .chunks(max)
                    .map(|chunk| chunk.iter().collect())
                    .collect()
            }
        })
        .collect()
}

struct PdfPage {
    operations: String,
    y: f32,
}
impl PdfPage {
    fn new(dark: bool) -> Self {
        let mut operations = String::new();
        if dark {
            operations.push_str("0.075 0.075 0.07 rg 0 0 595 842 re f\n");
        }
        Self {
            operations,
            y: 786.0,
        }
    }
    fn text(&mut self, x: f32, y: f32, font: &str, size: f32, color: &str, value: &str) {
        self.operations.push_str(&format!(
            "BT /{font} {size:.2} Tf {color} rg {x:.2} {y:.2} Td ({}) Tj ET\n",
            pdf_literal(value)
        ));
    }
    fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: &str) {
        self.operations.push_str(&format!(
            "{color} RG 0.6 w {x1:.2} {y1:.2} m {x2:.2} {y2:.2} l S\n"
        ));
    }
}

struct PdfImage {
    object: Vec<u8>,
    width: u32,
    height: u32,
}

fn jpeg_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    let mut index = 2;
    while index + 9 < bytes.len() {
        if bytes[index] != 0xff {
            index += 1;
            continue;
        }
        while index < bytes.len() && bytes[index] == 0xff {
            index += 1;
        }
        let marker = *bytes.get(index)?;
        index += 1;
        if marker == 0xd8 || marker == 0xd9 {
            continue;
        }
        let length = u16::from_be_bytes([*bytes.get(index)?, *bytes.get(index + 1)?]) as usize;
        if length < 7 || index + length > bytes.len() {
            return None;
        }
        if matches!(marker, 0xc0..=0xc3 | 0xc5..=0xc7 | 0xc9..=0xcb | 0xcd..=0xcf) {
            let height = u16::from_be_bytes([bytes[index + 3], bytes[index + 4]]) as u32;
            let width = u16::from_be_bytes([bytes[index + 5], bytes[index + 6]]) as u32;
            return Some((width, height));
        }
        index += length;
    }
    None
}

fn pdf_logo(data_url: &str) -> AppResult<Option<PdfImage>> {
    let (mime, bytes) = parse_logo_data_url(data_url)?;
    if mime == "image/jpeg" {
        let Some((width, height)) = jpeg_dimensions(&bytes) else {
            return Ok(None);
        };
        let mut object = format!("<< /Type /XObject /Subtype /Image /Width {width} /Height {height} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode /Length {} >>\nstream\n", bytes.len()).into_bytes();
        object.extend_from_slice(&bytes);
        object.extend_from_slice(b"\nendstream");
        return Ok(Some(PdfImage {
            object,
            width,
            height,
        }));
    }
    if bytes.len() < 33
        || &bytes[12..16] != b"IHDR"
        || bytes[24] != 8
        || !matches!(bytes[25], 0 | 2)
    {
        return Ok(None);
    }
    let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    if width == 0 || height == 0 {
        return Ok(None);
    }
    let colors = if bytes[25] == 2 { 3 } else { 1 };
    let mut index = 8;
    let mut idat = Vec::new();
    while index + 12 <= bytes.len() {
        let length = u32::from_be_bytes([
            bytes[index],
            bytes[index + 1],
            bytes[index + 2],
            bytes[index + 3],
        ]) as usize;
        let kind = &bytes[index + 4..index + 8];
        let end = index + 12 + length;
        if end > bytes.len() {
            return Ok(None);
        }
        if kind == b"IDAT" {
            idat.extend_from_slice(&bytes[index + 8..index + 8 + length]);
        }
        if kind == b"IEND" {
            break;
        }
        index = end;
    }
    if idat.is_empty() {
        return Ok(None);
    }
    let color_space = if colors == 3 {
        "/DeviceRGB"
    } else {
        "/DeviceGray"
    };
    let mut object = format!("<< /Type /XObject /Subtype /Image /Width {width} /Height {height} /ColorSpace {color_space} /BitsPerComponent 8 /Filter /FlateDecode /DecodeParms << /Predictor 15 /Colors {colors} /BitsPerComponent 8 /Columns {width} >> /Length {} >>\nstream\n", idat.len()).into_bytes();
    object.extend_from_slice(&idat);
    object.extend_from_slice(b"\nendstream");
    Ok(Some(PdfImage {
        object,
        width,
        height,
    }))
}

fn document_pdf(document: &ClientQuoteDocument) -> AppResult<Vec<u8>> {
    if document.total_minor < 0
        || document.quote_number.trim().is_empty()
        || document.lines.is_empty()
        || !matches!(document.currency.as_str(), "USD" | "ARS")
    {
        return Err(AppError::Validation(
            "El documento para cliente no es válido para exportar.".into(),
        ));
    }
    let dark = document.document_theme == "dark";
    let logo = document
        .profile
        .logo_data_url
        .as_deref()
        .and_then(|value| pdf_logo(value).ok().flatten());
    let ink = if dark {
        "0.93 0.92 0.87"
    } else {
        "0.10 0.10 0.09"
    };
    let muted = if dark {
        "0.68 0.67 0.62"
    } else {
        "0.38 0.37 0.34"
    };
    let accent = "0.70 0.09 0.07";
    let mut pages = vec![PdfPage::new(dark)];
    let new_page = |pages: &mut Vec<PdfPage>| {
        pages.push(PdfPage::new(dark));
        let page = pages.last_mut().expect("page");
        page.text(54.0, 786.0, "F2", 9.0, muted, "PRICING OS · COTIZACIÓN");
        page.text(541.0, 786.0, "F1", 9.0, muted, &document.quote_number);
        page.line(54.0, 769.0, 541.0, 769.0, muted);
        page.y = 736.0;
    };
    {
        let page = pages.last_mut().expect("page");
        if let Some(image) = &logo {
            let scale = (92.0 / image.width as f32).min(34.0 / image.height as f32);
            let width = image.width as f32 * scale;
            let height = image.height as f32 * scale;
            page.operations.push_str(&format!(
                "q {width:.2} 0 0 {height:.2} 300 748 cm /ImLogo Do Q\n"
            ));
        }
        page.text(54.0, page.y, "F2", 10.0, accent, "COTIZACIÓN");
        page.y -= 33.0;
        page.text(
            54.0,
            page.y,
            "F2",
            28.0,
            ink,
            if document
                .profile
                .business_name
                .as_deref()
                .unwrap_or("")
                .is_empty()
            {
                if document.profile.display_name.trim().is_empty() {
                    "PRICING OS"
                } else {
                    &document.profile.display_name
                }
            } else {
                document.profile.business_name.as_deref().unwrap()
            },
        );
        page.y -= 24.0;
        if !document.profile.display_name.trim().is_empty()
            && document.profile.business_name.as_deref()
                != Some(document.profile.display_name.as_str())
        {
            page.text(
                54.0,
                page.y,
                "F1",
                10.0,
                muted,
                &document.profile.display_name,
            );
            page.y -= 16.0;
        }
        let contact = [
            document.profile.email.as_deref(),
            document.profile.phone.as_deref(),
            document.profile.website.as_deref(),
            document.profile.location.as_deref(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" · ");
        if !contact.is_empty() {
            for line in wrap(&contact, 9.0, 300.0) {
                page.text(54.0, page.y, "F1", 9.0, muted, &line);
                page.y -= 13.0;
            }
        }
        page.text(410.0, 742.0, "F2", 9.0, muted, "NÚMERO");
        page.text(410.0, 727.0, "F2", 12.0, ink, &document.quote_number);
        page.text(410.0, 695.0, "F2", 9.0, muted, "FECHA");
        page.text(410.0, 680.0, "F1", 11.0, ink, &document.issue_date);
        if let Some(valid) = &document.valid_until {
            page.text(410.0, 648.0, "F2", 9.0, muted, "VÁLIDA HASTA");
            page.text(410.0, 633.0, "F1", 11.0, ink, valid);
        }
        page.y = page.y.min(586.0);
        page.line(54.0, page.y, 541.0, page.y, muted);
        page.y -= 27.0;
        page.text(54.0, page.y, "F2", 9.0, accent, "PREPARADO PARA");
        page.y -= 20.0;
        page.text(54.0, page.y, "F2", 18.0, ink, &document.client_name);
        page.y -= 33.0;
        page.text(54.0, page.y, "F2", 9.0, accent, "PROYECTO");
        page.y -= 20.0;
        page.text(54.0, page.y, "F2", 18.0, ink, &document.project_name);
        page.y -= 35.0;
    }
    for (index, line) in document.lines.iter().enumerate() {
        let description = line
            .description
            .as_deref()
            .map(|value| wrap(value, 10.0, 350.0))
            .unwrap_or_default();
        let required = 30.0 + description.len() as f32 * 14.0;
        if pages.last().expect("page").y - required < 72.0 {
            new_page(&mut pages);
        }
        let page = pages.last_mut().expect("page");
        page.line(54.0, page.y + 9.0, 541.0, page.y + 9.0, muted);
        page.text(
            54.0,
            page.y - 9.0,
            "F1",
            9.0,
            muted,
            &format!("{:02}", index + 1),
        );
        page.text(84.0, page.y - 9.0, "F2", 13.0, ink, &line.title);
        page.text(
            541.0 - money(line.price_minor, &document.currency).len() as f32 * 6.8,
            page.y - 9.0,
            "F2",
            12.0,
            ink,
            &money(line.price_minor, &document.currency),
        );
        page.y -= 28.0;
        for value in description {
            page.text(84.0, page.y, "F1", 10.0, muted, &value);
            page.y -= 14.0;
        }
        page.y -= 12.0;
    }
    if pages.last().expect("page").y < 155.0 {
        new_page(&mut pages);
    }
    let page = pages.last_mut().expect("page");
    page.line(54.0, page.y, 541.0, page.y, accent);
    page.y -= 27.0;
    page.text(54.0, page.y, "F2", 10.0, muted, "TOTAL FINAL");
    let total = money(document.total_minor, &document.currency);
    page.text(
        541.0 - total.len() as f32 * 13.0,
        page.y - 3.0,
        "F2",
        23.0,
        ink,
        &total,
    );
    page.y -= 48.0;
    for (label, value) in [
        ("ALCANCE", document.scope.as_deref()),
        ("REVISIONES", document.revisions.as_deref()),
        ("PLAZO ESTIMADO", document.estimated_timeline.as_deref()),
        ("CONDICIONES", document.client_notes.as_deref()),
    ] {
        if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
            let wrapped = wrap(value, 10.0, 470.0);
            let needed = 25.0 + wrapped.len() as f32 * 14.0;
            if pages.last().expect("page").y - needed < 62.0 {
                new_page(&mut pages);
            }
            let page = pages.last_mut().expect("page");
            page.text(54.0, page.y, "F2", 9.0, accent, label);
            page.y -= 16.0;
            for item in wrapped {
                page.text(54.0, page.y, "F1", 10.0, ink, &item);
                page.y -= 14.0;
            }
            page.y -= 12.0;
        }
    }
    for page in &mut pages {
        page.text(
            54.0,
            38.0,
            "F1",
            8.0,
            muted,
            "Documento generado localmente por Pricing OS",
        );
        page.text(500.0, 38.0, "F1", 8.0, muted, "Cotización");
    }
    Ok(build_pdf(pages, dark, logo))
}

fn add_pdf_object(objects: &mut Vec<Vec<u8>>, object: Vec<u8>) -> usize {
    objects.push(object);
    objects.len()
}

fn build_pdf(pages: Vec<PdfPage>, _dark: bool, logo: Option<PdfImage>) -> Vec<u8> {
    let mut objects: Vec<Vec<u8>> = Vec::new();
    let font_regular = add_pdf_object(
        &mut objects,
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>"
            .to_vec(),
    );
    let font_bold = add_pdf_object(
        &mut objects,
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>"
            .to_vec(),
    );
    let pages_object = add_pdf_object(&mut objects, Vec::new());
    let logo_object = logo
        .as_ref()
        .map(|image| add_pdf_object(&mut objects, image.object.clone()));
    let mut page_objects = Vec::new();
    for page in pages {
        let content = page.operations.into_bytes();
        let content_id = add_pdf_object(
            &mut objects,
            format!("<< /Length {} >>\nstream\n", content.len())
                .into_bytes()
                .into_iter()
                .chain(content)
                .chain(b"\nendstream".to_vec())
                .collect(),
        );
        let images = logo_object
            .map(|id| format!(" /XObject << /ImLogo {id} 0 R >>"))
            .unwrap_or_default();
        let page_id = add_pdf_object(&mut objects, format!("<< /Type /Page /Parent {pages_object} 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 {font_regular} 0 R /F2 {font_bold} 0 R >>{images} >> /Contents {content_id} 0 R >>").into_bytes());
        page_objects.push(page_id);
    }
    objects[pages_object - 1] = format!(
        "<< /Type /Pages /Kids [{}] /Count {} >>",
        page_objects
            .iter()
            .map(|id| format!("{id} 0 R"))
            .collect::<Vec<_>>()
            .join(" "),
        page_objects.len()
    )
    .into_bytes();
    let catalog = add_pdf_object(
        &mut objects,
        format!("<< /Type /Catalog /Pages {pages_object} 0 R >>").into_bytes(),
    );
    let mut result = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets = vec![0_usize];
    for (index, object) in objects.iter().enumerate() {
        offsets.push(result.len());
        result.extend_from_slice(format!("{} 0 obj\n", index + 1).as_bytes());
        result.extend_from_slice(object);
        result.extend_from_slice(b"\nendobj\n");
    }
    let xref = result.len();
    result.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for offset in offsets.iter().skip(1) {
        result.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    result.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root {catalog} 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    result
}

fn safe_filename(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect()
}
fn export_dir(app: &AppHandle, kind: &str) -> AppResult<PathBuf> {
    let base = app
        .path()
        .document_dir()
        .or_else(|_| app.path().app_data_dir())
        .map_err(|error| {
            AppError::Validation(format!("No se pudo acceder a Documentos: {error}"))
        })?;
    let path = base.join("Pricing OS").join(kind);
    fs::create_dir_all(&path)?;
    Ok(path)
}
fn unused_path(directory: PathBuf, file_name: String) -> PathBuf {
    let candidate = directory.join(&file_name);
    if !candidate.exists() {
        return candidate;
    }
    let stem = Path::new(&file_name)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("archivo");
    let extension = Path::new(&file_name)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    (2..10_000)
        .map(|index| directory.join(format!("{stem}-{index}.{extension}")))
        .find(|path| !path.exists())
        .unwrap_or(candidate)
}

#[tauri::command]
pub async fn export_client_quote_pdf(
    document: ClientQuoteDocument,
    app: AppHandle,
) -> Result<ExportedFile, String> {
    async {
        let bytes = document_pdf(&document)?;
        let file_name = format!(
            "{}-{}.pdf",
            safe_filename(&document.quote_number),
            safe_filename(&document.project_name)
        );
        let destination = unused_path(export_dir(&app, "Cotizaciones")?, file_name);
        fs::write(&destination, bytes)?;
        Ok(ExportedFile {
            filename: destination
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("cotizacion.pdf")
                .to_string(),
            path: destination.to_string_lossy().to_string(),
        })
    }
    .await
    .map_err(command_error)
}

async fn table_columns(pool: &SqlitePool, table: &str) -> AppResult<Vec<String>> {
    Ok(sqlx::query(&format!("PRAGMA table_info({table})"))
        .fetch_all(pool)
        .await?
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect())
}
async fn table_rows(pool: &SqlitePool, table: &str) -> AppResult<Vec<Value>> {
    let columns = table_columns(pool, table).await?;
    let args = columns
        .iter()
        .map(|column| {
            format!(
                "'{}', \"{}\"",
                column.replace('\'', "''"),
                column.replace('"', "\"\"")
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let rows: Vec<String> = sqlx::query_scalar(&format!("SELECT json_object({args}) FROM {table}"))
        .fetch_all(pool)
        .await?;
    rows.into_iter()
        .map(|row| serde_json::from_str(&row).map_err(AppError::from))
        .collect()
}

async fn backup_payload(pool: &SqlitePool) -> AppResult<Value> {
    let mut data = Map::new();
    for table in BACKUP_TABLES {
        data.insert(
            (*table).to_string(),
            Value::Array(table_rows(pool, table).await?),
        );
    }
    let logo_path: Option<String> =
        sqlx::query_scalar("SELECT logo_path FROM professional_profile WHERE id=1")
            .fetch_one(pool)
            .await?;
    if let Some(bytes) = logo_path.as_deref().and_then(|path| fs::read(path).ok()) {
        let mime = if bytes.starts_with(b"\x89PNG") {
            "image/png"
        } else if bytes.starts_with(&[0xff, 0xd8]) {
            "image/jpeg"
        } else {
            ""
        };
        if !mime.is_empty() {
            data.insert(
                "profileLogo".into(),
                json!({"mime": mime, "dataBase64": encode_base64(&bytes)}),
            );
        }
    }
    Ok(json!({"schemaVersion": BACKUP_SCHEMA_VERSION, "exportedAt": now(), "data": data}))
}
fn backup_summary(payload: &Value) -> AppResult<BackupSummary> {
    let version = payload
        .get("schemaVersion")
        .and_then(Value::as_i64)
        .ok_or_else(|| {
            AppError::Validation("El backup no contiene una versión de esquema.".into())
        })?;
    if version != BACKUP_SCHEMA_VERSION {
        return Err(AppError::Validation(format!(
            "La versión {version} del backup no es compatible."
        )));
    }
    let exported_at = payload
        .get("exportedAt")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            AppError::Validation("El backup no contiene fecha de exportación.".into())
        })?;
    let data = payload
        .get("data")
        .and_then(Value::as_object)
        .ok_or_else(|| AppError::Validation("El backup no contiene datos válidos.".into()))?;
    for table in BACKUP_TABLES {
        if !data.get(*table).is_some_and(Value::is_array) {
            return Err(AppError::Validation(format!(
                "El backup no contiene la tabla requerida: {table}."
            )));
        }
    }
    let has_logo = data.get("profileLogo").is_some();
    if let Some(logo) = data.get("profileLogo") {
        let mime = logo.get("mime").and_then(Value::as_str);
        let encoded = logo.get("dataBase64").and_then(Value::as_str);
        if !matches!(mime, Some("image/png" | "image/jpeg"))
            || encoded
                .and_then(|value| decode_base64(value).ok())
                .is_none()
        {
            return Err(AppError::Validation(
                "El logo incluido en el backup no es válido.".into(),
            ));
        }
    }
    Ok(BackupSummary {
        schema_version: version,
        exported_at: exported_at.to_string(),
        clients: data
            .get("clients")
            .and_then(Value::as_array)
            .map_or(0, Vec::len),
        quotes: data
            .get("quotes")
            .and_then(Value::as_array)
            .map_or(0, Vec::len),
        sources: data
            .get("market_sources")
            .and_then(Value::as_array)
            .map_or(0, Vec::len),
        has_profile_logo: has_logo,
    })
}

#[tauri::command]
pub async fn create_pricing_backup(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<ExportedFile, String> {
    async {
        let payload = backup_payload(&state.pool).await?;
        let timestamp = Utc::now().format("%Y-%m-%d-%H%M%S");
        let path = unused_path(
            export_dir(&app, "Backups")?,
            format!("pricing-os-backup-{timestamp}.json"),
        );
        fs::write(&path, serde_json::to_vec_pretty(&payload)?)?;
        Ok(ExportedFile {
            filename: path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("backup.json")
                .to_string(),
            path: path.to_string_lossy().to_string(),
        })
    }
    .await
    .map_err(command_error)
}

#[tauri::command]
pub async fn inspect_pricing_backup(content: String) -> Result<BackupSummary, String> {
    serde_json::from_str::<Value>(&content)
        .map_err(|_| {
            AppError::Validation("El archivo no es un backup JSON válido de Pricing OS.".into())
        })
        .and_then(|payload| backup_summary(&payload))
        .map_err(command_error)
}

fn bind_value<'q>(
    query: sqlx::query::Query<'q, Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
    value: &Value,
) -> AppResult<sqlx::query::Query<'q, Sqlite, sqlx::sqlite::SqliteArguments<'q>>> {
    match value {
        Value::Null => Ok(query.bind(Option::<String>::None)),
        Value::Bool(value) => Ok(query.bind(*value)),
        Value::Number(value) if value.is_i64() => {
            Ok(query.bind(value.as_i64().unwrap_or_default()))
        }
        Value::Number(value) if value.is_u64() => {
            Ok(query.bind(value.as_u64().unwrap_or_default() as i64))
        }
        Value::Number(value) => Ok(query.bind(value.as_f64().unwrap_or_default())),
        Value::String(value) => Ok(query.bind(value.clone())),
        _ => Err(AppError::Validation(
            "El backup contiene un valor de tabla no permitido.".into(),
        )),
    }
}
async fn insert_rows(
    tx: &mut Transaction<'_, Sqlite>,
    table: &str,
    columns: &[String],
    rows: &[Value],
) -> AppResult<()> {
    for row in rows {
        let object = row.as_object().ok_or_else(|| {
            AppError::Validation(format!("La tabla {table} contiene una fila inválida."))
        })?;
        let keys = columns
            .iter()
            .filter(|key| object.contains_key(*key))
            .cloned()
            .collect::<Vec<_>>();
        if keys.is_empty() {
            return Err(AppError::Validation(format!(
                "La tabla {table} contiene una fila sin columnas conocidas."
            )));
        }
        let statement = format!(
            "INSERT INTO {table} ({}) VALUES ({})",
            keys.iter()
                .map(|key| format!("\"{key}\""))
                .collect::<Vec<_>>()
                .join(","),
            vec!["?"; keys.len()].join(",")
        );
        let mut query = sqlx::query(&statement);
        for key in &keys {
            query = bind_value(query, object.get(key).expect("known key"))?;
        }
        query.execute(&mut **tx).await?;
    }
    Ok(())
}

async fn restore_database_payload(pool: &SqlitePool, payload: &Value) -> AppResult<()> {
    backup_summary(payload)?;
    let data = payload
        .get("data")
        .and_then(Value::as_object)
        .expect("validated data");
    // Read the current schema before taking the restore transaction. This also
    // lets a one-connection SQLite pool restore safely without waiting for a
    // second connection while the transaction is open.
    let mut current_columns = Vec::with_capacity(BACKUP_TABLES.len());
    for table in BACKUP_TABLES {
        current_columns.push((*table, table_columns(pool, table).await?));
    }
    let mut tx = pool.begin().await?;
    // The category catalog has a self-referential parent relation. Deferring
    // foreign keys keeps the replacement atomic while every parent/child row
    // is deleted and recreated inside this transaction.
    sqlx::query("PRAGMA defer_foreign_keys = ON")
        .execute(&mut *tx)
        .await?;
    for table in BACKUP_TABLES.iter().rev() {
        sqlx::query(&format!("DELETE FROM {table}"))
            .execute(&mut *tx)
            .await?;
    }
    for (table, columns) in &current_columns {
        insert_rows(
            &mut tx,
            table,
            columns,
            data.get(*table)
                .and_then(Value::as_array)
                .expect("validated table"),
        )
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

async fn restore_payload(pool: &SqlitePool, payload: &Value, app: &AppHandle) -> AppResult<()> {
    restore_database_payload(pool, payload).await?;
    let data = payload
        .get("data")
        .and_then(Value::as_object)
        .expect("validated data");
    if let Some(logo) = data.get("profileLogo") {
        let mime = logo
            .get("mime")
            .and_then(Value::as_str)
            .expect("validated logo");
        let encoded = logo
            .get("dataBase64")
            .and_then(Value::as_str)
            .expect("validated logo");
        let extension = if mime == "image/png" { "png" } else { "jpg" };
        let path = profile_dir(app)?.join(format!(
            "restored-logo-{}.{}",
            Utc::now().timestamp_millis(),
            extension
        ));
        fs::write(&path, decode_base64(encoded)?)?;
        sqlx::query("UPDATE professional_profile SET logo_path=? WHERE id=1")
            .bind(path.to_string_lossy().to_string())
            .execute(pool)
            .await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn restore_pricing_backup(
    content: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<RestoreResult, String> {
    async {
        let payload: Value = serde_json::from_str(&content).map_err(|_| {
            AppError::Validation("El archivo no es un backup JSON válido de Pricing OS.".into())
        })?;
        let summary = backup_summary(&payload)?;
        let automatic = backup_payload(&state.pool).await?;
        let timestamp = Utc::now().format("%Y-%m-%d-%H%M%S");
        let automatic_path = unused_path(
            export_dir(&app, "Backups")?,
            format!("before-restore-{timestamp}.json"),
        );
        fs::write(&automatic_path, serde_json::to_vec_pretty(&automatic)?)?;
        restore_payload(&state.pool, &payload, &app).await?;
        Ok(RestoreResult {
            automatic_backup_path: automatic_path.to_string_lossy().to_string(),
            summary,
        })
    }
    .await
    .map_err(command_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

    #[test]
    fn public_document_excludes_private_snapshot_data() {
        let snapshot = json!({"savedAt":"2026-08-11T12:00:00Z","quote":{"currency":"USD","notes":"el cliente suele negociar"},"project":{"name":"Proyecto reservado"},"client":{"name":"Acme"},"services":[{"id":"service-a","title":"Edición de video","finalSubtotalMinor":75000,"calculatedSubtotalMinor":50000,"suggestedSubtotalMinor":75000,"pricingSnapshot":{"cost":23000,"hourly":3500,"margin":520000},"sources":{"assigned":[{"name":"proveedor privado"}]}}],"totals":{"floorMinor":50000,"recommendedMinor":75000,"premiumMinor":100000,"selectedMinor":85000,"externalCostsMinor":23000,"effectiveHourlyMinor":3500,"marginMicros":520000}});
        let profile = ProfessionalProfile {
            display_name: "Estudio".into(),
            business_name: None,
            email: None,
            phone: None,
            website: None,
            location: None,
            logo_path: None,
            default_currency: "USD".into(),
            default_quote_validity_days: None,
            default_client_terms: None,
            document_theme: "light".into(),
            updated_at: "".into(),
            logo_data_url: None,
        };
        let document = document_from_snapshot(
            &snapshot,
            "PR-2026-0042".into(),
            &profile,
            &ClientDocumentConfig {
                quote_id: "quote-a".into(),
                snapshot_revision: None,
                presentation_mode: "itemized".into(),
                scope: None,
                revisions: None,
                estimated_timeline: None,
                client_notes: Some("Incluye hasta 2 rondas de correcciones.".into()),
                valid_until: None,
                service_descriptions: HashMap::new(),
            },
        )
        .expect("public document");
        assert_eq!(document.total_minor, 85_000);
        let rendered = serde_json::to_string(&document).expect("document json");
        for forbidden in [
            "proveedor privado",
            "el cliente suele negociar",
            "50000",
            "75000",
            "100000",
            "23000",
            "3500",
            "520000",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "private value leaked: {forbidden}"
            );
        }
    }

    #[test]
    fn pdf_is_a_real_document_with_the_selected_price() {
        let document = ClientQuoteDocument {
            quote_number: "PR-2026-0001".into(),
            issue_date: "11/08/2026".into(),
            valid_until: None,
            currency: "USD".into(),
            profile: ClientDocumentProfile {
                display_name: "Estudio".into(),
                business_name: None,
                email: None,
                phone: None,
                website: None,
                location: None,
                logo_data_url: None,
            },
            client_name: "Cliente".into(),
            project_name: "Proyecto".into(),
            presentation_mode: "global".into(),
            lines: vec![ClientQuoteLine {
                title: "Proyecto completo".into(),
                description: None,
                quantity: None,
                price_minor: 85_000,
            }],
            total_minor: 85_000,
            scope: None,
            revisions: None,
            estimated_timeline: None,
            client_notes: None,
            document_theme: "light".into(),
        };
        let pdf = document_pdf(&document).expect("pdf");
        if let Ok(path) = std::env::var("PRICING_OS_PDF_QA_PATH") {
            std::fs::write(path, &pdf).expect("write visual qa pdf");
        }
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(String::from_utf8_lossy(&pdf).contains("USD 850"));
    }

    #[test]
    fn requested_historical_revision_selects_its_own_snapshot() {
        tauri::async_runtime::block_on(async {
            let options = SqliteConnectOptions::from_str("sqlite::memory:")
                .expect("valid sqlite options")
                .create_if_missing(true)
                .foreign_keys(true);
            let pool = SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(options)
                .await
                .expect("database");
            sqlx::migrate!("./migrations")
                .run(&pool)
                .await
                .expect("migrations");

            let created_at = "2026-08-11T12:00:00Z";
            sqlx::query("INSERT INTO clients (id,name,status,created_at,updated_at) VALUES ('client-a','Cliente','active',?,?)")
                .bind(created_at)
                .bind(created_at)
                .execute(&pool)
                .await
                .expect("client");
            sqlx::query("INSERT INTO projects (id,client_id,name,currency,status,created_at,updated_at) VALUES ('project-a','client-a','Proyecto','USD','active',?,?)")
                .bind(created_at)
                .bind(created_at)
                .execute(&pool)
                .await
                .expect("project");
            sqlx::query("INSERT INTO quotes (id,project_id,version,status,currency,snapshot_revision,saved_at,created_at,updated_at) VALUES ('quote-a','project-a',1,'draft','USD',2,?,?,?)")
                .bind(created_at)
                .bind(created_at)
                .bind(created_at)
                .execute(&pool)
                .await
                .expect("quote");
            for (revision, total) in [(1_i64, 10_000_i64), (2_i64, 20_000_i64)] {
                let snapshot = json!({
                    "savedAt": created_at,
                    "quote": { "currency": "USD" },
                    "project": { "name": "Proyecto" },
                    "client": { "name": "Cliente" },
                    "services": [{ "id": "service-a", "title": "Edición", "finalSubtotalMinor": total }],
                    "totals": { "selectedMinor": total }
                });
                sqlx::query("INSERT INTO quote_snapshots (id,quote_id,revision,reason,project_name,client_name,currency,selected_price_kind,selected_price_minor,snapshot_json,created_at) VALUES (?,?,?,?,?,?,?,?,?,?,?)")
                    .bind(format!("snapshot-{revision}"))
                    .bind("quote-a")
                    .bind(revision)
                    .bind("manual_save")
                    .bind("Proyecto")
                    .bind("Cliente")
                    .bind("USD")
                    .bind("recommended")
                    .bind(total)
                    .bind(snapshot.to_string())
                    .bind(created_at)
                    .execute(&pool)
                    .await
                    .expect("snapshot");
            }

            let mut tx = pool.begin().await.expect("transaction");
            let (_, historical, _) = document_snapshot_row(&mut tx, "quote-a", Some(1))
                .await
                .expect("selected historical query")
                .expect("historical snapshot");
            let historical: Value = serde_json::from_str(&historical).expect("historical JSON");
            assert_eq!(
                historical["totals"]["selectedMinor"].as_i64(),
                Some(10_000),
                "an explicit revision must not fall back to the current snapshot"
            );
            tx.commit().await.expect("commit");

            let mut tx = pool.begin().await.expect("transaction");
            let (_, current, _) = document_snapshot_row(&mut tx, "quote-a", None)
                .await
                .expect("current query")
                .expect("current snapshot");
            let current: Value = serde_json::from_str(&current).expect("current JSON");
            assert_eq!(current["totals"]["selectedMinor"].as_i64(), Some(20_000));
        });
    }

    #[test]
    fn backup_restore_preserves_market_dependencies_and_user_classification_data() {
        tauri::async_runtime::block_on(async {
            let options = SqliteConnectOptions::from_str("sqlite::memory:")
                .expect("valid sqlite options")
                .create_if_missing(true)
                .foreign_keys(true);
            let pool = SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(options)
                .await
                .expect("database");
            sqlx::migrate!("./migrations")
                .run(&pool)
                .await
                .expect("migrations");

            let source_id: String =
                sqlx::query_scalar("SELECT id FROM market_sources WHERE system_key='bcra'")
                    .fetch_one(&pool)
                    .await
                    .expect("BCRA source");
            let category_id: String =
                sqlx::query_scalar("SELECT id FROM engine_categories ORDER BY id LIMIT 1")
                    .fetch_one(&pool)
                    .await
                    .expect("engine category");

            sqlx::query("INSERT INTO market_fetch_logs (id,source_id,url,method,started_at,finished_at,status,http_status,duration_ms,cache_hit,observation_count,error_type,error_message) VALUES ('backup-log',?,'https://api.bcra.gob.ar/test','GET','2026-08-11T00:00:00Z','2026-08-11T00:00:01Z','SUCCESS',200,100,0,1,NULL,NULL)")
                .bind(&source_id)
                .execute(&pool)
                .await
                .expect("market fetch log");
            sqlx::query("INSERT INTO market_fx_rates (id,source_id,base_currency,quote_currency,rate_micros,rate_date,source_url,retrieved_at) VALUES ('backup-fx',?,'USD','ARS',1350000000,'2026-08-11','https://api.bcra.gob.ar/test','2026-08-11T00:00:01Z')")
                .bind(&source_id)
                .execute(&pool)
                .await
                .expect("market FX rate");
            sqlx::query("INSERT INTO classification_aliases (id,normalized_term,engine_type,category_id,tags_json,origin,use_count,created_at,updated_at) VALUES ('backup-alias','servicio propio','service',?,'[\"propio\"]','user',1,'2026-08-11T00:00:00Z','2026-08-11T00:00:00Z')")
                .bind(&category_id)
                .execute(&pool)
                .await
                .expect("user classification alias");
            sqlx::query("INSERT INTO classification_runs (id,subject_type,subject_id,input_json,automatic_proposal_json,ai_proposal_json,final_proposal_json,ai_used,ai_model,status,created_at) VALUES ('backup-classification-run','engine','engine-video-editing','{}','{}',NULL,'{}',0,NULL,'success','2026-08-11T00:00:00Z')")
                .execute(&pool)
                .await
                .expect("classification audit");

            let payload = backup_payload(&pool).await.expect("backup payload");

            // A log created after the backup must not survive the replacement.
            // Its presence also proves restore deletes these RESTRICT children
            // before deleting and recreating market_sources.
            sqlx::query("INSERT INTO market_fetch_logs (id,source_id,url,method,started_at,finished_at,status,http_status,duration_ms,cache_hit,observation_count,error_type,error_message) VALUES ('after-backup-log',?,'https://api.bcra.gob.ar/after','GET','2026-08-11T00:01:00Z','2026-08-11T00:01:01Z','SUCCESS',200,100,0,1,NULL,NULL)")
                .bind(&source_id)
                .execute(&pool)
                .await
                .expect("later market fetch log");

            restore_database_payload(&pool, &payload)
                .await
                .expect("restore with market dependencies");

            let fetch_logs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM market_fetch_logs")
                .fetch_one(&pool)
                .await
                .expect("fetch log count");
            let fx_rates: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM market_fx_rates WHERE id='backup-fx'")
                    .fetch_one(&pool)
                    .await
                    .expect("FX rate count");
            let aliases: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM classification_aliases WHERE id='backup-alias'",
            )
            .fetch_one(&pool)
            .await
            .expect("alias count");
            let runs: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM classification_runs WHERE id='backup-classification-run'",
            )
            .fetch_one(&pool)
            .await
            .expect("classification run count");
            assert_eq!(fetch_logs, 1);
            assert_eq!(fx_rates, 1);
            assert_eq!(aliases, 1);
            assert_eq!(runs, 1);
        });
    }
}
