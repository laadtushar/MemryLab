use std::collections::HashMap;

use tauri::{Manager, State};

use crate::adapters::sqlite::activity_store::ActivityEntry;
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
                Ok(Some(doc)) => (doc.timestamp.map(|t| t.to_rfc3339()).unwrap_or_default(), doc.source_platform.to_string()),
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
pub async fn semantic_search(
    query: String,
    top_k: Option<usize>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<SearchResult>, String> {
    let k = top_k.unwrap_or(10);
    tracing::debug!(query_len = query.len(), top_k = k, "Semantic search");

    // Spawn on blocking thread pool — retrieve state via app_handle inside
    let results = tokio::task::spawn_blocking(move || {
        let state = app_handle.state::<AppState>();
        let provider = state.embedding_provider.read().map_err(|e| format!("Lock error: {}", e))?;
        let query_vector = tauri::async_runtime::block_on(provider.embed(&query))
            .map_err(|e| format!("Embedding failed: {}. Is Ollama running?", e))?;

        let vector_results = state.vector_store
            .search(&query_vector, k, None)
            .map_err(|e| e.to_string())?;

        let chunk_ids: Vec<String> = vector_results.iter().map(|r| r.id.clone()).collect();
        let chunks = state.document_store
            .get_chunks_by_ids(&chunk_ids)
            .map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for vr in &vector_results {
            if let Some(chunk) = chunks.iter().find(|c| c.id == vr.id) {
                let (timestamp, platform) = match state.document_store.get_by_id(&chunk.document_id) {
                    Ok(Some(doc)) => (doc.timestamp.map(|t| t.to_rfc3339()).unwrap_or_default(), doc.source_platform.to_string()),
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
        Ok::<Vec<SearchResult>, String>(results)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;

    tracing::debug!(results = results.len(), "Semantic search complete");
    Ok(results)
}

/// Hybrid search: combine keyword (BM25) and semantic (vector) results via RRF.
#[tauri::command]
pub async fn hybrid_search(
    query: String,
    top_k: Option<usize>,
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResult>, String> {
    let k = top_k.unwrap_or(10);
    let fetch_k = k * 3; // fetch more candidates for fusion
    let query_for_log = query.clone();

    // Get keyword results (fast, synchronous FTS5)
    let keyword_results = keyword_search(query.clone(), Some(fetch_k), state.clone())?;

    // Try semantic search (may fail if Ollama isn't running) — runs on blocking thread
    let semantic_results = semantic_search(query, Some(fetch_k), app_handle).await.unwrap_or_default();

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

    let truncated_query: String = query_for_log.chars().take(100).collect();
    let _ = state.activity_store.log_activity(&ActivityEntry {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        action_type: "search".to_string(),
        title: truncated_query,
        description: String::new(),
        result_summary: format!("{} results", fused.len()),
        metadata: serde_json::json!({}),
        duration_ms: 0,
        status: "success".to_string(),
    });

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

/// Search suggestions (autocomplete) using FTS5 prefix matching
#[tauri::command]
pub fn search_suggestions(
    prefix: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    if prefix.len() < 2 {
        return Ok(Vec::new());
    }
    state.page_index.suggest(&prefix, 8).map_err(|e| e.to_string())
}

/// Find related documents to a given document using BM25 term similarity
#[tauri::command]
pub fn related_documents(
    document_id: String,
    top_k: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResult>, String> {
    let k = top_k.unwrap_or(5);
    // Get chunks of this document
    let chunks = state.document_store
        .get_chunks_by_document(&document_id)
        .map_err(|e| e.to_string())?;

    if chunks.is_empty() {
        return Ok(Vec::new());
    }

    // Use the first/longest chunk for similarity
    let source_chunk = chunks.iter().max_by_key(|c| c.text.len()).unwrap();
    let related = state.page_index
        .find_related(&source_chunk.id, k * 2)
        .map_err(|e| e.to_string())?;

    // Resolve to documents, dedup by document_id
    let chunk_ids: Vec<String> = related.iter().map(|r| r.chunk_id.clone()).collect();
    let result_chunks = state.document_store
        .get_chunks_by_ids(&chunk_ids)
        .map_err(|e| e.to_string())?;

    let mut seen_docs = std::collections::HashSet::new();
    seen_docs.insert(document_id.clone()); // exclude source doc
    let mut results = Vec::new();

    for fts in &related {
        if let Some(chunk) = result_chunks.iter().find(|c| c.id == fts.chunk_id) {
            if seen_docs.insert(chunk.document_id.clone()) {
                let (timestamp, platform) = match state.document_store.get_by_id(&chunk.document_id) {
                    Ok(Some(doc)) => (doc.timestamp.map(|t| t.to_rfc3339()).unwrap_or_default(), doc.source_platform.to_string()),
                    _ => (String::new(), String::new()),
                };
                results.push(SearchResult {
                    chunk_id: chunk.id.clone(),
                    document_id: chunk.document_id.clone(),
                    text: fts.snippet.clone(),
                    score: fts.rank_score,
                    timestamp,
                    source_platform: platform,
                });
                if results.len() >= k { break; }
            }
        }
    }

    Ok(results)
}

/// Search memory facts by text content
#[tauri::command]
pub fn search_memory_facts(
    query: String,
    category: Option<String>,
    top_k: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<crate::commands::insights::MemoryFactResponse>, String> {
    let k = top_k.unwrap_or(50);
    let facts = state.memory_store
        .get_all(None, None)
        .map_err(|e| e.to_string())?;

    let q = query.to_lowercase();
    let filtered: Vec<_> = facts.into_iter()
        .filter(|f| {
            f.fact_text.to_lowercase().contains(&q) &&
            (category.is_none() || format!("{:?}", f.category).to_lowercase() == category.as_deref().unwrap_or(""))
        })
        .take(k)
        .map(|f| crate::commands::insights::MemoryFactResponse {
            id: f.id,
            fact_text: f.fact_text,
            category: format!("{:?}", f.category).to_lowercase(),
            confidence: f.confidence,
            first_seen: f.first_seen.to_rfc3339(),
            last_updated: f.last_updated.to_rfc3339(),
            is_active: f.is_active,
        })
        .collect();

    Ok(filtered)
}

/// Search entities by name
#[tauri::command]
pub fn search_entities(
    query: String,
    entity_type: Option<String>,
    top_k: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<crate::commands::entities::EntityResponse>, String> {
    let k = top_k.unwrap_or(50);
    let subgraph = state.graph_store
        .get_all_entities(500, entity_type.as_deref())
        .map_err(|e| e.to_string())?;

    let q = query.to_lowercase();
    let filtered: Vec<_> = subgraph.nodes.into_iter()
        .filter(|e| e.name.to_lowercase().contains(&q))
        .take(k)
        .map(|e| crate::commands::entities::EntityResponse {
            id: e.id,
            name: e.name,
            entity_type: format!("{:?}", e.entity_type).to_lowercase(),
            mention_count: e.mention_count,
            first_seen: e.first_seen.map(|d| d.to_rfc3339()),
            last_seen: e.last_seen.map(|d| d.to_rfc3339()),
        })
        .collect();

    Ok(filtered)
}

/// Quick search across all content types (documents, memories, entities)
#[derive(serde::Serialize)]
pub struct QuickSearchResult {
    pub result_type: String,  // "document", "memory", "entity", "chat"
    pub id: String,
    pub title: String,
    pub snippet: String,
    pub score: f64,
}

#[tauri::command]
pub fn quick_search(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<QuickSearchResult>, String> {
    if query.len() < 2 {
        return Ok(Vec::new());
    }

    let mut results = Vec::new();

    // Search documents via FTS5
    if let Ok(fts_results) = state.page_index.search(&query, 5) {
        for r in fts_results {
            results.push(QuickSearchResult {
                result_type: "document".to_string(),
                id: r.chunk_id.clone(),
                title: "Document".to_string(),
                snippet: r.snippet,
                score: r.rank_score,
            });
        }
    }

    // Search memory facts
    let q = query.to_lowercase();
    if let Ok(facts) = state.memory_store.get_all(None, None) {
        for f in facts.iter().filter(|f| f.fact_text.to_lowercase().contains(&q)).take(3) {
            results.push(QuickSearchResult {
                result_type: "memory".to_string(),
                id: f.id.clone(),
                title: format!("{:?}", f.category),
                snippet: f.fact_text.chars().take(120).collect(),
                score: 1.0,
            });
        }
    }

    // Search entities
    if let Ok(sg) = state.graph_store.get_all_entities(100, None) {
        for e in sg.nodes.iter().filter(|e| e.name.to_lowercase().contains(&q)).take(3) {
            results.push(QuickSearchResult {
                result_type: "entity".to_string(),
                id: e.id.clone(),
                title: e.name.clone(),
                snippet: format!("{:?} — {} mentions", e.entity_type, e.mention_count),
                score: e.mention_count as f64,
            });
        }
    }

    // Search chat messages
    if let Ok(conversations) = state.chat_store.list_conversations(20) {
        for conv in conversations.iter().take(20) {
            if conv.title.to_lowercase().contains(&q) {
                results.push(QuickSearchResult {
                    result_type: "chat".to_string(),
                    id: conv.id.clone(),
                    title: conv.title.clone(),
                    snippet: format!("Conversation — {}", conv.updated_at),
                    score: 0.5,
                });
            }
        }
    }

    Ok(results)
}
