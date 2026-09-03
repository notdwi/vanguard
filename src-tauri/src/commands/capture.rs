use std::net::SocketAddr;

use tauri::{AppHandle, State};

use crate::capture::engine::Engine;
use crate::error::{AppError, Result};
use crate::events;
use crate::models::{CaptureConfig, CaptureState, CaptureStatus};
use crate::proxy::{launcher, server};
use crate::state::{Active, AppState, DEFAULT_PROXY_PORT};
use crate::storage::sessions;
use crate::tls::ca;

#[tauri::command]
pub fn capture_status(state: State<'_, AppState>) -> CaptureStatus {
    state.status()
}

#[tauri::command]
pub async fn start_capture(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    port: Option<u16>,
) -> Result<CaptureStatus> {
    if state.engine().is_some() {
        return Err(AppError::Invalid(
            "a capture is already running; stop it before starting another".into(),
        ));
    }

    let session = sessions::get(&state.db, &session_id)?;
    let ca_files = ca::load_or_generate(&state.ca_root())?;
    let start_sequence = sessions::next_sequence(&state.db, &session_id)?;

    let engine = Engine::start(
        state.db.clone(),
        std::sync::Arc::new(app.clone()) as std::sync::Arc<dyn crate::events::EventSink>,
        session_id.clone(),
        session.config.clone(),
        start_sequence,
    );

    let addr = SocketAddr::from(([127, 0, 0, 1], port.unwrap_or(DEFAULT_PROXY_PORT)));
    let proxy = server::spawn(addr, &ca_files, engine.clone()).await?;

    state.set_active(Active { engine, proxy });
    sessions::set_status(&state.db, &session_id, CaptureState::Capturing)?;
    state.invalidate_analysis();

    let status = state.status();
    events::capture_status(&app, &status);
    Ok(status)
}

#[tauri::command]
pub fn stop_capture(app: AppHandle, state: State<'_, AppState>) -> CaptureStatus {
    state.stop_active();
    state.invalidate_analysis();
    let status = state.status();
    events::capture_status(&app, &status);
    events::sessions_changed(&app);
    status
}

#[tauri::command]
pub fn pause_capture(
    app: AppHandle,
    state: State<'_, AppState>,
    paused: bool,
) -> Result<CaptureStatus> {
    let engine = state.require_engine()?;
    engine.set_paused(paused);
    let status = state.status();
    let session_state = if paused { CaptureState::Paused } else { CaptureState::Capturing };
    sessions::set_status(&state.db, engine.session_id(), session_state)?;
    events::capture_status(&app, &status);
    Ok(status)
}

#[tauri::command]
pub fn update_capture_config(
    state: State<'_, AppState>,
    session_id: String,
    config: CaptureConfig,
) -> Result<()> {
    sessions::set_config(&state.db, &session_id, &config)?;
    Ok(())
}

#[tauri::command]
pub fn flush_counters(app: AppHandle, state: State<'_, AppState>) -> CaptureStatus {
    if let Some(engine) = state.engine() {
        engine.persist_counters();
    }
    let status = state.status();
    events::emit(
        &app,
        events::COUNTERS,
        events::Counters {
            session_id: status.session_id.clone().unwrap_or_default(),
            captured: status.captured,
            ignored: status.ignored,
        },
    );
    status
}

#[tauri::command]
pub fn list_browsers() -> Vec<launcher::BrowserOption> {
    launcher::available()
}

#[tauri::command]
pub fn launch_browser(
    state: State<'_, AppState>,
    browser_id: String,
    url: Option<String>,
) -> Result<()> {
    let addr = state
        .proxy_addr()
        .ok_or_else(|| AppError::Invalid("start a capture before launching a browser".into()))?;
    launcher::launch(
        &browser_id,
        &addr.to_string(),
        &state.root,
        &crate::tls::ca::cert_path(&state.ca_root()),
        url.as_deref().unwrap_or_default(),
    )
}

#[tauri::command]
pub fn clear_browser_profile(state: State<'_, AppState>, browser_id: String) -> Result<()> {
    launcher::clear_profile(&state.root, &browser_id)
}
