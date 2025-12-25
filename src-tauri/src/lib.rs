pub mod commands;
pub mod logging;
pub mod models;
pub mod ping;
pub mod state;
pub mod stats;

use state::AppState;
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .format_timestamp_millis()
        .init();

    log::info!("Starting Ping Connectivity Monitor");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(Arc::new(AppState::new()))
        .invoke_handler(tauri::generate_handler![
            commands::start_pinging,
            commands::stop_pinging,
            commands::pause_pinging,
            commands::resume_pinging,
            commands::get_ping_state,
            commands::get_statistics,
            commands::get_statistics_for_target,
            commands::get_recent_pings,
            commands::get_log_path,
            commands::set_ping_interval,
            commands::get_targets,
            commands::add_target,
            commands::remove_target,
            commands::update_target,
            commands::toggle_target,
            commands::get_config,
            commands::save_config,
            commands::get_preset_targets,
            commands::reset_statistics,
            commands::open_log_directory,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
