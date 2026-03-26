use tauri::State;

use crate::app_state::AppState;
use crate::adapters::sqlite::prompt_store::PromptVersion;

#[derive(serde::Serialize)]
pub struct PromptVersionResponse {
    pub id: String,
    pub name: String,
    pub version: String,
    pub template: String,
    pub is_active: bool,
    pub created_at: String,
}

impl From<PromptVersion> for PromptVersionResponse {
    fn from(p: PromptVersion) -> Self {
        Self {
            id: p.id,
            name: p.name,
            version: p.version,
            template: p.template,
            is_active: p.is_active,
            created_at: p.created_at,
        }
    }
}

#[tauri::command]
pub fn list_prompts(state: State<'_, AppState>) -> Result<Vec<PromptVersionResponse>, String> {
    state
        .prompt_store
        .list_all()
        .map(|v| v.into_iter().map(PromptVersionResponse::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_prompt(
    name: String,
    version: String,
    template: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let prompt = PromptVersion {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.clone(),
        version: version.clone(),
        template,
        is_active: false,
        created_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
    };
    state
        .prompt_store
        .save(&prompt)
        .map_err(|e| e.to_string())?;

    // Auto-activate the new version
    state
        .prompt_store
        .set_active(&name, &version)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_active_prompt(
    name: String,
    version: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .prompt_store
        .set_active(&name, &version)
        .map_err(|e| e.to_string())
}
