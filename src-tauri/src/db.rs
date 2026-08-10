use std::str::FromStr;

use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    SqlitePool,
};
use tauri::{AppHandle, Manager};

use crate::error::AppResult;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
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
    Ok(AppState { pool })
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
            pool.close().await;
            let reopened = SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(options)
                .await
                .expect("second connection");
            let saved: (Option<i64>, i64) = sqlx::query_as("SELECT balanced_margin_micros, version FROM service_definitions WHERE id='service-programming'")
                .fetch_one(&reopened).await.expect("saved definition");
            assert_eq!(saved, (Some(275_000), 2));
            reopened.close().await;
            std::fs::remove_file(path).expect("remove test database");
        });
    }
}
