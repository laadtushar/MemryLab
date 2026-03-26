use tauri::State;

use crate::adapters::sqlite::activity_store::ActivityEntry;
use crate::app_state::AppState;
use crate::query::rag_pipeline;

#[derive(Debug, Clone, serde::Serialize)]
pub struct AskResponse {
    pub answer: String,
    pub sources: Vec<crate::query::rag_pipeline::RagSource>,
    pub conversation_id: Option<String>,
}

#[tauri::command]
pub fn ask(
    query: String,
    conversation_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<AskResponse, String> {
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
    })?;

    let duration_ms = start.elapsed().as_millis() as u64;
    tracing::info!(
        sources = result.sources.len(),
        answer_len = result.answer.len(),
        duration_ms = duration_ms,
        "RAG query complete"
    );

    // Persist to chat history
    let conv_id = if let Some(cid) = conversation_id {
        // Existing conversation
        let _ = state.chat_store.add_message(&cid, "user", &query, &[]);
        let _ = state.chat_store.add_message(&cid, "assistant", &result.answer, &result.sources);
        Some(cid)
    } else {
        // Create new conversation with first few words as title
        let title: String = query.chars().take(60).collect();
        match state.chat_store.create_conversation(&title) {
            Ok(conv) => {
                let _ = state.chat_store.add_message(&conv.id, "user", &query, &[]);
                let _ = state.chat_store.add_message(&conv.id, "assistant", &result.answer, &result.sources);
                Some(conv.id)
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to create conversation");
                None
            }
        }
    };

    // Log activity
    let truncated_query: String = query.chars().take(100).collect();
    let _ = state.activity_store.log_activity(&ActivityEntry {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        action_type: "ask".to_string(),
        title: truncated_query,
        description: String::new(),
        result_summary: format!("{} sources", result.sources.len()),
        metadata: serde_json::json!({}),
        duration_ms: duration_ms as i64,
        status: "success".to_string(),
    });

    Ok(AskResponse {
        answer: result.answer,
        sources: result.sources,
        conversation_id: conv_id,
    })
}
