use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::domain::ports::embedding_provider::IEmbeddingProvider;
use crate::domain::ports::llm_provider::{Classification, ILlmProvider, LlmParams};
use crate::error::AppError;

/// Universal OpenAI-compatible provider.
/// Works with: OpenRouter, Groq, Gemini, Cerebras, Mistral, SambaNova,
/// Cohere, NVIDIA NIM, Cloudflare Workers AI, and any OpenAI-compat endpoint.
pub struct OpenAiCompatProvider {
    client: Client,
    base_url: String,
    api_key: String,
    default_model: String,
    embedding_model: Option<String>,
    embedding_dimensions: usize,
    provider_id: String,
}

impl OpenAiCompatProvider {
    pub fn new(
        base_url: &str,
        api_key: &str,
        default_model: &str,
        provider_id: &str,
    ) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            default_model: default_model.to_string(),
            embedding_model: None,
            embedding_dimensions: 768,
            provider_id: provider_id.to_string(),
        }
    }

    pub fn with_embedding_model(mut self, model: &str, dimensions: usize) -> Self {
        self.embedding_model = Some(model.to_string());
        self.embedding_dimensions = dimensions;
        self
    }
}

// ── Request/Response types (OpenAI chat completions format) ──

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatChoiceMessage,
}

#[derive(Deserialize)]
struct ChatChoiceMessage {
    content: Option<String>,
}

#[derive(Serialize)]
struct EmbeddingRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

// ── ILlmProvider implementation ──

#[async_trait]
impl ILlmProvider for OpenAiCompatProvider {
    async fn complete(&self, prompt: &str, params: &LlmParams) -> Result<String, AppError> {
        let model = params
            .model
            .clone()
            .unwrap_or_else(|| self.default_model.clone());

        let mut messages = Vec::new();
        if let Some(ref system) = params.system_prompt {
            messages.push(ChatMessage {
                role: "system".to_string(),
                content: system.clone(),
            });
        }
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        });

        let request = ChatRequest {
            model,
            messages,
            temperature: params.temperature,
            max_tokens: params.max_tokens,
        };

        let url = format!("{}/chat/completions", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                AppError::LlmProvider(format!("{} request failed: {}", self.provider_id, e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::LlmProvider(format!(
                "{} returned {}: {}",
                self.provider_id, status, body
            )));
        }

        let result: ChatResponse = response.json().await.map_err(|e| {
            AppError::LlmProvider(format!("Failed to parse {} response: {}", self.provider_id, e))
        })?;

        let text = result
            .choices
            .into_iter()
            .filter_map(|c| c.message.content)
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
            confidence: 0.85,
        })
    }

    async fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }

    fn provider_name(&self) -> &str {
        &self.provider_id
    }
}

// ── IEmbeddingProvider implementation ──

#[async_trait]
impl IEmbeddingProvider for OpenAiCompatProvider {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, AppError> {
        let result = self.embed_batch(&[text.to_string()]).await?;
        result
            .into_iter()
            .next()
            .ok_or_else(|| AppError::LlmProvider("Empty embedding response".to_string()))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, AppError> {
        let model = self
            .embedding_model
            .as_ref()
            .ok_or_else(|| {
                AppError::LlmProvider(format!(
                    "{} does not have an embedding model configured",
                    self.provider_id
                ))
            })?
            .clone();

        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let request = EmbeddingRequest {
            model,
            input: texts.to_vec(),
        };

        let url = format!("{}/embeddings", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                AppError::LlmProvider(format!("{} embed request failed: {}", self.provider_id, e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::LlmProvider(format!(
                "{} embed returned {}: {}",
                self.provider_id, status, body
            )));
        }

        let result: EmbeddingResponse = response.json().await.map_err(|e| {
            AppError::LlmProvider(format!(
                "Failed to parse {} embed response: {}",
                self.provider_id, e
            ))
        })?;

        Ok(result.data.into_iter().map(|d| d.embedding).collect())
    }

    fn dimensions(&self) -> usize {
        self.embedding_dimensions
    }

    fn model_name(&self) -> &str {
        self.embedding_model.as_deref().unwrap_or("none")
    }
}
