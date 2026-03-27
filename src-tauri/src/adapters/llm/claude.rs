use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::domain::ports::llm_provider::{Classification, ILlmProvider, LlmParams};
use crate::error::AppError;

const CLAUDE_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Claude API adapter implementing ILlmProvider.
/// Does NOT implement IEmbeddingProvider — embeddings always run locally.
pub struct ClaudeProvider {
    client: Client,
    api_key: String,
    default_model: String,
}

impl ClaudeProvider {
    pub fn new(api_key: &str, default_model: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .connect_timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| Client::new()),
            api_key: api_key.to_string(),
            default_model: default_model.to_string(),
        }
    }
}

// ── Request/Response types for Claude API ──

#[derive(Serialize)]
struct MessagesRequest {
    model: String,
    max_tokens: usize,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct MessagesResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: Option<String>,
}

#[derive(Deserialize)]
struct ApiError {
    error: ApiErrorDetail,
}

#[derive(Deserialize)]
struct ApiErrorDetail {
    message: String,
}

// ── ILlmProvider implementation ──

#[async_trait]
impl ILlmProvider for ClaudeProvider {
    async fn complete(&self, prompt: &str, params: &LlmParams) -> Result<String, AppError> {
        let model = params
            .model
            .clone()
            .unwrap_or_else(|| self.default_model.clone());

        let request = MessagesRequest {
            model,
            max_tokens: params.max_tokens.unwrap_or(4096),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            system: params.system_prompt.clone(),
            temperature: params.temperature,
        };

        let response = self
            .client
            .post(CLAUDE_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::LlmProvider(format!("Claude API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();

            // Try to parse structured error
            if let Ok(api_err) = serde_json::from_str::<ApiError>(&body) {
                return Err(AppError::LlmProvider(format!(
                    "Claude API error ({}): {}",
                    status, api_err.error.message
                )));
            }

            return Err(AppError::LlmProvider(format!(
                "Claude API returned {}: {}",
                status, body
            )));
        }

        let result: MessagesResponse = response
            .json()
            .await
            .map_err(|e| AppError::LlmProvider(format!("Failed to parse Claude response: {}", e)))?;

        let text = result
            .content
            .into_iter()
            .filter_map(|block| block.text)
            .collect::<Vec<_>>()
            .join("");

        Ok(text)
    }

    async fn classify(
        &self,
        text: &str,
        categories: &[String],
        params: &LlmParams,
    ) -> Result<Classification, AppError> {
        let categories_str = categories.join(", ");
        let prompt = format!(
            "Classify the following text into exactly one of these categories: {}\n\n\
             Text: \"{}\"\n\n\
             Respond with ONLY the category name, nothing else.",
            categories_str, text
        );

        let mut classify_params = params.clone();
        classify_params.temperature = Some(0.1);

        let response = self.complete(&prompt, &classify_params).await?;
        let response_trimmed = response.trim().to_lowercase();

        let category = categories
            .iter()
            .find(|c| response_trimmed.contains(&c.to_lowercase()))
            .cloned()
            .unwrap_or_else(|| response_trimmed.clone());

        Ok(Classification {
            category,
            confidence: 0.9, // Claude typically higher confidence
        })
    }

    async fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }

    fn provider_name(&self) -> &str {
        "claude"
    }
}
