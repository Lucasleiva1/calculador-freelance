mod classification;
mod commands;
mod db;
mod economy_import;
mod error;
mod file_exports;
mod history;
mod market;
mod models;
mod phase6;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let state = tauri::async_runtime::block_on(db::initialize(app.handle()))
                .map_err(|error| Box::<dyn std::error::Error>::from(error.to_string()))?;
            app.manage(state);
            if let Some(window) = app.get_webview_window("main") {
                window.show()?;
                window.unminimize()?;
                window.set_focus()?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::exit_application,
            commands::bootstrap_app,
            commands::load_workspace,
            commands::create_project,
            commands::save_client,
            commands::set_client_archived,
            commands::set_project_archived,
            commands::add_quote_service,
            commands::save_quote_service,
            commands::set_service_deleted,
            commands::reorder_quote_services,
            commands::update_settings,
            commands::change_project_currency,
            commands::save_preset,
            commands::delete_user_preset,
            commands::restore_system_preset,
            commands::load_pricing_configuration,
            commands::save_service_definition,
            commands::save_service_parameter,
            commands::delete_service_parameter,
            commands::save_parameter_option,
            commands::delete_parameter_option,
            commands::save_pricing_rule,
            commands::delete_pricing_rule,
            commands::save_economic_profile,
            economy_import::extract_economy_pdf_text,
            file_exports::save_economy_template,
            commands::save_market_source,
            commands::delete_market_source,
            commands::restore_market_source,
            commands::restore_market_sources_catalog,
            commands::test_market_source,
            commands::approve_market_source,
            commands::refresh_market_source,
            commands::save_manual_market_observation,
            commands::list_market_observations,
            commands::list_market_snapshots,
            commands::get_market_overview,
            commands::start_market_research,
            commands::get_market_research_job,
            commands::cancel_market_research,
            commands::open_market_source,
            history::list_quote_history,
            history::get_quote_history,
            history::save_quote_snapshot,
            history::update_quote_admin,
            history::duplicate_quote,
            history::delete_quote_permanently,
            phase6::get_professional_profile,
            phase6::save_professional_profile,
            phase6::get_client_document_config,
            phase6::save_client_document_config,
            phase6::create_client_quote_document,
            phase6::export_client_quote_pdf,
            phase6::create_pricing_backup,
            phase6::inspect_pricing_backup,
            phase6::restore_pricing_backup,
            classification::classify_pricing_engine,
            classification::classify_market_source,
            classification::test_ollama,
            classification::save_pricing_engine,
            classification::set_pricing_engine_archived,
            classification::save_engine_source,
            classification::remove_engine_source,
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar Pricing OS");
}
