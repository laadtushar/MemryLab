use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::domain::ports::embedding_provider::IEmbeddingProvider;
use crate::domain::ports::llm_provider::{Classification, ILlmProvider, LlmParams};
use crate::error::AppError;

/// Ollama HTTP API adapter implementing both ILlmProvider and IEmbeddingProvider.
pub struct OllamaProvider {
    client: Client,
    base_url: String,
    default_model: String,
    embedding_model: String,
    embedding_dimensions: usize,
}

impl OllamaProvider {
    pub fn new(base_url: &str, default_model: &str, embedding_model: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .connect_timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| Client::new()),
            base_url: base_url.trim_end_matches('/').to_string(),
            default_model: default_model.to_string(),
            embedding_model: embedding_model.to_string(),
            embedding_dimensions: 1024, // nomic-embed-text default
        }
    }

    pub fn with_dimensions(mut self, dims: usize) -> Self {
        self.embedding_dimensions = dims;
        self
    }
}

// ── Request/Response types for Ollama API ──

#[derive(Serialize)]
struct GenerateRequest {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<GenerateOptions>,
}

#[derive(Serialize)]
struct GenerateOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<usize>,
}

#[derive(Deserialize)]
struct GenerateResponse {
    response: String,
}

#[derive(Serialize)]
struct EmbedRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Deserialize)]
struct EmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

#[derive(Deserialize)]
struct TagsResponse {
    models: Vec<ModelInfo>,
}

#[derive(Deserialize)]
struct ModelInfo {
    name: String,
}

// ── ILlmProvider implementation ──

#[async_trait]
impl ILlmProvider for OllamaProvider {
    async fn complete(&self, prompt: &str, params: &LlmParams) -> Result<String, AppError> {
        let model = params
            .model
            .clone()
            .unwrap_or_else(|| self.default_model.clone());

        let request = GenerateRequest {
            model,
            prompt: prompt.to_string(),
            system: params.system_prompt.clone(),
            stream: false,
            options: Some(GenerateOptions {
                temperature: params.temperature,
                num_predict: params.max_tokens,
            }),
        };

        let response = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::LlmProvider(format!("Ollama request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::LlmProvider(format!(
                "Ollama returned {}: {}",
                status, body
            )));
        }

        let result: GenerateResponse = response
            .json()
            .await
            .map_err(|e| AppError::LlmProvider(format!("Failed to parse Ollama response: {}", e)))?;

        Ok(result.response)
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
        classify_params.temperature = Some(0.1); // Low temperature for classification

        let response = self.complete(&prompt, &classify_params).await?;
        let response_trimmed = response.trim().to_lowercase();

        // Find the best matching category
        let category = categories
            .iter()
            .find(|c| response_trimmed.contains(&c.to_lowercase()))
            .cloned()
            .unwrap_or_else(|| response_trimmed.clone());

        Ok(Classification {
            category,
            confidence: 0.8, // Heuristic confidence for LLM-based classification
        })
    }

    async fn is_available(&self) -> bool {
        self.client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    fn provider_name(&self) -> &str {
        "ollama"
    }
}

// ── IEmbeddingProvider implementation ──

#[async_trait]
impl IEmbeddingProvider for OllamaProvider {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, AppError> {
        let result = self.embed_batch(&[text.to_string()]).await?;
        result
            .into_iter()
            .next()
            .ok_or_else(|| AppError::LlmProvider("Empty embedding response".to_string()))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, AppError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let request = EmbedRequest {
            model: self.embedding_model.clone(),
            input: texts.to_vec(),
        };

        let response = self
            .client
            .post(format!("{}/api/embed", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::LlmProvider(format!("Ollama embed request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::LlmProvider(format!(
                "Ollama embed returned {}: {}",
                status, body
            )));
        }

        let result: EmbedResponse = response
            .json()
            .await
            .map_err(|e| AppError::LlmProvider(format!("Failed to parse embed response: {}", e)))?;

        Ok(result.embeddings)
    }

    fn dimensions(&self) -> usize {
        self.embedding_dimensions
    }

    fn model_name(&self) -> &str {
        &self.embedding_model
    }
}

// ── Utility functions ──

impl OllamaProvider {
    /// List all available models on the Ollama server.
    pub async fn list_models(&self) -> Result<Vec<String>, AppError> {
        let response = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map_err(|e| AppError::LlmProvider(format!("Failed to list Ollama models: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::LlmProvider(
                "Ollama is not running. Please start Ollama to use local models.".to_string(),
            ));
        }

        let tags: TagsResponse = response
            .json()
            .await
            .map_err(|e| AppError::LlmProvider(format!("Failed to parse model list: {}", e)))?;

        Ok(tags.models.into_iter().map(|m| m.name).collect())
    }
}
