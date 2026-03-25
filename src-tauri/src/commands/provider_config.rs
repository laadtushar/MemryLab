use tauri::State;

use crate::adapters::llm::claude::ClaudeProvider;
use crate::adapters::llm::ollama::OllamaProvider;
use crate::adapters::llm::openai_compat::OpenAiCompatProvider;
use crate::app_state::AppState;

/// Persisted LLM configuration.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct LlmConfig {
    pub active_provider: String,
    // Ollama (local)
    pub ollama_url: String,
    pub ollama_model: String,
    pub embedding_model: String,
    // Claude
    pub claude_api_key: Option<String>,
    pub claude_model: String,
    // Universal OpenAI-compatible provider
    pub openai_compat_base_url: Option<String>,
    pub openai_compat_api_key: Option<String>,
    pub openai_compat_model: Option<String>,
    pub openai_compat_embedding_model: Option<String>,
    pub openai_compat_provider_id: Option<String>,
}

/// A provider preset for quick setup.
#[derive(serde::Serialize)]
pub struct ProviderPreset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub base_url: String,
    pub signup_url: String,
    pub free_tier: bool,
    pub default_model: String,
    pub embedding_model: Option<String>,
    pub embedding_dimensions: Option<usize>,
    pub models: Vec<PresetModel>,
    pub rate_limits: String,
    pub supports_embeddings: bool,
}

#[derive(serde::Serialize)]
pub struct PresetModel {
    pub id: String,
    pub name: String,
    pub free: bool,
}

/// Get the current LLM configuration.
#[tauri::command]
pub fn get_llm_config(state: State<'_, AppState>) -> Result<LlmConfig, String> {
    let cs = &state.config_store;

    let active = cs.get("llm.active_provider").ok().flatten().unwrap_or_else(|| "ollama".into());
    let ollama_url = cs.get("llm.ollama_url").ok().flatten().unwrap_or_else(|| "http://localhost:11434".into());
    let ollama_model = cs.get("llm.model").ok().flatten().unwrap_or_else(|| "llama3.1:8b".into());
    let embed_model = cs.get("llm.embedding_model").ok().flatten().unwrap_or_else(|| "nomic-embed-text".into());
    let claude_key = cs.get("llm.claude_api_key").ok().flatten();
    let claude_model = cs.get("llm.claude_model").ok().flatten().unwrap_or_else(|| "claude-sonnet-4-20250514".into());

    let compat_base_url = cs.get("llm.openai_compat_base_url").ok().flatten();
    let compat_api_key = cs.get("llm.openai_compat_api_key").ok().flatten();
    let compat_model = cs.get("llm.openai_compat_model").ok().flatten();
    let compat_embed_model = cs.get("llm.openai_compat_embedding_model").ok().flatten();
    let compat_provider_id = cs.get("llm.openai_compat_provider_id").ok().flatten();

    Ok(LlmConfig {
        active_provider: active,
        ollama_url,
        ollama_model,
        embedding_model: embed_model,
        claude_api_key: claude_key.map(|k| mask_api_key(&k)),
        claude_model,
        openai_compat_base_url: compat_base_url,
        openai_compat_api_key: compat_api_key.map(|k| mask_api_key(&k)),
        openai_compat_model: compat_model,
        openai_compat_embedding_model: compat_embed_model,
        openai_compat_provider_id: compat_provider_id,
    })
}

/// Save LLM configuration and switch the active provider at runtime.
#[tauri::command]
pub fn save_llm_config(
    config: LlmConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let cs = &state.config_store;

    cs.set("llm.active_provider", &config.active_provider).map_err(|e| e.to_string())?;
    cs.set("llm.ollama_url", &config.ollama_url).map_err(|e| e.to_string())?;
    cs.set("llm.model", &config.ollama_model).map_err(|e| e.to_string())?;
    cs.set("llm.embedding_model", &config.embedding_model).map_err(|e| e.to_string())?;
    cs.set("llm.claude_model", &config.claude_model).map_err(|e| e.to_string())?;

    // Only update API keys if non-masked values provided
    if let Some(ref key) = config.claude_api_key {
        if !key.contains('*') && !key.is_empty() {
            cs.set("llm.claude_api_key", key).map_err(|e| e.to_string())?;
        }
    }

    // OpenAI-compat fields
    if let Some(ref url) = config.openai_compat_base_url {
        cs.set("llm.openai_compat_base_url", url).map_err(|e| e.to_string())?;
    }
    if let Some(ref key) = config.openai_compat_api_key {
        if !key.contains('*') && !key.is_empty() {
            cs.set("llm.openai_compat_api_key", key).map_err(|e| e.to_string())?;
        }
    }
    if let Some(ref model) = config.openai_compat_model {
        cs.set("llm.openai_compat_model", model).map_err(|e| e.to_string())?;
    }
    if let Some(ref model) = config.openai_compat_embedding_model {
        cs.set("llm.openai_compat_embedding_model", model).map_err(|e| e.to_string())?;
    }
    if let Some(ref id) = config.openai_compat_provider_id {
        cs.set("llm.openai_compat_provider_id", id).map_err(|e| e.to_string())?;
    }

    // Swap the active LLM provider at runtime
    let new_provider: Box<dyn crate::domain::ports::llm_provider::ILlmProvider> =
        match config.active_provider.as_str() {
            "claude" => {
                let real_key = cs
                    .get("llm.claude_api_key")
                    .ok()
                    .flatten()
                    .ok_or("Claude API key not set")?;
                Box::new(ClaudeProvider::new(&real_key, &config.claude_model))
            }
            "openai_compat" => {
                let base_url = cs.get("llm.openai_compat_base_url").ok().flatten()
                    .ok_or("OpenAI-compatible base URL not set")?;
                let api_key = cs.get("llm.openai_compat_api_key").ok().flatten()
                    .unwrap_or_default();
                let model = cs.get("llm.openai_compat_model").ok().flatten()
                    .ok_or("OpenAI-compatible model not set")?;
                let provider_id = cs.get("llm.openai_compat_provider_id").ok().flatten()
                    .unwrap_or_else(|| "openai_compat".into());
                Box::new(OpenAiCompatProvider::new(&base_url, &api_key, &model, &provider_id))
            }
            _ => {
                // Default: Ollama
                Box::new(OllamaProvider::new(
                    &config.ollama_url,
                    &config.ollama_model,
                    &config.embedding_model,
                ))
            }
        };

    let mut provider = state
        .llm_provider
        .write()
        .map_err(|e| format!("Lock error: {}", e))?;
    *provider = new_provider;

    // Update embedding provider
    let new_embed: Box<dyn crate::domain::ports::embedding_provider::IEmbeddingProvider> =
        if config.active_provider == "openai_compat" {
            let base_url = cs.get("llm.openai_compat_base_url").ok().flatten().unwrap_or_default();
            let api_key = cs.get("llm.openai_compat_api_key").ok().flatten().unwrap_or_default();
            let embed_model = cs.get("llm.openai_compat_embedding_model").ok().flatten();
            let provider_id = cs.get("llm.openai_compat_provider_id").ok().flatten()
                .unwrap_or_else(|| "openai_compat".into());

            if let Some(model) = embed_model {
                Box::new(
                    OpenAiCompatProvider::new(&base_url, &api_key, "", &provider_id)
                        .with_embedding_model(&model, 768),
                )
            } else {
                // Fall back to Ollama for embeddings
                Box::new(OllamaProvider::new(
                    &config.ollama_url,
                    &config.ollama_model,
                    &config.embedding_model,
                ))
            }
        } else {
            Box::new(OllamaProvider::new(
                &config.ollama_url,
                &config.ollama_model,
                &config.embedding_model,
            ))
        };

    let mut embed = state
        .embedding_provider
        .write()
        .map_err(|e| format!("Lock error: {}", e))?;
    *embed = new_embed;

    Ok(())
}

/// List all available provider presets for the UI.
#[tauri::command]
pub fn list_provider_presets() -> Vec<ProviderPreset> {
    vec![
        ProviderPreset {
            id: "ollama".into(),
            name: "Ollama (Local)".into(),
            description: "Run models locally on your machine. Free, private, no internet needed.".into(),
            base_url: "http://localhost:11434".into(),
            signup_url: "https://ollama.com/download".into(),
            free_tier: true,
            default_model: "llama3.1:8b".into(),
            embedding_model: Some("nomic-embed-text".into()),
            embedding_dimensions: Some(768),
            models: vec![
                PresetModel { id: "llama3.1:8b".into(), name: "Llama 3.1 8B".into(), free: true },
                PresetModel { id: "llama3.1:70b".into(), name: "Llama 3.1 70B".into(), free: true },
                PresetModel { id: "mistral:7b".into(), name: "Mistral 7B".into(), free: true },
                PresetModel { id: "phi3:mini".into(), name: "Phi-3 Mini".into(), free: true },
                PresetModel { id: "qwen2.5:7b".into(), name: "Qwen 2.5 7B".into(), free: true },
            ],
            rate_limits: "Unlimited (local hardware)".into(),
            supports_embeddings: true,
        },
        ProviderPreset {
            id: "openrouter".into(),
            name: "OpenRouter".into(),
            description: "29 free models from multiple providers. Best single integration point.".into(),
            base_url: "https://openrouter.ai/api/v1".into(),
            signup_url: "https://openrouter.ai/".into(),
            free_tier: true,
            default_model: "meta-llama/llama-3.3-70b-instruct:free".into(),
            embedding_model: None,
            embedding_dimensions: None,
            models: vec![
                PresetModel { id: "meta-llama/llama-3.3-70b-instruct:free".into(), name: "Llama 3.3 70B".into(), free: true },
                PresetModel { id: "google/gemini-2.5-pro-exp-03-25:free".into(), name: "Gemini 2.5 Pro".into(), free: true },
                PresetModel { id: "mistralai/mistral-small-3.1-24b-instruct:free".into(), name: "Mistral Small 3.1".into(), free: true },
                PresetModel { id: "qwen/qwen3-coder-480b:free".into(), name: "Qwen3 Coder 480B".into(), free: true },
                PresetModel { id: "openai/gpt-4o-mini:free".into(), name: "GPT-4o Mini".into(), free: true },
                PresetModel { id: "nvidia/llama-3.1-nemotron-3-super-49b-v1:free".into(), name: "Nemotron Super 49B".into(), free: true },
            ],
            rate_limits: "~20 req/min, ~200 req/day per free model".into(),
            supports_embeddings: false,
        },
        ProviderPreset {
            id: "groq".into(),
            name: "Groq".into(),
            description: "Ultra-fast inference on LPU hardware. Generous free tier.".into(),
            base_url: "https://api.groq.com/openai/v1".into(),
            signup_url: "https://console.groq.com/".into(),
            free_tier: true,
            default_model: "llama-3.3-70b-versatile".into(),
            embedding_model: None,
            embedding_dimensions: None,
            models: vec![
                PresetModel { id: "llama-3.3-70b-versatile".into(), name: "Llama 3.3 70B".into(), free: true },
                PresetModel { id: "llama-3.1-8b-instant".into(), name: "Llama 3.1 8B Instant".into(), free: true },
                PresetModel { id: "llama-4-scout-17b-16e-instruct".into(), name: "Llama 4 Scout 17B".into(), free: true },
                PresetModel { id: "mixtral-8x7b-32768".into(), name: "Mixtral 8x7B".into(), free: true },
                PresetModel { id: "gemma2-9b-it".into(), name: "Gemma2 9B".into(), free: true },
            ],
            rate_limits: "30 RPM, ~1,000 req/day (70B)".into(),
            supports_embeddings: false,
        },
        ProviderPreset {
            id: "gemini".into(),
            name: "Google Gemini".into(),
            description: "Most generous free tier. Chat + embeddings. No credit card needed.".into(),
            base_url: "https://generativelanguage.googleapis.com/v1beta/openai".into(),
            signup_url: "https://aistudio.google.com/".into(),
            free_tier: true,
            default_model: "gemini-2.5-flash".into(),
            embedding_model: Some("text-embedding-004".into()),
            embedding_dimensions: Some(768),
            models: vec![
                PresetModel { id: "gemini-2.5-flash".into(), name: "Gemini 2.5 Flash".into(), free: true },
                PresetModel { id: "gemini-2.5-flash-lite".into(), name: "Gemini 2.5 Flash Lite".into(), free: true },
                PresetModel { id: "gemini-2.5-pro".into(), name: "Gemini 2.5 Pro".into(), free: true },
            ],
            rate_limits: "10 RPM (Flash), 5 RPM (Pro), 250K tokens/min".into(),
            supports_embeddings: true,
        },
        ProviderPreset {
            id: "cerebras".into(),
            name: "Cerebras".into(),
            description: "Blazing fast inference (3000+ tok/s). 1M tokens/day free.".into(),
            base_url: "https://api.cerebras.ai/v1".into(),
            signup_url: "https://cloud.cerebras.ai/".into(),
            free_tier: true,
            default_model: "llama-3.3-70b".into(),
            embedding_model: None,
            embedding_dimensions: None,
            models: vec![
                PresetModel { id: "llama-3.3-70b".into(), name: "Llama 3.3 70B".into(), free: true },
                PresetModel { id: "llama3.1-8b".into(), name: "Llama 3.1 8B".into(), free: true },
                PresetModel { id: "qwen-3-32b".into(), name: "Qwen 3 32B".into(), free: true },
                PresetModel { id: "deepseek-r1-distill-llama-70b".into(), name: "DeepSeek R1 70B".into(), free: true },
            ],
            rate_limits: "1M tokens/day, varies by model".into(),
            supports_embeddings: false,
        },
        ProviderPreset {
            id: "mistral".into(),
            name: "Mistral".into(),
            description: "French AI lab. Free experiment plan with chat + embeddings.".into(),
            base_url: "https://api.mistral.ai/v1".into(),
            signup_url: "https://console.mistral.ai/".into(),
            free_tier: true,
            default_model: "mistral-small-latest".into(),
            embedding_model: Some("mistral-embed".into()),
            embedding_dimensions: Some(1024),
            models: vec![
                PresetModel { id: "mistral-small-latest".into(), name: "Mistral Small".into(), free: true },
                PresetModel { id: "open-mistral-nemo".into(), name: "Mistral Nemo (Open)".into(), free: true },
                PresetModel { id: "pixtral-12b-2409".into(), name: "Pixtral 12B".into(), free: true },
            ],
            rate_limits: "1 RPS, 500K tokens/min, 1B tokens/month".into(),
            supports_embeddings: true,
        },
        ProviderPreset {
            id: "sambanova".into(),
            name: "SambaNova".into(),
            description: "Free access to Llama 405B — the largest open model. Indefinite free tier.".into(),
            base_url: "https://api.sambanova.ai/v1".into(),
            signup_url: "https://cloud.sambanova.ai/".into(),
            free_tier: true,
            default_model: "Meta-Llama-3.3-70B-Instruct".into(),
            embedding_model: None,
            embedding_dimensions: None,
            models: vec![
                PresetModel { id: "Meta-Llama-3.3-70B-Instruct".into(), name: "Llama 3.3 70B".into(), free: true },
                PresetModel { id: "Meta-Llama-3.1-8B-Instruct".into(), name: "Llama 3.1 8B".into(), free: true },
                PresetModel { id: "Meta-Llama-3.1-405B-Instruct".into(), name: "Llama 3.1 405B".into(), free: true },
                PresetModel { id: "DeepSeek-R1-0528".into(), name: "DeepSeek R1".into(), free: true },
            ],
            rate_limits: "10-30 RPM by model".into(),
            supports_embeddings: false,
        },
        ProviderPreset {
            id: "cohere".into(),
            name: "Cohere".into(),
            description: "Best free embeddings. Also has chat and reranking.".into(),
            base_url: "https://api.cohere.ai/compatibility/v1".into(),
            signup_url: "https://dashboard.cohere.com/".into(),
            free_tier: true,
            default_model: "command-a-03-2025".into(),
            embedding_model: Some("embed-v4.0".into()),
            embedding_dimensions: Some(1024),
            models: vec![
                PresetModel { id: "command-a-03-2025".into(), name: "Command A".into(), free: true },
                PresetModel { id: "command-r-plus".into(), name: "Command R+".into(), free: true },
            ],
            rate_limits: "1,000 API calls/month, 5 RPM embed".into(),
            supports_embeddings: true,
        },
        ProviderPreset {
            id: "claude".into(),
            name: "Claude (Anthropic)".into(),
            description: "Best reasoning quality. Requires API key (pay-per-use).".into(),
            base_url: "https://api.anthropic.com".into(),
            signup_url: "https://console.anthropic.com/".into(),
            free_tier: false,
            default_model: "claude-sonnet-4-20250514".into(),
            embedding_model: None,
            embedding_dimensions: None,
            models: vec![
                PresetModel { id: "claude-sonnet-4-20250514".into(), name: "Claude Sonnet 4".into(), free: false },
                PresetModel { id: "claude-haiku-4-5-20251001".into(), name: "Claude Haiku 4.5".into(), free: false },
            ],
            rate_limits: "Pay-per-use (~$3/M input tokens Sonnet)".into(),
            supports_embeddings: false,
        },
    ]
}

fn mask_api_key(key: &str) -> String {
    if key.len() <= 8 {
        "*".repeat(key.len())
    } else {
        format!("{}...{}", &key[..4], &key[key.len() - 4..])
    }
}
