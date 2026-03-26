use std::collections::HashMap;

use tauri::State;

use crate::app_state::AppState;

#[derive(serde::Serialize, Clone)]
pub struct SearchResult {
    pub chunk_id: String,
    pub document_id: String,
    pub text: String,
    pub score: f64,
    pub timestamp: String,
    pub source_platform: String,
}

#[tauri::command]
pub fn keyword_search(
    query: String,
    top_k: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResult>, String> {
    let k = top_k.unwrap_or(10);
    tracing::debug!(query_len = query.len(), top_k = k, "Keyword search");
    let fts_results = state
        .page_index
        .search(&query, k)
        .map_err(|e| e.to_string())?;

    let chunk_ids: Vec<String> = fts_results.iter().map(|r| r.chunk_id.clone()).collect();
    let chunks = state
        .document_store
        .get_chunks_by_ids(&chunk_ids)
        .map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for fts_result in &fts_results {
        if let Some(chunk) = chunks.iter().find(|c| c.id == fts_result.chunk_id) {
            let (timestamp, platform) = match state.document_store.get_by_id(&chunk.document_id) {
                Ok(Some(doc)) => (doc.timestamp.to_rfc3339(), doc.source_platform.to_string()),
                _ => (String::new(), String::new()),
            };

            results.push(SearchResult {
                chunk_id: chunk.id.clone(),
                document_id: chunk.document_id.clone(),
                text: fts_result.snippet.clone(),
                score: fts_result.rank_score,
                timestamp,
                source_platform: platform,
            });
        }
    }

    tracing::debug!(results = results.len(), "Keyword search complete");
    Ok(results)
}

#[tauri::command]
pub fn semantic_search(
    query: String,
    top_k: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResult>, String> {
    let k = top_k.unwrap_or(10);
    tracing::debug!(query_len = query.len(), top_k = k, "Semantic search");

    // Embed the query using the embedding provider
    let provider = state
        .embedding_provider
        .read()
        .map_err(|e| format!("Lock error: {}", e))?;

    let query_vector = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(provider.embed(&query))
    })
    .map_err(|e| format!("Embedding failed: {}. Is Ollama running?", e))?;

    // Search the vector store
    let vector_results = state
        .vector_store
        .search(&query_vector, k, None)
        .map_err(|e| e.to_string())?;

    // Enrich with document metadata
    let chunk_ids: Vec<String> = vector_results.iter().map(|r| r.id.clone()).collect();
    let chunks = state
        .document_store
        .get_chunks_by_ids(&chunk_ids)
        .map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for vr in &vector_results {
        if let Some(chunk) = chunks.iter().find(|c| c.id == vr.id) {
            let (timestamp, platform) = match state.document_store.get_by_id(&chunk.document_id) {
                Ok(Some(doc)) => (doc.timestamp.to_rfc3339(), doc.source_platform.to_string()),
                _ => (String::new(), String::new()),
            };

            results.push(SearchResult {
                chunk_id: chunk.id.clone(),
                document_id: chunk.document_id.clone(),
                text: chunk.text.clone(),
                score: vr.score as f64,
                timestamp,
                source_platform: platform,
            });
        }
    }

    tracing::debug!(results = results.len(), "Semantic search complete");
    Ok(results)
}

/// Hybrid search: combine keyword (BM25) and semantic (vector) results via RRF.
#[tauri::command]
pub fn hybrid_search(
    query: String,
    top_k: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResult>, String> {
    let k = top_k.unwrap_or(10);
    let fetch_k = k * 3; // fetch more candidates for fusion

    // Get keyword results
    let keyword_results = keyword_search(query.clone(), Some(fetch_k), state.clone())?;

    // Try semantic search (may fail if Ollama isn't running)
    let semantic_results = semantic_search(query, Some(fetch_k), state).unwrap_or_default();

    // Reciprocal Rank Fusion: score(d) = Σ 1/(k + rank_i(d))
    let rrf_k = 60.0_f64;
    let mut rrf_scores: HashMap<String, (f64, SearchResult)> = HashMap::new();

    for (rank, result) in keyword_results.iter().enumerate() {
        let rrf_score = 1.0 / (rrf_k + rank as f64);
        rrf_scores
            .entry(result.chunk_id.clone())
            .and_modify(|(score, _)| *score += rrf_score)
            .or_insert((rrf_score, result.clone()));
    }

    for (rank, result) in semantic_results.iter().enumerate() {
        let rrf_score = 1.0 / (rrf_k + rank as f64);
        rrf_scores
            .entry(result.chunk_id.clone())
            .and_modify(|(score, _)| *score += rrf_score)
            .or_insert((rrf_score, result.clone()));
    }

    let mut fused: Vec<SearchResult> = rrf_scores
        .into_values()
        .map(|(score, mut result)| {
            result.score = score;
            result
        })
        .collect();

    fused.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    fused.truncate(k);

    Ok(fused)
}

#[tauri::command]
pub fn get_document_text(
    document_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let doc = state
        .document_store
        .get_by_id(&document_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Document not found".to_string())?;
    Ok(doc.raw_text)
}
