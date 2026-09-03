use tauri::State;

use crate::error::Result;
use crate::state::AppState;
use crate::tls::{ca, install};

#[tauri::command]
pub fn ca_info(state: State<'_, AppState>) -> ca::CaInfo {
    let root = state.ca_root();
    let installed = install::is_installed(&ca::cert_path(&root));
    ca::info(&root, installed)
}

/// Describes exactly which commands the install button will run, so the user
/// can read them before anything touches the system trust store.
#[tauri::command]
pub fn ca_plan(state: State<'_, AppState>) -> install::TrustStorePlan {
    install::plan(&ca::cert_path(&state.ca_root()))
}

#[tauri::command]
pub fn generate_ca(state: State<'_, AppState>) -> Result<ca::CaInfo> {
    let root = state.ca_root();
    ca::generate(&root)?;
    Ok(ca::info(&root, install::is_installed(&ca::cert_path(&root))))
}

#[tauri::command]
pub fn install_ca(state: State<'_, AppState>) -> Result<ca::CaInfo> {
    let root = state.ca_root();
    if !ca::exists(&root) {
        ca::generate(&root)?;
    }
    install::install(&ca::cert_path(&root))?;
    Ok(ca::info(&root, install::is_installed(&ca::cert_path(&root))))
}

#[tauri::command]
pub fn uninstall_ca(state: State<'_, AppState>) -> Result<ca::CaInfo> {
    let root = state.ca_root();
    install::uninstall(&ca::cert_path(&root))?;
    Ok(ca::info(&root, install::is_installed(&ca::cert_path(&root))))
}

#[tauri::command]
pub fn delete_ca(state: State<'_, AppState>) -> Result<ca::CaInfo> {
    let root = state.ca_root();
    ca::delete(&root)?;
    Ok(ca::info(&root, false))
}

#[tauri::command]
pub fn export_ca(state: State<'_, AppState>, destination: String) -> Result<String> {
    let root = state.ca_root();
    let source = ca::cert_path(&root);
    if !source.exists() {
        ca::generate(&root)?;
    }
    std::fs::copy(&source, &destination)?;
    Ok(destination)
}
