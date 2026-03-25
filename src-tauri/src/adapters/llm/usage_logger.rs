use async_trait::async_trait;
use std::sync::Arc;
use std::time::Instant;

use crate::adapters::sqlite::connection::SqliteConnection;
use crate::domain::ports::llm_provider::{Classification, ILlmProvider, LlmParams};
use crate::error::AppError;

/// Decorator that wraps an `ILlmProvider` and logs every call to the
/// `llm_usage_log` table for transparent cloud-usage tracking.
pub struct UsageLoggingProvider {
    inner: Box<dyn ILlmProvider>,
    db: Arc<SqliteConnection>,
}

impl UsageLoggingProvider {
    pub fn new(inner: Box<dyn ILlmProvider>, db: Arc<SqliteConnection>) -> Self {
        Self { inner, db }
    }

    fn log_usage(
        &self,
        provider: &str,
        model: &str,
        prompt_tokens: i64,
        completion_tokens: i64,
        purpose: &str,
        duration_ms: i64,
    ) {
        let id = uuid::Uuid::new_v4().to_string();
        let _ = self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO llm_usage_log (id, provider, model, prompt_tokens, completion_tokens, purpose, duration_ms) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![id, provider, model, prompt_tokens, completion_tokens, purpose, duration_ms],
            )?;
            Ok(())
        });
    }

    /// Rough token estimation: ~4 chars per token.
    fn estimate_tokens(text: &str) -> i64 {
        (text.len() as i64) / 4
    }
}

#[async_trait]
impl ILlmProvider for UsageLoggingProvider {
    async fn complete(&self, prompt: &str, params: &LlmParams) -> Result<String, AppError> {
        let start = Instant::now();
        let result = self.inner.complete(prompt, params).await?;
        let duration = start.elapsed().as_millis() as i64;
        let model = params.model.clone().unwrap_or_else(|| "default".into());
        self.log_usage(
            self.inner.provider_name(),
            &model,
            Self::estimate_tokens(prompt),
            Self::estimate_tokens(&result),
            "complete",
            duration,
        );
        Ok(result)
    }

    async fn classify(
        &self,
        text: &str,
        categories: &[String],
        params: &LlmParams,
    ) -> Result<Classification, AppError> {
        let start = Instant::now();
        let result = self.inner.classify(text, categories, params).await?;
        let duration = start.elapsed().as_millis() as i64;
        let model = params.model.clone().unwrap_or_else(|| "default".into());
        self.log_usage(
            self.inner.provider_name(),
            &model,
            Self::estimate_tokens(text),
            Self::estimate_tokens(&result.category),
            "classify",
            duration,
        );
        Ok(result)
    }

    async fn is_available(&self) -> bool {
        self.inner.is_available().await
    }

    fn provider_name(&self) -> &str {
        self.inner.provider_name()
    }
}
