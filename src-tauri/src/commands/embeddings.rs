use std::collections::HashMap;

use tauri::{AppHandle, Emitter, Manager};

use crate::adapters::sqlite::activity_store::ActivityEntry;
use crate::app_state::AppState;

#[derive(serde::Serialize)]
pub struct EmbeddingResult {
    pub chunks_processed: usize,
    pub embeddings_generated: usize,
    pub already_embedded: usize,
    pub errors: Vec<String>,
}

#[derive(Clone, serde::Serialize)]
struct EmbeddingProgress {
    stage: String,
    current: usize,
    total: usize,
    message: String,
}

fn emit_progress(app_handle: &AppHandle, stage: &str, current: usize, total: usize, message: &str) {
    let _ = app_handle.emit(
        "embedding-progress",
        EmbeddingProgress {
            stage: stage.to_string(),
            current,
            total,
            message: message.to_string(),
        },
    );
}

/// Generate embeddings for all chunks that don't have them yet.
/// Emits "embedding-progress" events so the frontend can show a progress bar.
#[tauri::command]
pub async fn generate_embeddings(
    app_handle: AppHandle,
) -> Result<EmbeddingResult, String> {
    tokio::task::spawn_blocking(move || {
        generate_embeddings_blocking(&app_handle)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

fn generate_embeddings_blocking(
    app_handle: &AppHandle,
) -> Result<EmbeddingResult, String> {
    let state = app_handle.state::<AppState>();
    let provider = state
        .embedding_provider
        .read()
        .map_err(|e| format!("Lock error: {}", e))?;

    emit_progress(app_handle, "scanning", 0, 0, "Scanning documents for chunks...");

    let months = state
        .timeline_store
        .get_document_count_by_month()
        .map_err(|e| e.to_string())?;

    if months.is_empty() {
        emit_progress(&app_handle, "complete", 0, 0, "No documents found");
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
    emit_progress(&app_handle, "embedding", 0, total, &format!("Found {} chunks to embed", total));
    tracing::info!(total_chunks = total, "Starting embedding generation");

    let mut generated = 0;
    let mut errors = Vec::new();
    let batch_size = 10;

    for (batch_idx, batch) in all_chunks.chunks(batch_size).enumerate() {
        let texts: Vec<String> = batch.iter().map(|(_, t)| t.clone()).collect();
        let ids: Vec<&str> = batch.iter().map(|(id, _)| id.as_str()).collect();

        let progress = (batch_idx * batch_size).min(total);
        emit_progress(
            &app_handle,
            "embedding",
            progress,
            total,
            &format!("Embedding batch {}/{}...", batch_idx + 1, (total + batch_size - 1) / batch_size),
        );

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
                errors.push(format!("Embedding failed: {}", e));
                tracing::error!(error = %e, "Embedding batch failed");
                break;
            }
        }
    }

    emit_progress(&app_handle, "complete", total, total, &format!("Done! {} embeddings generated", generated));
    tracing::info!(generated = generated, errors = errors.len(), "Embedding generation complete");

    let _ = state.activity_store.log_activity(&ActivityEntry {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        action_type: "embeddings".to_string(),
        title: "Generated embeddings".to_string(),
        description: String::new(),
        result_summary: format!("{} generated", generated),
        metadata: serde_json::json!({}),
        duration_ms: 0,
        status: if errors.is_empty() { "success".to_string() } else { "warning".to_string() },
    });

    Ok(EmbeddingResult {
        chunks_processed: total,
        embeddings_generated: generated,
        already_embedded: 0,
        errors,
    })
}
