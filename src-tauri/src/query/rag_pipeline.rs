use crate::domain::ports::document_store::IDocumentStore;
use crate::domain::ports::embedding_provider::IEmbeddingProvider;
use crate::domain::ports::llm_provider::{ILlmProvider, LlmParams};
use crate::domain::ports::memory_store::IMemoryStore;
use crate::domain::ports::page_index::IPageIndex;
use crate::domain::ports::vector_store::IVectorStore;
use crate::error::AppError;
use crate::prompts::templates::{render_template, RAG_RESPONSE_V1};
use std::collections::HashMap;

/// A RAG response with the generated answer and source citations.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RagResponse {
    pub answer: String,
    pub sources: Vec<RagSource>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RagSource {
    pub chunk_id: String,
    pub document_id: String,
    pub text_snippet: String,
    pub timestamp: String,
    pub score: f64,
}

/// Full RAG pipeline: embed query → retrieve → fuse → assemble context → generate.
pub async fn query_rag(
    query: &str,
    document_store: &dyn IDocumentStore,
    vector_store: &dyn IVectorStore,
    page_index: &dyn IPageIndex,
    memory_store: &dyn IMemoryStore,
    embedding_provider: &dyn IEmbeddingProvider,
    llm: &dyn ILlmProvider,
    top_k: usize,
) -> Result<RagResponse, AppError> {
    // Step 1: Embed the query (graceful degradation if embedding fails)
    let vector_results = match embedding_provider.embed(query).await {
        Ok(query_vector) => {
            vector_store.search(&query_vector, top_k * 2, None).unwrap_or_default()
        }
        Err(e) => {
            tracing::warn!(error = %e, "Embedding failed, skipping vector search");
            Vec::new()
        }
    };

    // Step 2: Full-text search (always works, no embedding needed)
    let fts_results = page_index.search(query, top_k * 2).unwrap_or_default();

    // Step 3: Reciprocal Rank Fusion (k=60)
    let rrf_k = 60.0_f64;
    let mut rrf_scores: HashMap<String, f64> = HashMap::new();

    for (rank, result) in vector_results.iter().enumerate() {
        let score = 1.0 / (rrf_k + rank as f64);
        *rrf_scores.entry(result.id.clone()).or_insert(0.0) += score;
    }
    for (rank, result) in fts_results.iter().enumerate() {
        let score = 1.0 / (rrf_k + rank as f64);
        *rrf_scores.entry(result.chunk_id.clone()).or_insert(0.0) += score;
    }

    // Sort by fused score
    let mut ranked: Vec<(String, f64)> = rrf_scores.into_iter().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    ranked.truncate(top_k);

    // Step 4: Fetch full chunks + assemble context
    let chunk_ids: Vec<String> = ranked.iter().map(|(id, _)| id.clone()).collect();
    let chunks = document_store.get_chunks_by_ids(&chunk_ids)?;

    let mut sources = Vec::new();
    let mut context_parts = Vec::new();

    for (chunk_id, score) in &ranked {
        if let Some(chunk) = chunks.iter().find(|c| &c.id == chunk_id) {
            let timestamp = document_store
                .get_by_id(&chunk.document_id)
                .ok()
                .flatten()
                .map(|d| d.timestamp.to_rfc3339())
                .unwrap_or_default();

            context_parts.push(format!(
                "[{}] {}",
                timestamp.get(..10).unwrap_or("unknown"),
                chunk.text
            ));

            sources.push(RagSource {
                chunk_id: chunk_id.clone(),
                document_id: chunk.document_id.clone(),
                text_snippet: chunk.text.chars().take(200).collect(),
                timestamp,
                score: *score,
            });
        }
    }

    // Step 5: Memory augmentation — search for each keyword to get broader results
    let mut memory_facts = memory_store.recall(query, 5)?;
    // Also search individual words for better coverage
    if memory_facts.len() < 3 {
        for word in query.split_whitespace() {
            if word.len() > 3 {
                if let Ok(more) = memory_store.recall(word, 3) {
                    for fact in more {
                        if !memory_facts.iter().any(|f| f.id == fact.id) {
                            memory_facts.push(fact);
                        }
                    }
                }
            }
            if memory_facts.len() >= 10 { break; }
        }
    }
    let memories_text = if memory_facts.is_empty() {
        "No relevant memories found.".to_string()
    } else {
        memory_facts
            .iter()
            .map(|f| format!("- {} (confidence: {:.0}%)", f.fact_text, f.confidence * 100.0))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let context = context_parts.join("\n\n");

    // Step 6: LLM generation
    let prompt = render_template(
        RAG_RESPONSE_V1,
        &[
            ("context", &context),
            ("memories", &memories_text),
            ("query", query),
        ],
    );

    let params = LlmParams {
        temperature: Some(0.5),
        max_tokens: Some(2048),
        ..Default::default()
    };

    let answer = llm.complete(&prompt, &params).await?;

    Ok(RagResponse { answer, sources })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rrf_fusion_logic() {
        // Simulate RRF with known ranks
        let rrf_k = 60.0_f64;
        let mut scores: HashMap<String, f64> = HashMap::new();

        // Item "a" is rank 0 in both systems
        *scores.entry("a".into()).or_insert(0.0) += 1.0 / (rrf_k + 0.0);
        *scores.entry("a".into()).or_insert(0.0) += 1.0 / (rrf_k + 0.0);

        // Item "b" is rank 1 in system 1, rank 0 in system 2
        *scores.entry("b".into()).or_insert(0.0) += 1.0 / (rrf_k + 1.0);
        *scores.entry("b".into()).or_insert(0.0) += 1.0 / (rrf_k + 0.0);

        // "a" should score higher (top rank in both)
        assert!(scores["a"] > scores["b"]);

        // "a" score should be 2/(60+0) = 2/60
        let expected_a = 2.0 / 60.0;
        assert!((scores["a"] - expected_a).abs() < 0.001);
    }
}
