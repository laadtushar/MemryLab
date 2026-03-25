use tauri::State;

use crate::app_state::AppState;
use crate::domain::models::common::TimeBoundary;

#[tauri::command]
pub fn list_boundaries(state: State<'_, AppState>) -> Result<Vec<TimeBoundary>, String> {
    let cs = &state.config_store;
    let entries = cs.get_by_prefix("boundary.").map_err(|e| e.to_string())?;
    let mut boundaries = Vec::new();
    for (_key, value) in entries {
        if let Ok(b) = serde_json::from_str::<TimeBoundary>(&value) {
            boundaries.push(b);
        }
    }
    boundaries.sort_by(|a, b| a.date.cmp(&b.date));
    Ok(boundaries)
}

#[tauri::command]
pub fn add_boundary(
    name: String,
    date: String,
    end_date: Option<String>,
    color: Option<String>,
    state: State<'_, AppState>,
) -> Result<TimeBoundary, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let boundary = TimeBoundary {
        id: id.clone(),
        name,
        date,
        end_date,
        color,
    };
    let json = serde_json::to_string(&boundary).map_err(|e| e.to_string())?;
    state
        .config_store
        .set(&format!("boundary.{}", id), &json)
        .map_err(|e| e.to_string())?;
    Ok(boundary)
}

#[tauri::command]
pub fn delete_boundary(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state
        .config_store
        .delete(&format!("boundary.{}", id))
        .map_err(|e| e.to_string())?;
    Ok(())
}
