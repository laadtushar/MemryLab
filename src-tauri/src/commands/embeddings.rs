use std::collections::HashMap;

use tauri::State;

use crate::app_state::AppState;

#[derive(serde::Serialize)]
pub struct EmbeddingResult {
    pub chunks_processed: usize,
    pub embeddings_generated: usize,
    pub already_embedded: usize,
    pub errors: Vec<String>,
}

/// Generate embeddings for all chunks that don't have them yet.
#[tauri::command]
pub fn generate_embeddings(
    state: State<'_, AppState>,
) -> Result<EmbeddingResult, String> {
    let provider = state
        .embedding_provider
        .read()
        .map_err(|e| format!("Lock error: {}", e))?;

    // Get all document IDs
    let months = state
        .timeline_store
        .get_document_count_by_month()
        .map_err(|e| e.to_string())?;

    if months.is_empty() {
        return Ok(EmbeddingResult {
            chunks_processed: 0,
            embeddings_generated: 0,
            already_embedded: 0,
            errors: vec![],
        });
    }

    let date_range = state
        .timeline_store
        .get_date_range()
        .map_err(|e| e.to_string())?
        .ok_or("No documents found")?;

    let doc_ids = state
        .timeline_store
        .get_documents_in_range(&date_range)
        .map_err(|e| e.to_string())?;

    let mut all_chunks: Vec<(String, String)> = Vec::new();
    for doc_id in &doc_ids {
        let chunks = state
            .document_store
            .get_chunks_by_document(doc_id)
            .map_err(|e| e.to_string())?;
        for chunk in chunks {
            all_chunks.push((chunk.id, chunk.text));
        }
    }

    let total = all_chunks.len();

    // Check which chunks already have embeddings by attempting a search
    // (Simple heuristic: try to embed all, vector store upsert is idempotent)
    let mut generated = 0;
    let mut errors = Vec::new();
    let batch_size = 10;

    for batch in all_chunks.chunks(batch_size) {
        let texts: Vec<String> = batch.iter().map(|(_, t)| t.clone()).collect();
        let ids: Vec<&str> = batch.iter().map(|(id, _)| id.as_str()).collect();

        match tauri::async_runtime::block_on(provider.embed_batch(&texts)) {
            Ok(embeddings) => {
                let items: Vec<(String, Vec<f32>, HashMap<String, String>)> = ids
                    .iter()
                    .zip(embeddings.into_iter())
                    .map(|(id, vec)| (id.to_string(), vec, HashMap::new()))
                    .collect();

                if let Err(e) = state.vector_store.upsert_batch(&items) {
                    errors.push(format!("Store batch failed: {}", e));
                } else {
                    generated += items.len();
                }
            }
            Err(e) => {
                errors.push(format!("Embedding failed: {}. Is Ollama running with nomic-embed-text?", e));
                break; // Stop on first failure — Ollama is likely offline
            }
        }
    }

    Ok(EmbeddingResult {
        chunks_processed: total,
        embeddings_generated: generated,
        already_embedded: 0,
        errors,
    })
}
