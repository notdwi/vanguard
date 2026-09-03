use tauri::{AppHandle, State};

use crate::error::Result;
use crate::events;
use crate::models::{CaptureConfig, Session, SessionSummary};
use crate::state::AppState;
use crate::storage::{queries, sessions};

#[tauri::command]
pub fn list_sessions(state: State<'_, AppState>) -> Result<Vec<SessionSummary>> {
    sessions::list(&state.db)
}

#[tauri::command]
pub fn get_session(state: State<'_, AppState>, session_id: String) -> Result<Session> {
    sessions::get(&state.db, &session_id)
}

#[tauri::command]
pub fn create_session(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
    config: Option<CaptureConfig>,
) -> Result<Session> {
    let name = if name.trim().is_empty() { default_name() } else { name };
    let session = sessions::create(&state.db, &name, &config.unwrap_or_default())?;
    events::sessions_changed(&app);
    Ok(session)
}

#[tauri::command]
pub fn rename_session(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    name: String,
) -> Result<()> {
    sessions::rename(&state.db, &session_id, &name)?;
    events::sessions_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn delete_session(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
) -> Result<()> {
    if state.engine().map(|e| e.session_id() == session_id).unwrap_or(false) {
        state.stop_active();
    }
    sessions::delete(&state.db, &session_id)?;
    state.invalidate_analysis();
    events::sessions_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn clear_session(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
) -> Result<()> {
    sessions::clear_requests(&state.db, &session_id)?;
    state.invalidate_analysis();
    events::sessions_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn session_hosts(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<HostCount>> {
    Ok(queries::hosts(&state.db, &session_id)?
        .into_iter()
        .map(|(host, count)| HostCount { host, count })
        .collect())
}

#[derive(serde::Serialize)]
pub struct HostCount {
    pub host: String,
    pub count: i64,
}

fn default_name() -> String {
    format!("Session {}", chrono::Local::now().format("%d %b %H:%M"))
}
