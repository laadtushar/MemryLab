use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::adapters::keychain::{self, KeychainAdapter};
use crate::adapters::llm::claude::ClaudeProvider;
use crate::adapters::llm::ollama::OllamaProvider;
use crate::adapters::sqlite::config_store::SqliteConfigStore;
use crate::adapters::sqlite::connection::SqliteConnection;
use crate::adapters::sqlite::document_store::SqliteDocumentStore;
use crate::adapters::sqlite::graph_store::SqliteGraphStore;
use crate::adapters::sqlite::memory_store::SqliteMemoryStore;
use crate::adapters::sqlite::page_index::SqliteFts5Index;
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
pub struct AppState {
    pub document_store: Box<dyn IDocumentStore>,
    pub vector_store: Box<dyn IVectorStore>,
    pub memory_store: Box<dyn IMemoryStore>,
    pub page_index: Box<dyn IPageIndex>,
    pub graph_store: Box<dyn IGraphStore>,
    pub timeline_store: Box<dyn ITimelineStore>,
    pub llm_provider: Arc<RwLock<Box<dyn ILlmProvider>>>,
    pub embedding_provider: Arc<RwLock<Box<dyn IEmbeddingProvider>>>,
    pub config_store: Arc<SqliteConfigStore>,
    pub keychain: Arc<KeychainAdapter>,
}

impl AppState {
    /// Initialize AppState with default adapters (SQLite + Ollama).
    /// Loads saved provider config from the config store if available.
    pub fn new(data_dir: PathBuf) -> Result<Self, AppError> {
        std::fs::create_dir_all(&data_dir)?;
        let db_path = data_dir.join("memory_palace.db");
        let db = Arc::new(SqliteConnection::open(&db_path)?);

        let vector_store = SqliteVectorStore::new(db.clone())?;
        let config_store = Arc::new(SqliteConfigStore::new(db.clone()));
        let kc = Arc::new(KeychainAdapter::new());

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

        let llm_provider: Box<dyn ILlmProvider> = if active_provider == "claude" {
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
                // Fallback to Ollama if no API key
                Box::new(OllamaProvider::new(&ollama_url, &llm_model, &embed_model))
            }
        } else {
            Box::new(OllamaProvider::new(&ollama_url, &llm_model, &embed_model))
        };

        // Embedding provider is always local (Ollama) for privacy
        let ollama_embed: Box<dyn IEmbeddingProvider> =
            Box::new(OllamaProvider::new(&ollama_url, &llm_model, &embed_model));

        Ok(Self {
            document_store: Box::new(SqliteDocumentStore::new(db.clone())),
            vector_store: Box::new(vector_store),
            memory_store: Box::new(SqliteMemoryStore::new(db.clone())),
            page_index: Box::new(SqliteFts5Index::new(db.clone())),
            graph_store: Box::new(SqliteGraphStore::new(db.clone())),
            timeline_store: Box::new(SqliteTimelineStore::new(db)),
            llm_provider: Arc::new(RwLock::new(llm_provider)),
            embedding_provider: Arc::new(RwLock::new(ollama_embed)),
            config_store,
            keychain: kc,
        })
    }
}
