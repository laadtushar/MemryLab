use chrono::Utc;
use uuid::Uuid;

use crate::domain::models::memory::{FactCategory, MemoryFact};
use crate::domain::ports::llm_provider::{ILlmProvider, LlmParams};
use crate::error::AppError;
use crate::prompts::templates::{render_template, BELIEF_EXTRACTION_V1};

/// Extract beliefs, preferences, and self-descriptions from text chunks.
pub async fn extract_beliefs(
    chunks: &[(String, String)], // (chunk_id, text)
    llm: &dyn ILlmProvider,
) -> Result<Vec<MemoryFact>, AppError> {
    if chunks.is_empty() {
        return Ok(Vec::new());
    }

    let chunks_text = chunks
        .iter()
        .enumerate()
        .map(|(i, (_, text))| format!("[{}] {}", i + 1, text))
        .collect::<Vec<_>>()
        .join("\n\n");

    let prompt = render_template(BELIEF_EXTRACTION_V1, &[("chunks", &chunks_text)]);
    let params = LlmParams {
        temperature: Some(0.3),
        max_tokens: Some(2048),
        ..Default::default()
    };

    let response = llm.complete(&prompt, &params).await?;
    let source_chunk_ids: Vec<String> = chunks.iter().map(|(id, _)| id.clone()).collect();

    Ok(parse_belief_response(&response, &source_chunk_ids))
}

fn parse_belief_response(response: &str, source_chunks: &[String]) -> Vec<MemoryFact> {
    let json_str = extract_json_array(response);
    let parsed: Result<Vec<BeliefEntry>, _> = serde_json::from_str(&json_str);

    match parsed {
        Ok(entries) => entries
            .into_iter()
            .map(|entry| {
                let category = match entry.category.to_lowercase().as_str() {
                    "belief" => FactCategory::Belief,
                    "preference" => FactCategory::Preference,
                    "self_description" => FactCategory::SelfDescription,
                    _ => FactCategory::Fact,
                };

                MemoryFact {
                    id: Uuid::new_v4().to_string(),
                    fact_text: entry.fact_text,
                    source_chunks: source_chunks.to_vec(),
                    confidence: entry.confidence.clamp(0.0, 1.0),
                    category,
                    first_seen: Utc::now(),
                    last_updated: Utc::now(),
                    contradicted_by: vec![],
                    is_active: true,
                }
            })
            .collect(),
        Err(e) => {
            log::warn!("Failed to parse belief response: {}", e);
            Vec::new()
        }
    }
}

#[derive(serde::Deserialize)]
struct BeliefEntry {
    fact_text: String,
    category: String,
    confidence: f64,
}

fn extract_json_array(text: &str) -> String {
    let text = text.trim();
    let text = if text.starts_with("```") {
        let inner = text.trim_start_matches("```json").trim_start_matches("```");
        inner.trim_end_matches("```").trim()
    } else {
        text
    };
    if let Some(start) = text.find('[') {
        if let Some(end) = text.rfind(']') {
            return text[start..=end].to_string();
        }
    }
    text.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_belief_response() {
        let response = r#"[
            {"fact_text": "I value honesty above all", "category": "belief", "confidence": 0.9},
            {"fact_text": "I prefer working from home", "category": "preference", "confidence": 0.8}
        ]"#;
        let facts = parse_belief_response(response, &["c1".to_string()]);
        assert_eq!(facts.len(), 2);
        assert_eq!(facts[0].category, FactCategory::Belief);
        assert_eq!(facts[1].category, FactCategory::Preference);
    }
}
