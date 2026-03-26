use tauri::State;

use crate::app_state::AppState;
use crate::query::rag_pipeline::{self, RagResponse};

#[tauri::command]
pub fn ask(
    query: String,
    state: State<'_, AppState>,
) -> Result<RagResponse, String> {
    tracing::info!(query_len = query.len(), "RAG query received");
    let start = std::time::Instant::now();

    let llm = state
        .llm_provider
        .read()
        .map_err(|e| format!("Lock error: {}", e))?;

    let embedding = state
        .embedding_provider
        .read()
        .map_err(|e| format!("Lock error: {}", e))?;

    let result = tauri::async_runtime::block_on(rag_pipeline::query_rag(
        &query,
        state.document_store.as_ref(),
        state.vector_store.as_ref(),
        state.page_index.as_ref(),
        state.memory_store.as_ref(),
        embedding.as_ref(),
        llm.as_ref(),
        5,
    ))
    .map_err(|e| {
        tracing::error!(error = %e, "RAG query failed");
        e.to_string()
    });

    if let Ok(ref r) = result {
        tracing::info!(
            sources = r.sources.len(),
            answer_len = r.answer.len(),
            duration_ms = start.elapsed().as_millis() as u64,
            "RAG query complete"
        );
    }

    result
}
