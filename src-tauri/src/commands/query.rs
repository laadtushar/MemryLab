use tauri::State;

use crate::app_state::AppState;
use crate::query::rag_pipeline::{self, RagResponse};

#[tauri::command]
pub async fn ask(
    query: String,
    state: State<'_, AppState>,
) -> Result<RagResponse, String> {
    tracing::info!(query_len = query.len(), "RAG query received");
    let start = std::time::Instant::now();

    let doc_store = state.document_store.clone();
    let vec_store = state.vector_store.clone();
    let page_idx = state.page_index.clone();
    let mem_store = state.memory_store.clone();
    let llm_lock = state.llm_provider.clone();
    let embed_lock = state.embedding_provider.clone();
    let q = query.clone();

    let result = tokio::task::spawn_blocking(move || {
        let llm = llm_lock.read().map_err(|e| format!("Lock error: {}", e))?;
        let embed = embed_lock.read().map_err(|e| format!("Lock error: {}", e))?;

        tokio::runtime::Handle::current().block_on(rag_pipeline::query_rag(
            &q,
            doc_store.as_ref(),
            vec_store.as_ref(),
            page_idx.as_ref(),
            mem_store.as_ref(),
            embed.as_ref(),
            llm.as_ref(),
            5,
        ))
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?;

    match &result {
        Ok(r) => tracing::info!(
            sources = r.sources.len(),
            answer_len = r.answer.len(),
            duration_ms = start.elapsed().as_millis() as u64,
            "RAG query complete"
        ),
        Err(e) => tracing::error!(error = %e, "RAG query failed"),
    }

    result
}
