use chrono::Utc;
use uuid::Uuid;

use crate::domain::models::common::TimeGranularity;
use crate::domain::models::memory::{FactCategory, MemoryFact};
use crate::domain::ports::document_store::IDocumentStore;
use crate::domain::ports::graph_store::IGraphStore;
use crate::domain::ports::llm_provider::ILlmProvider;
use crate::domain::ports::memory_store::IMemoryStore;
use crate::domain::ports::timeline_store::ITimelineStore;
use crate::error::AppError;

use super::belief_extractor;
use super::contradiction_detector;
use super::entity_extractor;
use super::insight_generator;
use super::narrative_generator;
use super::sentiment_tracker;
use super::theme_extractor;

/// Configuration for an analysis run.
pub struct AnalysisConfig {
    pub granularity: TimeGranularity,
    /// If set, only analyze documents created/updated after this timestamp (incremental mode).
    pub since: Option<chrono::DateTime<Utc>>,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            granularity: TimeGranularity::Monthly,
            since: None,
        }
    }
}

/// Result of a full analysis run.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AnalysisResult {
    pub themes_extracted: usize,
    pub beliefs_extracted: usize,
    pub sentiments_classified: usize,
    pub entities_extracted: usize,
    pub insights_generated: usize,
    pub contradictions_found: usize,
    pub narratives_generated: usize,
}

/// Run the full analysis pipeline: themes → sentiment → beliefs → entities → insights.
pub async fn run_analysis(
    document_store: &dyn IDocumentStore,
    timeline_store: &dyn ITimelineStore,
    memory_store: &dyn IMemoryStore,
    graph_store: &dyn IGraphStore,
    llm: &dyn ILlmProvider,
    config: Option<AnalysisConfig>,
) -> Result<AnalysisResult, AppError> {
    run_analysis_with_progress(document_store, timeline_store, memory_store, graph_store, llm, config, |_, _| {}).await
}

/// Run the full analysis pipeline with a progress callback invoked at each stage.
pub async fn run_analysis_with_progress(
    document_store: &dyn IDocumentStore,
    timeline_store: &dyn ITimelineStore,
    memory_store: &dyn IMemoryStore,
    graph_store: &dyn IGraphStore,
    llm: &dyn ILlmProvider,
    config: Option<AnalysisConfig>,
    on_progress: impl Fn(&str, &str),
) -> Result<AnalysisResult, AppError> {
    let config = config.unwrap_or_default();
    // Check LLM availability
    if !llm.is_available().await {
        return Err(AppError::Analysis(
            "LLM provider is not available. Please start Ollama or configure an API key."
                .to_string(),
        ));
    }

    // Stage 1: Theme extraction
    on_progress("themes", "Extracting themes...");
    log::info!("Analysis: extracting themes...");
    let themes = theme_extractor::extract_themes(
        document_store,
        timeline_store,
        llm,
        30, // max chunks per window
        &config.granularity,
    )
    .await?;
    log::info!("Analysis: extracted {} themes", themes.len());

    // Stage 2: Sample chunks for belief extraction and sentiment
    let incremental_label = if config.since.is_some() { " (incremental)" } else { "" };
    on_progress("sampling", &format!("Sampling documents{}...", incremental_label));
    log::info!("Analysis: sampling chunks{}...", incremental_label);
    let months = timeline_store.get_document_count_by_month()?;
    let mut all_chunks: Vec<(String, String)> = Vec::new();

    // Sample chunks from documents, preserving source document timestamp
    let mut chunk_timestamps: std::collections::HashMap<String, chrono::DateTime<Utc>> = std::collections::HashMap::new();
    for (_, _) in months.iter().take(12) {
        let date_range = timeline_store.get_date_range()?;
        if let Some(range) = date_range {
            let doc_ids = timeline_store.get_documents_in_range(&range)?;
            for doc_id in doc_ids.iter().take(10) {
                let doc = document_store.get_by_id(doc_id).ok().flatten();
                let doc_timestamp = doc.as_ref().and_then(|d| d.timestamp).unwrap_or_else(Utc::now);

                // Incremental: skip documents older than the last analysis run
                if let Some(since) = config.since {
                    let doc_updated = doc.as_ref().map(|d| d.updated_at).unwrap_or(doc_timestamp);
                    if doc_updated < since {
                        continue;
                    }
                }

                let chunks = document_store.get_chunks_by_document(doc_id)?;
                for chunk in chunks.into_iter().take(3) {
                    chunk_timestamps.insert(chunk.id.clone(), doc_timestamp);
                    all_chunks.push((chunk.id, chunk.text));
                }
            }
        }
        break; // Only sample once for now
    }
    if all_chunks.is_empty() && config.since.is_some() {
        log::info!("Incremental analysis: no new documents since last run");
        on_progress("complete", "No new documents to analyze");
        return Ok(AnalysisResult {
            themes_extracted: 0,
            beliefs_extracted: 0,
            sentiments_classified: 0,
            entities_extracted: 0,
            insights_generated: 0,
            contradictions_found: 0,
            narratives_generated: 0,
        });
    }

    // Stage 3: Sentiment classification on sampled chunks
    on_progress("sentiment", &format!("Classifying sentiment on {} chunks...", all_chunks.len()));
    log::info!("Analysis: classifying sentiment on {} chunks...", all_chunks.len());
    let sentiment_results = if !all_chunks.is_empty() {
        // Limit to 20 chunks to avoid excessive LLM calls
        let sentiment_sample: Vec<(String, String)> = all_chunks.iter().take(20).cloned().collect();
        sentiment_tracker::classify_sentiment_batch(&sentiment_sample, llm).await
    } else {
        Vec::new()
    };
    let sentiments_classified = sentiment_results.len();
    log::info!("Analysis: classified {} sentiments", sentiments_classified);

    // Stage 4: Belief extraction
    on_progress("beliefs", "Extracting beliefs...");
    log::info!("Analysis: extracting beliefs...");
    let beliefs = if !all_chunks.is_empty() {
        belief_extractor::extract_beliefs(&all_chunks, llm).await?
    } else {
        Vec::new()
    };
    log::info!("Analysis: extracted {} beliefs", beliefs.len());

    // Store beliefs in memory store, using source document timestamps
    for fact in &beliefs {
        let mut fact = fact.clone();
        // Use the earliest source chunk's document timestamp instead of Utc::now()
        if let Some(ts) = fact.source_chunks.iter()
            .filter_map(|cid| chunk_timestamps.get(cid))
            .min()
        {
            fact.first_seen = *ts;
        }
        if let Err(e) = memory_store.store(&fact) {
            log::warn!("Failed to store belief: {}", e);
        }
    }

    // Stage 5: Entity extraction → graph store
    on_progress("entities", "Extracting entities...");
    log::info!("Analysis: extracting entities...");
    let entities_with_sources = entity_extractor::extract_entities(&all_chunks, llm, 15).await?;
    let entities_extracted = entities_with_sources.len();
    for (entity, _source_chunks) in &entities_with_sources {
        if let Err(e) = graph_store.add_node(entity) {
            log::warn!("Failed to store entity '{}': {}", entity.name, e);
        }
    }
    log::info!("Analysis: extracted {} entities", entities_extracted);

    // Stage 6: Insight generation
    on_progress("insights", "Generating insights...");
    log::info!("Analysis: generating insights...");
    let insights = insight_generator::generate_insights(&themes, &beliefs, llm, 5).await?;
    log::info!("Analysis: generated {} insights", insights.len());

    // Store insights as MemoryFacts with Insight category so they persist and appear in the UI
    let earliest_theme_date = themes.iter().map(|t| t.time_window_start).min().unwrap_or_else(Utc::now);
    for insight in &insights {
        let fact = MemoryFact {
            id: Uuid::new_v4().to_string(),
            fact_text: format!("{}: {}", insight.title, insight.body),
            source_chunks: insight.supporting_evidence.clone(),
            confidence: 0.7,
            category: FactCategory::Insight,
            first_seen: earliest_theme_date,
            last_updated: Utc::now(),
            contradicted_by: vec![],
            is_active: true,
        };
        if let Err(e) = memory_store.store(&fact) {
            log::warn!("Failed to store insight: {}", e);
        }
    }

    // Stage 7: Contradiction detection
    on_progress("contradictions", "Detecting contradictions...");
    log::info!("Analysis: detecting contradictions...");
    let contradictions_found = match contradiction_detector::detect_contradictions(
        memory_store,
        llm,
    )
    .await
    {
        Ok(count) => {
            log::info!("Analysis: found {} contradictions", count);
            count
        }
        Err(e) => {
            log::warn!("Contradiction detection failed: {}", e);
            0
        }
    };

    // Stage 8: Narrative generation (final stage)
    on_progress("narratives", "Generating narratives...");
    log::info!("Analysis: generating narratives...");
    let narratives_generated = match narrative_generator::generate_narratives(
        document_store,
        memory_store,
        llm,
    )
    .await
    {
        Ok(count) => {
            log::info!("Analysis: generated {} narratives", count);
            count
        }
        Err(e) => {
            log::warn!("Narrative generation failed: {}", e);
            0
        }
    };

    Ok(AnalysisResult {
        themes_extracted: themes.len(),
        beliefs_extracted: beliefs.len(),
        sentiments_classified,
        entities_extracted,
        insights_generated: insights.len(),
        contradictions_found,
        narratives_generated,
    })
}
