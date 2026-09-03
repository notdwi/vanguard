use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::Result;
use crate::state::{AppState, DEFAULT_PROXY_PORT};

const KEY: &str = "app_settings";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub language: String,
    pub proxy_port: u16,
    pub mask_secrets: bool,
    pub timeline_page_size: i64,
    pub auto_analyse: bool,
    pub default_replay_delay_ms: u64,
    pub sensitive_headers: Vec<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "en".into(),
            proxy_port: DEFAULT_PROXY_PORT,
            mask_secrets: true,
            timeline_page_size: 500,
            auto_analyse: true,
            default_replay_delay_ms: 0,
            sensitive_headers: crate::http::headers::SENSITIVE
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<AppSettings> {
    Ok(match state.db.get_setting(KEY)? {
        Some(raw) => serde_json::from_str(&raw).unwrap_or_default(),
        None => AppSettings::default(),
    })
}

#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: AppSettings) -> Result<AppSettings> {
    state.db.set_setting(KEY, &serde_json::to_string(&settings)?)?;
    Ok(settings)
}

#[derive(Serialize)]
pub struct StorageInfo {
    pub data_dir: String,
    pub database_bytes: u64,
    pub blob_bytes: u64,
}

#[tauri::command]
pub fn storage_info(state: State<'_, AppState>) -> StorageInfo {
    let db_path = state.root.join("vanguard.db");
    StorageInfo {
        data_dir: state.root.to_string_lossy().into_owned(),
        database_bytes: std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0),
        blob_bytes: dir_size(&state.db.blob_root()),
    }
}

fn dir_size(path: &std::path::Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(path) else { return 0 };
    entries
        .flatten()
        .map(|e| match e.file_type() {
            Ok(t) if t.is_dir() => dir_size(&e.path()),
            Ok(_) => e.metadata().map(|m| m.len()).unwrap_or(0),
            Err(_) => 0,
        })
        .sum()
}
