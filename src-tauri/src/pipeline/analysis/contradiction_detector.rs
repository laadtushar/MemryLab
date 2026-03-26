use crate::domain::models::memory::FactCategory;
use crate::domain::ports::llm_provider::{ILlmProvider, LlmParams};
use crate::domain::ports::memory_store::IMemoryStore;
use crate::error::AppError;

#[derive(serde::Deserialize)]
struct ContradictionResult {
    is_contradiction: bool,
    #[allow(dead_code)]
    explanation: String,
    #[allow(dead_code)]
    severity: String,
}

/// Detect contradictions among active Belief/Preference facts.
/// Checks up to 20 pairs per run and marks contradictions in the memory store.
/// Returns the number of contradictions found.
pub async fn detect_contradictions(
    memory_store: &dyn IMemoryStore,
    llm: &dyn ILlmProvider,
) -> Result<usize, AppError> {
    // Get all active beliefs and preferences
    let beliefs = memory_store.get_all(Some(&FactCategory::Belief), None)?;
    let preferences = memory_store.get_all(Some(&FactCategory::Preference), None)?;

    let mut candidates: Vec<(String, String)> = Vec::new();
    for fact in beliefs.iter().chain(preferences.iter()) {
        if fact.is_active && fact.fact_text.split_whitespace().count() > 5 {
            candidates.push((fact.id.clone(), fact.fact_text.clone()));
        }
    }

    if candidates.len() < 2 {
        return Ok(0);
    }

    let params = LlmParams {
        temperature: Some(0.2),
        max_tokens: Some(512),
        ..Default::default()
    };

    let mut contradictions_found = 0usize;
    let mut pairs_checked = 0usize;

    'outer: for i in 0..candidates.len() {
        for j in (i + 1)..candidates.len() {
            if pairs_checked >= 20 {
                break 'outer;
            }

            let prompt = crate::prompts::templates::CONTRADICTION_CHECK_V1
                .replace("{fact_a}", &candidates[i].1)
                .replace("{fact_b}", &candidates[j].1);

            match llm.complete(&prompt, &params).await {
                Ok(response) => {
                    let parsed = parse_contradiction_response(&response);
                    if let Some(result) = parsed {
                        if result.is_contradiction {
                            // Mark both facts as contradicted by each other
                            let _ = memory_store.contradict(&candidates[i].0, &candidates[j].0);
                            let _ = memory_store.contradict(&candidates[j].0, &candidates[i].0);
                            contradictions_found += 1;
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Contradiction check failed for pair: {}", e);
                }
            }

            pairs_checked += 1;
        }
    }

    log::info!(
        "Contradiction detection: checked {} pairs, found {} contradictions",
        pairs_checked,
        contradictions_found
    );

    Ok(contradictions_found)
}

fn parse_contradiction_response(response: &str) -> Option<ContradictionResult> {
    // Strip markdown code fences
    let text = response.trim();
    let text = if text.starts_with("```") {
        let inner = text.trim_start_matches("```json").trim_start_matches("```");
        inner.trim_end_matches("```").trim()
    } else {
        text
    };

    serde_json::from_str::<ContradictionResult>(text)
        .or_else(|_| {
            let json_str = text
                .find('{')
                .and_then(|start| text.rfind('}').map(|end| &text[start..=end]))
                .unwrap_or(text);
            serde_json::from_str(json_str)
        })
        .ok()
}
