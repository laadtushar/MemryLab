use chrono::Utc;
use uuid::Uuid;

use crate::domain::models::memory::{FactCategory, MemoryFact};
use crate::domain::ports::document_store::IDocumentStore;
use crate::domain::ports::llm_provider::{ILlmProvider, LlmParams};
use crate::domain::ports::memory_store::IMemoryStore;
use crate::error::AppError;

/// Generate narratives for major themes found in the user's data.
/// Stores each narrative as a MemoryFact with category Insight and "Narrative:" prefix.
/// Returns count of narratives generated.
pub async fn generate_narratives(
    document_store: &dyn IDocumentStore,
    memory_store: &dyn IMemoryStore,
    llm: &dyn ILlmProvider,
) -> Result<usize, AppError> {
    // Get all insights to find themes
    let all_facts = memory_store.get_all(Some(&FactCategory::Insight), None)?;

    // Extract unique theme-like subjects from insight titles
    // We look for insights that aren't already narratives
    let mut themes: Vec<String> = Vec::new();
    for fact in &all_facts {
        if fact.fact_text.starts_with("Narrative:") {
            continue;
        }
        // Use the first part of insight text (the title portion) as a theme
        let title = fact
            .fact_text
            .split(':')
            .next()
            .unwrap_or(&fact.fact_text)
            .trim()
            .to_string();
        if !title.is_empty() && !themes.contains(&title) {
            themes.push(title);
        }
    }

    // Also consider beliefs as theme sources
    let beliefs = memory_store.get_all(Some(&FactCategory::Belief), None)?;
    let preferences = memory_store.get_all(Some(&FactCategory::Preference), None)?;

    // Build evidence text from beliefs and preferences
    let evidence_items: Vec<String> = beliefs
        .iter()
        .chain(preferences.iter())
        .filter(|f| f.is_active)
        .take(30)
        .map(|f| format!("- \"{}\" ({})", f.fact_text, f.first_seen.format("%Y-%m")))
        .collect();

    let evidence_text = evidence_items.join("\n");

    if themes.is_empty() || evidence_text.is_empty() {
        return Ok(0);
    }

    let params = LlmParams {
        temperature: Some(0.8),
        max_tokens: Some(2048),
        ..Default::default()
    };

    let mut narratives_generated = 0usize;

    // Generate up to 5 narratives
    for subject in themes.iter().take(5) {
        // Gather representative quotes from document chunks linked to this theme
        let mut theme_evidence = evidence_text.clone();

        // Try to find chunks related to the subject
        let related_facts: Vec<&MemoryFact> = all_facts
            .iter()
            .filter(|f| f.fact_text.contains(subject.as_str()))
            .collect();

        // Gather source chunk texts
        let mut quote_texts: Vec<String> = Vec::new();
        for fact in related_facts.iter().take(3) {
            for chunk_id in fact.source_chunks.iter().take(2) {
                if let Ok(Some(chunk)) = document_store.get_chunk_by_id(chunk_id) {
                    let snippet = chunk.text.chars().take(200).collect::<String>();
                    quote_texts.push(format!("- \"{}...\"", snippet));
                }
            }
        }

        if !quote_texts.is_empty() {
            theme_evidence = format!("{}\n\nDirect quotes:\n{}", theme_evidence, quote_texts.join("\n"));
        }

        let prompt = crate::prompts::templates::NARRATIVE_GENERATION_V1
            .replace("{subject}", subject)
            .replace("{time_range}", "across all available data")
            .replace("{evidence}", &theme_evidence[..theme_evidence.len().min(3000)]);

        match llm.complete(&prompt, &params).await {
            Ok(narrative_text) => {
                let fact = MemoryFact {
                    id: Uuid::new_v4().to_string(),
                    fact_text: format!("Narrative: {}\n\n{}", subject, narrative_text.trim()),
                    source_chunks: vec![],
                    confidence: 0.6,
                    category: FactCategory::Insight,
                    first_seen: Utc::now(),
                    last_updated: Utc::now(),
                    contradicted_by: vec![],
                    is_active: true,
                };
                if let Err(e) = memory_store.store(&fact) {
                    log::warn!("Failed to store narrative: {}", e);
                } else {
                    narratives_generated += 1;
                }
            }
            Err(e) => {
                log::warn!("Failed to generate narrative for '{}': {}", subject, e);
            }
        }
    }

    log::info!("Narrative generation: created {} narratives", narratives_generated);
    Ok(narratives_generated)
}
