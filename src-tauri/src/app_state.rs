use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::adapters::keychain::{self, KeychainAdapter};
use crate::adapters::llm::claude::ClaudeProvider;
use crate::adapters::llm::ollama::OllamaProvider;
use crate::adapters::llm::usage_logger::UsageLoggingProvider;
use crate::adapters::sqlite::config_store::SqliteConfigStore;
use crate::adapters::sqlite::connection::SqliteConnection;
use crate::adapters::sqlite::document_store::SqliteDocumentStore;
use crate::adapters::sqlite::graph_store::SqliteGraphStore;
use crate::adapters::sqlite::memory_store::SqliteMemoryStore;
use crate::adapters::sqlite::page_index::SqliteFts5Index;
use crate::adapters::sqlite::prompt_store::SqlitePromptStore;
use crate::adapters::sqlite::timeline_store::SqliteTimelineStore;
use crate::adapters::sqlite::vector_store::SqliteVectorStore;
use crate::domain::ports::document_store::IDocumentStore;
use crate::domain::ports::embedding_provider::IEmbeddingProvider;
use crate::domain::ports::graph_store::IGraphStore;
use crate::domain::ports::llm_provider::ILlmProvider;
use crate::domain::ports::memory_store::IMemoryStore;
use crate::domain::ports::page_index::IPageIndex;
use crate::domain::ports::timeline_store::ITimelineStore;
use crate::domain::ports::vector_store::IVectorStore;
use crate::error::AppError;

/// Central application state holding all adapter instances.
/// Stores use Arc for thread-safe sharing across async tasks.
pub struct AppState {
    pub document_store: Arc<dyn IDocumentStore>,
    pub vector_store: Arc<dyn IVectorStore>,
    pub memory_store: Arc<dyn IMemoryStore>,
    pub page_index: Arc<dyn IPageIndex>,
    pub graph_store: Arc<dyn IGraphStore>,
    pub timeline_store: Arc<dyn ITimelineStore>,
    pub llm_provider: Arc<RwLock<Box<dyn ILlmProvider>>>,
    pub embedding_provider: Arc<RwLock<Box<dyn IEmbeddingProvider>>>,
    pub config_store: Arc<SqliteConfigStore>,
    pub prompt_store: Arc<SqlitePromptStore>,
    pub keychain: Arc<KeychainAdapter>,
    pub db: Arc<SqliteConnection>,
}

impl AppState {
    /// Initialize AppState with default adapters (SQLite + Ollama).
    /// Loads saved provider config from the config store if available.
    /// Opens the database without encryption (backward compat).
    pub fn new(data_dir: PathBuf) -> Result<Self, AppError> {
        Self::new_with_passphrase(data_dir, "")
    }

    /// Initialize AppState with an encrypted database.
    /// An empty passphrase opens the database without encryption.
    pub fn new_with_passphrase(data_dir: PathBuf, passphrase: &str) -> Result<Self, AppError> {
        std::fs::create_dir_all(&data_dir)?;
        let db_path = data_dir.join("memory_palace.db");
        tracing::info!(db_path = %db_path.display(), encrypted = !passphrase.is_empty(), "Opening database");
        let db = if passphrase.is_empty() {
            Arc::new(SqliteConnection::open(&db_path)?)
        } else {
            Arc::new(SqliteConnection::open_encrypted(&db_path, passphrase)?)
        };

        let vector_store = SqliteVectorStore::new(db.clone())?;
        let config_store = Arc::new(SqliteConfigStore::new(db.clone()));
        let kc = Arc::new(KeychainAdapter::new());

        tracing::info!(keychain_available = kc.is_available(), "Keychain status");

        // Migrate plaintext API keys from config store to OS keychain (one-time)
        if kc.is_available() {
            for (config_key, kc_key) in [
                ("llm.claude_api_key", keychain::keys::CLAUDE_API_KEY),
                ("llm.openai_compat_api_key", keychain::keys::OPENAI_COMPAT_API_KEY),
            ] {
                if let Ok(Some(plaintext_key)) = config_store.get(config_key) {
                    if !plaintext_key.is_empty() {
                        // Only migrate if not already in keychain
                        if kc.get_secret(kc_key).ok().flatten().is_none() {
                            let _ = kc.store_secret(kc_key, &plaintext_key);
                            log::info!("Migrated {} to OS keychain", config_key);
                        }
                        // Remove plaintext key from config store
                        let _ = config_store.delete(config_key);
                    }
                }
            }
        }

        // Load saved LLM config or use defaults
        let ollama_url = config_store
            .get("llm.ollama_url")
            .ok()
            .flatten()
            .unwrap_or_else(|| "http://localhost:11434".to_string());
        let llm_model = config_store
            .get("llm.model")
            .ok()
            .flatten()
            .unwrap_or_else(|| "llama3.1:8b".to_string());
        let embed_model = config_store
            .get("llm.embedding_model")
            .ok()
            .flatten()
            .unwrap_or_else(|| "nomic-embed-text".to_string());

        // Check if user has configured Claude as their provider
        let active_provider = config_store
            .get("llm.active_provider")
            .ok()
            .flatten()
            .unwrap_or_else(|| "ollama".to_string());

        let llm_provider: Box<dyn ILlmProvider> = match active_provider.as_str() {
            "claude" => {
                let api_key = kc.get_secret(keychain::keys::CLAUDE_API_KEY).ok().flatten()
                    .or_else(|| config_store.get("llm.claude_api_key").ok().flatten());
                if let Some(api_key) = api_key {
                    let claude_model = config_store
                        .get("llm.claude_model")
                        .ok()
                        .flatten()
                        .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());
                    Box::new(ClaudeProvider::new(&api_key, &claude_model))
                } else {
                    Box::new(OllamaProvider::new(&ollama_url, &llm_model, &embed_model))
                }
            }
            "openai_compat" => {
                let base_url = config_store.get("llm.openai_compat_base_url").ok().flatten()
                    .unwrap_or_default();
                let api_key = kc.get_secret(keychain::keys::OPENAI_COMPAT_API_KEY).ok().flatten()
                    .or_else(|| config_store.get("llm.openai_compat_api_key").ok().flatten())
                    .unwrap_or_default();
                let model = config_store.get("llm.openai_compat_model").ok().flatten()
                    .unwrap_or_else(|| "gemini-2.5-flash".to_string());
                let provider_id = config_store.get("llm.openai_compat_provider_id").ok().flatten()
                    .unwrap_or_else(|| "openai_compat".to_string());
                Box::new(crate::adapters::llm::openai_compat::OpenAiCompatProvider::new(
                    &base_url, &api_key, &model, &provider_id,
                ))
            }
            _ => Box::new(OllamaProvider::new(&ollama_url, &llm_model, &embed_model)),
        };

        // Wrap the LLM provider with usage logging
        let llm_provider: Box<dyn ILlmProvider> =
            Box::new(UsageLoggingProvider::new(llm_provider, db.clone()));

        // Embedding provider: use configured provider, not always Ollama
        let embed_provider: Box<dyn IEmbeddingProvider> = if active_provider == "openai_compat" {
            let base_url = config_store.get("llm.openai_compat_base_url").ok().flatten()
                .unwrap_or_default();
            let api_key = kc.get_secret(keychain::keys::OPENAI_COMPAT_API_KEY).ok().flatten()
                .or_else(|| config_store.get("llm.openai_compat_api_key").ok().flatten())
                .unwrap_or_default();
            let compat_embed = config_store.get("llm.openai_compat_embedding_model").ok().flatten();
            let provider_id = config_store.get("llm.openai_compat_provider_id").ok().flatten()
                .unwrap_or_else(|| "openai_compat".to_string());

            if let Some(model) = compat_embed {
                Box::new(
                    crate::adapters::llm::openai_compat::OpenAiCompatProvider::new(
                        &base_url, &api_key, "", &provider_id,
                    )
                    .with_embedding_model(&model, 3072),
                )
            } else {
                Box::new(OllamaProvider::new(&ollama_url, &llm_model, &embed_model))
            }
        } else {
            Box::new(OllamaProvider::new(&ollama_url, &llm_model, &embed_model))
        };

        // Prompt store with seeded defaults
        let prompt_store = Arc::new(SqlitePromptStore::new(db.clone()));
        if let Err(e) = prompt_store.seed_defaults() {
            log::warn!("Failed to seed default prompts: {}", e);
        }

        tracing::info!(
            active_provider = %active_provider,
            ollama_url = %ollama_url,
            "AppState initialized"
        );

        Ok(Self {
            document_store: Arc::new(SqliteDocumentStore::new(db.clone())),
            vector_store: Arc::new(vector_store),
            memory_store: Arc::new(SqliteMemoryStore::new(db.clone())),
            page_index: Arc::new(SqliteFts5Index::new(db.clone())),
            graph_store: Arc::new(SqliteGraphStore::new(db.clone())),
            timeline_store: Arc::new(SqliteTimelineStore::new(db.clone())),
            llm_provider: Arc::new(RwLock::new(llm_provider)),
            embedding_provider: Arc::new(RwLock::new(embed_provider)),
            config_store,
            prompt_store,
            keychain: kc,
            db,
        })
    }
}
