use rusqlite::params;
use tauri::State;

use crate::adapters::sqlite::activity_store::ActivityEntry;
use crate::app_state::AppState;

#[derive(serde::Serialize)]
pub struct OllamaStatus {
    pub connected: bool,
    pub models: Vec<String>,
}

#[derive(serde::Serialize)]
pub struct AppStats {
    pub total_documents: usize,
    pub total_memory_facts: usize,
    pub date_range: Option<(String, String)>,
}

#[tauri::command]
pub fn test_ollama_connection() -> Result<OllamaStatus, String> {
    // Run a quick synchronous HTTP check
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .map_err(|e| e.to_string())?;

    match client.get("http://localhost:11434/api/tags").send() {
        Ok(resp) if resp.status().is_success() => {
            let models: Vec<String> = resp
                .json::<serde_json::Value>()
                .ok()
                .and_then(|v| v.get("models")?.as_array().cloned())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|m| m.get("name")?.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            Ok(OllamaStatus {
                connected: true,
                models,
            })
        }
        _ => Ok(OllamaStatus {
            connected: false,
            models: vec![],
        }),
    }
}

#[tauri::command]
pub fn get_app_stats(
    state: State<'_, AppState>,
) -> Result<AppStats, String> {
    let months = state
        .timeline_store
        .get_document_count_by_month()
        .map_err(|e| e.to_string())?;

    let total_documents: usize = months.iter().map(|(_, c)| c).sum();

    let date_range = state
        .timeline_store
        .get_date_range()
        .map_err(|e| e.to_string())?
        .map(|r| (r.start.to_rfc3339(), r.end.to_rfc3339()));

    let total_facts = state
        .memory_store
        .get_all(None, None)
        .map_err(|e| e.to_string())?
        .len();

    Ok(AppStats {
        total_documents,
        total_memory_facts: total_facts,
        date_range,
    })
}

#[derive(serde::Serialize)]
pub struct UsageLogEntry {
    pub id: String,
    pub timestamp: String,
    pub provider: String,
    pub model: String,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub purpose: String,
    pub duration_ms: i64,
}

#[tauri::command]
pub fn get_usage_log(
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<UsageLogEntry>, String> {
    let limit = limit.unwrap_or(50) as i64;
    state
        .db
        .with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, timestamp, provider, model, prompt_tokens, completion_tokens, purpose, duration_ms
                 FROM llm_usage_log
                 ORDER BY timestamp DESC
                 LIMIT ?1",
            )?;
            let rows = stmt.query_map(params![limit], |row| {
                Ok(UsageLogEntry {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    provider: row.get(2)?,
                    model: row.get(3)?,
                    prompt_tokens: row.get(4)?,
                    completion_tokens: row.get(5)?,
                    purpose: row.get(6)?,
                    duration_ms: row.get(7)?,
                })
            })?;
            let mut entries = Vec::new();
            for row in rows {
                entries.push(row.map_err(|e| crate::error::AppError::Database(e))?);
            }
            Ok(entries)
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_activity_log(
    limit: Option<usize>,
    action_type: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<ActivityEntry>, String> {
    let limit = limit.unwrap_or(100);
    state
        .activity_store
        .get_recent(limit, action_type.as_deref())
        .map_err(|e| e.to_string())
}

/// Check whether the onboarding wizard has been completed.
#[tauri::command]
pub fn is_onboarding_complete(
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let val = state
        .config_store
        .get("onboarding.completed")
        .map_err(|e| e.to_string())?;
    Ok(val.as_deref() == Some("true"))
}

/// Mark the onboarding wizard as completed.
#[tauri::command]
pub fn complete_onboarding(
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .config_store
        .set("onboarding.completed", "true")
        .map_err(|e| e.to_string())
}
