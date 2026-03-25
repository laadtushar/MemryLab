use tauri::State;

use crate::app_state::AppState;
use crate::domain::models::common::TimeGranularity;
use crate::pipeline::analysis::orchestrator::{self, AnalysisConfig, AnalysisResult};

#[tauri::command]
pub fn run_analysis(
    granularity: Option<String>,
    state: State<'_, AppState>,
) -> Result<AnalysisResult, String> {
    let llm = state
        .llm_provider
        .read()
        .map_err(|e| format!("Lock error: {}", e))?;

    let config = AnalysisConfig {
        granularity: TimeGranularity::from_str_opt(granularity.as_deref()),
    };

    tauri::async_runtime::block_on(orchestrator::run_analysis(
        state.document_store.as_ref(),
        state.timeline_store.as_ref(),
        state.memory_store.as_ref(),
        state.graph_store.as_ref(),
        llm.as_ref(),
        Some(config),
    ))
    .map_err(|e| e.to_string())
}
