pub mod analyzer;
pub mod capture;
pub mod commands;
pub mod error;
pub mod events;
pub mod http;
pub mod models;
pub mod proxy;
pub mod repeater;
pub mod state;
pub mod storage;
pub mod tls;

use tauri::Manager;

use state::AppState;

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("VANGUARD_LOG")
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn,vanguard=info")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let state = AppState::new(&app.handle().clone())?;
            app.manage(state);
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if let Some(state) = window.app_handle().try_state::<AppState>() {
                    state.stop_active();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::capture::capture_status,
            commands::capture::start_capture,
            commands::capture::stop_capture,
            commands::capture::pause_capture,
            commands::capture::update_capture_config,
            commands::capture::flush_counters,
            commands::capture::list_browsers,
            commands::capture::launch_browser,
            commands::capture::clear_browser_profile,
            commands::sessions::list_sessions,
            commands::sessions::get_session,
            commands::sessions::create_session,
            commands::sessions::rename_session,
            commands::sessions::delete_session,
            commands::sessions::clear_session,
            commands::sessions::session_hosts,
            commands::requests::timeline,
            commands::requests::request_detail,
            commands::requests::load_body,
            commands::requests::copy_as_curl,
            commands::requests::mask_header_value,
            commands::repeater::list_drafts,
            commands::repeater::get_draft,
            commands::repeater::send_to_repeater,
            commands::repeater::new_draft,
            commands::repeater::save_draft,
            commands::repeater::delete_draft,
            commands::repeater::run_replay,
            commands::repeater::list_replays,
            commands::repeater::clear_replays,
            commands::repeater::draft_as_curl,
            commands::repeater::compare_responses,
            commands::analysis::analyse_session,
            commands::analysis::analyse_request,
            commands::analysis::endpoint_requests,
            commands::ca::ca_info,
            commands::ca::ca_plan,
            commands::ca::generate_ca,
            commands::ca::install_ca,
            commands::ca::uninstall_ca,
            commands::ca::delete_ca,
            commands::ca::export_ca,
            commands::har::import_har,
            commands::har::export_har,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::settings::storage_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running vanguard");
}
