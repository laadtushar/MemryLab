use tauri::{AppHandle, Manager};

use crate::services::folder_watcher::{FolderWatcherService, WatchedFolder};

#[tauri::command]
pub fn add_watch_folder(
    path: String,
    adapter_id: Option<String>,
    import_id: Option<String>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let watcher = app_handle.state::<FolderWatcherService>();
    watcher.watch_folder_with_id(&path, adapter_id.as_deref(), import_id)
}

#[tauri::command]
pub fn remove_watch_folder(
    path: String,
    app_handle: AppHandle,
) -> Result<(), String> {
    let watcher = app_handle.state::<FolderWatcherService>();
    watcher.unwatch_folder(&path)
}

#[tauri::command]
pub fn list_watch_folders(
    app_handle: AppHandle,
) -> Result<Vec<WatchedFolder>, String> {
    let watcher = app_handle.state::<FolderWatcherService>();
    Ok(watcher.get_saved_folders())
}
