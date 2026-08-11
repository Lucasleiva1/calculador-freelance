use std::{collections::HashMap, str::FromStr, sync::Arc};

use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    SqlitePool,
};
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

use crate::{error::AppResult, market::acquisition::http_client, models::MarketResearchJob};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub http: reqwest::Client,
    pub market_jobs: Arc<Mutex<HashMap<String, MarketResearchJob>>>,
    pub market_request_lock: Arc<Mutex<()>>,
}

pub async fn initialize(app: &AppHandle) -> AppResult<AppState> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::NotFound, error.to_string()))?;
    std::fs::create_dir_all(&data_dir)?;
    let db_path = data_dir.join("pricing-os.sqlite3");
    let connection = format!("sqlite://{}", db_path.to_string_lossy().replace('\\', "/"));
    let options = SqliteConnectOptions::from_str(&connection)?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(AppState {
        pool,
        http: http_client()?,
        market_jobs: Arc::new(Mutex::new(HashMap::new())),
        market_request_lock: Arc::new(Mutex::new(())),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_migration_creates_settings_presets_and_relations() {
        tauri::async_runtime::block_on(async {
            let options = SqliteConnectOptions::from_str("sqlite::memory:")
                .expect("valid options")
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
                .expect("migration");
            let settings: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM app_settings")
                .fetch_one(&pool)
                .await
                .expect("settings");
            let presets: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM service_presets WHERE origin = 'system'")
                    .fetch_one(&pool)
                    .await
                    .expect("presets");
            assert_eq!(settings, 1);
            assert_eq!(presets, 5);
            let definitions: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM service_definitions")
                .fetch_one(&pool)
                .await
                .expect("definitions");
            let parameters: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM service_parameters")
                .fetch_one(&pool)
                .await
                .expect("parameters");
            let sources: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM market_sources")
                .fetch_one(&pool)
                .await
                .expect("sources");
            let profiles: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM economic_profiles")
                .fetch_one(&pool)
                .await
                .expect("profiles");
            assert_eq!(definitions, 2);
            assert!(parameters >= 30);
            assert!(sources >= 40);
            assert_eq!(profiles, 2);
            let categories: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM engine_categories")
                .fetch_one(&pool)
                .await
                .expect("engine categories");
            let engines: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pricing_engines")
                .fetch_one(&pool)
                .await
                .expect("pricing engines");
            let ai_enabled: bool =
                sqlx::query_scalar("SELECT local_ai_enabled FROM app_settings WHERE id=1")
                    .fetch_one(&pool)
                    .await
                    .expect("local ai setting");
            assert!(categories >= 10);
            assert_eq!(engines, 2);
            assert!(!ai_enabled, "la IA local debe comenzar desactivada");
            let automatic_sources: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM market_sources WHERE acquisition_mode='auto_http' AND automation_status='APPROVED'",
            )
            .fetch_one(&pool)
            .await
            .expect("automatic sources");
            // El catálogo sólo automatiza BCRA para la conversión USD/ARS. Las
            // demás referencias requieren carga manual verificable para no
            // simular una fuente de precios ni infringir sus condiciones.
            assert_eq!(automatic_sources, 1);
            let automatic_key: String = sqlx::query_scalar(
                "SELECT system_key FROM market_sources WHERE acquisition_mode='auto_http' AND automation_status='APPROVED'",
            )
            .fetch_one(&pool)
            .await
            .expect("BCRA automatic source");
            assert_eq!(automatic_key, "bcra");
            let (upwork_url, upwork_status, upwork_automation): (String, String, String) =
                sqlx::query_as("SELECT base_url, current_status, automation_status FROM market_sources WHERE system_key='upwork'")
                    .fetch_one(&pool).await.expect("upwork");
            assert_eq!(upwork_url, "https://www.upwork.com/");
            assert_eq!(upwork_status, "DISABLED");
            assert_eq!(upwork_automation, "MANUAL_ONLY");

            let client = uuid::Uuid::new_v4().to_string();
            sqlx::query("INSERT INTO clients (id, name, status, created_at, updated_at) VALUES (?, 'ACME', 'active', 'now', 'now')")
                .bind(&client).execute(&pool).await.expect("client");
            for name in ["Campaña", "Contenido"] {
                sqlx::query("INSERT INTO projects (id, client_id, name, currency, market_scope, status, created_at, updated_at) VALUES (?, ?, ?, 'USD', 'international', 'active', 'now', 'now')")
                    .bind(uuid::Uuid::new_v4().to_string()).bind(&client).bind(name).execute(&pool).await.expect("project");
            }
            let project_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM projects WHERE client_id = ?")
                    .bind(client)
                    .fetch_one(&pool)
                    .await
                    .expect("count");
            assert_eq!(project_count, 2);
        });
    }

    #[test]
    fn pricing_configuration_persists_after_reopening_database() {
        tauri::async_runtime::block_on(async {
            let path =
                std::env::temp_dir().join(format!("pricing-os-{}.sqlite3", uuid::Uuid::new_v4()));
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
            sqlx::migrate!("./migrations")
                .run(&pool)
                .await
                .expect("migration");
            sqlx::query("UPDATE service_definitions SET balanced_margin_micros=275000, version=version+1 WHERE id='service-programming'")
                .execute(&pool).await.expect("update");
            sqlx::query("INSERT INTO pricing_engines (id,engine_key,name,engine_type,category_id,calculator_key,unit_kind,tags_json,status,classification_origin,created_at,updated_at) VALUES ('engine-shirts','venta-remeras','Venta de remeras','product','category-apparel','physical-product-v1','unit','[\"remeras\"]','active','automatic','now','now')")
                .execute(&pool).await.expect("product engine");
            for engine_id in ["engine-video-editing", "engine-shirts"] {
                sqlx::query("INSERT OR REPLACE INTO pricing_engine_sources (engine_id,source_id,role,preference,participates_in_suggestions,match_score_micros,assigned_by,created_at,updated_at) VALUES (?,'source-yunojuno','reference','available',1,800000,'manual','now','now')")
                    .bind(engine_id).execute(&pool).await.expect("shared source assignment");
            }
            pool.close().await;
            let reopened = SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(options)
                .await
                .expect("second connection");
            let saved: (Option<i64>, i64) = sqlx::query_as("SELECT balanced_margin_micros, version FROM service_definitions WHERE id='service-programming'")
                .fetch_one(&reopened).await.expect("saved definition");
            assert_eq!(saved, (Some(275_000), 2));
            let saved_engine: (String, String) = sqlx::query_as(
                "SELECT engine_type,calculator_key FROM pricing_engines WHERE id='engine-shirts'",
            )
            .fetch_one(&reopened)
            .await
            .expect("saved product engine");
            assert_eq!(
                saved_engine,
                ("product".into(), "physical-product-v1".into())
            );
            let shared_source_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM pricing_engine_sources WHERE source_id='source-yunojuno' AND engine_id IN ('engine-video-editing','engine-shirts')",
            )
            .fetch_one(&reopened)
            .await
            .expect("shared many-to-many source");
            assert_eq!(shared_source_count, 2);
            reopened.close().await;
            std::fs::remove_file(path).expect("remove test database");
        });
    }

    #[test]
    fn market_history_is_immutable_deduplicated_and_final_price_is_protected() {
        tauri::async_runtime::block_on(async {
            let options = SqliteConnectOptions::from_str("sqlite::memory:")
                .expect("valid options")
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
                .expect("migration");

            for id in ["observation-a", "observation-duplicate"] {
                sqlx::query("INSERT OR IGNORE INTO market_observations (id, source_id, origin, service_type, region, currency, price_type, unit, price_value_minor, original_value_text, source_type, source_url, retrieved_at, parser_version, confidence, comparison_eligibility, raw_fingerprint, created_at) VALUES (?, 'source-yunojuno', 'MANUAL', 'video-editing', 'INTERNATIONAL', 'USD', 'PROJECT', 'por proyecto', 60000, 'USD 600', 'rate_benchmark', 'https://www.yunojuno.com/', '2026-08-10T00:00:00Z', 'test', 'HIGH', 'ELIGIBLE', 'same-fingerprint', '2026-08-10T00:00:00Z')")
                    .bind(id).execute(&pool).await.expect("observation insert");
            }
            let unique_observations: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM market_observations WHERE raw_fingerprint='same-fingerprint'",
            )
            .fetch_one(&pool)
            .await
            .expect("dedup count");
            assert_eq!(unique_observations, 1);

            sqlx::query("INSERT INTO clients (id,name,status,created_at,updated_at) VALUES ('client-market','Cliente','active','now','now')").execute(&pool).await.expect("client");
            sqlx::query("INSERT INTO projects (id,client_id,name,currency,market_scope,status,created_at,updated_at) VALUES ('project-market','client-market','Proyecto','USD','international','active','now','now')").execute(&pool).await.expect("project");
            sqlx::query("INSERT INTO quotes (id,project_id,version,status,currency,created_at,updated_at) VALUES ('quote-market','project-market',1,'draft','USD','now','now')").execute(&pool).await.expect("quote");
            sqlx::query("INSERT INTO quote_services (id,quote_id,service_type,title,sort_order,configuration_version,configuration_json,calculated_subtotal_minor,suggested_subtotal_minor,final_subtotal_minor,has_override,manual_subtotal_minor,row_revision,created_at,updated_at) VALUES ('service-market','quote-market','video-editing','Video',0,1,'{}',54000,65000,72000,1,72000,0,'now','now')").execute(&pool).await.expect("service");

            sqlx::query("INSERT INTO market_snapshots (id,quote_id,quote_service_id,query_context_json,currency,observation_count,comparable_observation_count,source_count,minimum_filtered_minor,p25_minor,market_median_minor,p75_minor,maximum_filtered_minor,confidence_level,calculated_price_minor,suggested_price_minor,final_price_minor_at_creation,summary_json,created_at) VALUES ('snapshot-old','quote-market','service-market','{}','USD',1,1,1,58000,60000,67000,79000,90000,'LOW',54000,65000,72000,'{}','2026-08-10T00:00:00Z')").execute(&pool).await.expect("old snapshot");
            sqlx::query("INSERT INTO market_snapshots (id,quote_id,quote_service_id,query_context_json,currency,observation_count,comparable_observation_count,source_count,minimum_filtered_minor,p25_minor,market_median_minor,p75_minor,maximum_filtered_minor,confidence_level,calculated_price_minor,suggested_price_minor,final_price_minor_at_creation,summary_json,created_at) VALUES ('snapshot-new','quote-market','service-market','{}','USD',2,2,1,65000,68000,72000,82000,95000,'LOW',54000,68000,72000,'{}','2026-08-11T00:00:00Z')").execute(&pool).await.expect("new snapshot");
            let old_median: i64 = sqlx::query_scalar(
                "SELECT market_median_minor FROM market_snapshots WHERE id='snapshot-old'",
            )
            .fetch_one(&pool)
            .await
            .expect("old median");
            assert_eq!(old_median, 67_000);

            sqlx::query("UPDATE quote_services SET suggested_subtotal_minor=68000 WHERE id='service-market'").execute(&pool).await.expect("suggestion update");
            let protected: (Option<i64>, bool, Option<i64>) = sqlx::query_as("SELECT final_subtotal_minor, has_override, manual_subtotal_minor FROM quote_services WHERE id='service-market'").fetch_one(&pool).await.expect("protected final");
            assert_eq!(protected, (Some(72_000), true, Some(72_000)));
        });
    }
}
