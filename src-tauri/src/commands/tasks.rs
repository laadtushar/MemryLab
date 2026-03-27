use tauri::{AppHandle, Manager};

use crate::services::task_manager::{TaskManager, TaskRecord};

#[tauri::command]
pub fn cancel_task(task_id: String, app_handle: AppHandle) -> Result<bool, String> {
    let mgr = app_handle.state::<TaskManager>();
    let cancelled = mgr.cancel_task(&task_id);
    if cancelled {
        mgr.mark_cancelled(&task_id);
    }
    Ok(cancelled)
}

#[tauri::command]
pub fn get_interrupted_tasks(app_handle: AppHandle) -> Result<Vec<TaskRecord>, String> {
    let mgr = app_handle.state::<TaskManager>();
    Ok(mgr.get_interrupted_tasks())
}
