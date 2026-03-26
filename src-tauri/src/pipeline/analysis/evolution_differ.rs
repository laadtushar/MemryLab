use crate::domain::ports::llm_provider::{ILlmProvider, LlmParams};
use crate::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DiffResult {
    pub summary: String,
    pub sentiment_a: String,
    pub sentiment_b: String,
    pub key_shift: String,
    pub quote_a: String,
    pub quote_b: String,
}

pub async fn compare_periods(
    period_a_text: &str,
    period_b_text: &str,
    period_a_label: &str,
    period_b_label: &str,
    llm: &dyn ILlmProvider,
) -> Result<DiffResult, AppError> {
    let prompt = crate::prompts::templates::EVOLUTION_DIFF_V1
        .replace("{period_a_label}", period_a_label)
        .replace("{period_a_text}", &period_a_text[..period_a_text.len().min(3000)])
        .replace("{period_b_label}", period_b_label)
        .replace("{period_b_text}", &period_b_text[..period_b_text.len().min(3000)]);

    let params = LlmParams {
        temperature: Some(0.3),
        max_tokens: Some(1024),
        ..Default::default()
    };

    let response = llm.complete(&prompt, &params).await?;

    // Try to parse JSON, fallback to extracting from markdown code blocks
    serde_json::from_str::<DiffResult>(&response).or_else(|_| {
        let json_str = response
            .find('{')
            .and_then(|start| response.rfind('}').map(|end| &response[start..=end]))
            .unwrap_or(&response);
        serde_json::from_str(json_str)
            .map_err(|e| AppError::Analysis(format!("Failed to parse diff: {}", e)))
    })
}
