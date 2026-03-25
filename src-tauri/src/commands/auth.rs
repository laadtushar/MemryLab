use tauri::{AppHandle, Manager};
use crate::app_state::AppState;

/// Check if this is the first run (no database exists yet).
#[tauri::command]
pub fn is_first_run(app_handle: AppHandle) -> Result<bool, String> {
    let data_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let db_path = data_dir.join("memory_palace.db");
    Ok(!db_path.exists())
}

/// Check if the database is locked (needs passphrase).
#[tauri::command]
pub fn is_database_locked(app_handle: AppHandle) -> Result<bool, String> {
    // If AppState is already managed, database is unlocked
    Ok(app_handle.try_state::<AppState>().is_none())
}

/// Unlock the database with a passphrase and initialize AppState.
#[tauri::command]
pub fn unlock_database(passphrase: String, app_handle: AppHandle) -> Result<(), String> {
    let data_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;

    let state = AppState::new_with_passphrase(data_dir, &passphrase)
        .map_err(|e| format!("Failed to unlock database: {}", e))?;

    app_handle.manage(state);
    Ok(())
}

/// Set passphrase on first run (creates new encrypted database).
#[tauri::command]
pub fn set_passphrase(passphrase: String, app_handle: AppHandle) -> Result<(), String> {
    let data_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;

    let state = AppState::new_with_passphrase(data_dir, &passphrase)
        .map_err(|e| format!("Failed to create database: {}", e))?;

    app_handle.manage(state);
    Ok(())
}
