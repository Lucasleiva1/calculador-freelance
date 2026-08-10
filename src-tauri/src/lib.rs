mod commands;
mod db;
mod error;
mod models;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let state = tauri::async_runtime::block_on(db::initialize(app.handle()))
                .map_err(|error| Box::<dyn std::error::Error>::from(error.to_string()))?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
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
            commands::save_market_source,
            commands::delete_market_source,
            commands::restore_market_source,
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar Pricing OS");
}
