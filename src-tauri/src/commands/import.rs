use std::path::Path;

use tauri::{AppHandle, Emitter, Manager};
use tokio_util::sync::CancellationToken;

use crate::adapters::sqlite::activity_store::ActivityEntry;
use crate::app_state::AppState;
use crate::pipeline::ingestion::orchestrator::{ImportSummary, IngestionOrchestrator};
use crate::pipeline::ingestion::source_adapters::dayone::DayOneAdapter;
use crate::pipeline::ingestion::source_adapters::markdown::MarkdownAdapter;
use crate::pipeline::ingestion::source_adapters::obsidian::ObsidianAdapter;
use crate::pipeline::ingestion::source_adapters::registry;
use crate::pipeline::ingestion::source_adapters::{SourceAdapter, SourceAdapterMeta};
use crate::pipeline::ingestion::zip_handler;
use crate::services::task_manager::TaskManager;

#[derive(Clone, serde::Serialize)]
struct ImportProgress {
    import_id: String,
    stage: String,
    current: usize,
    total: usize,
    message: String,
}

fn run_import(
    app_handle: &AppHandle,
    state: &AppState,
    adapter: &dyn SourceAdapter,
    path: &Path,
    import_id: &str,
    cancel_token: &CancellationToken,
) -> Result<ImportSummary, String> {
    // Check cancellation before starting
    if cancel_token.is_cancelled() {
        return Err("Task cancelled".to_string());
    }

    tracing::info!(adapter = adapter.name(), path = %path.display(), "Starting import");
    let handle = app_handle.clone();
    let id_owned = import_id.to_string();
    let ct = cancel_token.clone();
    let progress_cb: Box<dyn Fn(&str, usize, usize, &str) + Send> =
        Box::new(move |stage, current, total, message| {
            if ct.is_cancelled() { return; }
            let _ = handle.emit(
                "import-progress",
                ImportProgress {
                    import_id: id_owned.clone(),
                    stage: stage.to_string(),
                    current,
                    total,
                    message: message.to_string(),
                },
            );
        });

    // Report that parsing is starting
    let parse_handle = app_handle.clone();
    let adapter_name = adapter.name().to_string();
    let _ = parse_handle.emit(
        "import-progress",
        ImportProgress {
            import_id: import_id.to_string(),
            stage: "parsing".to_string(),
            current: 0,
            total: 0,
            message: format!("Scanning {} files...", adapter_name),
        },
    );

    // Parse synchronously (adapters walk the filesystem)
    let documents = adapter.parse(path).map_err(|e| e.to_string())?;
    let doc_count = documents.len();

    // Check cancellation after parse
    if cancel_token.is_cancelled() {
        return Err("Task cancelled".to_string());
    }

    let _ = app_handle.emit(
        "import-progress",
        ImportProgress {
            import_id: import_id.to_string(),
            stage: "parsing".to_string(),
            current: doc_count,
            total: doc_count,
            message: format!("Found {} documents from {}", doc_count, adapter_name),
        },
    );

    tracing::info!(documents = doc_count, adapter = adapter.name(), "Parsing complete");

    if doc_count == 0 {
        tracing::warn!(adapter = adapter.name(), "No documents found during import");
        return Ok(ImportSummary {
            documents_imported: 0,
            chunks_created: 0,
            embeddings_generated: 0,
            duplicates_skipped: 0,
            errors: vec![],
            duration_ms: 0,
        });
    }

    // Create embedding provider from user's config (Gemini, Ollama, etc.)
    let embed_provider = create_embedding_provider(&state);

    let mut orchestrator = IngestionOrchestrator::new(
        state.document_store.as_ref(),
        state.timeline_store.as_ref(),
        state.page_index.as_ref(),
    )
    .with_vector_store(state.vector_store.as_ref())
    .with_cancellation_token(cancel_token.clone());

    if let Some(provider) = embed_provider {
        orchestrator = orchestrator.with_embedding_provider(provider);
    }

    let result = tauri::async_runtime::block_on(
        orchestrator.ingest_documents(documents, Some(&progress_cb)),
    )
    .map_err(|e| {
        tracing::error!(error = %e, "Import failed");
        e.to_string()
    });

    if let Ok(ref summary) = result {
        tracing::info!(
            documents = summary.documents_imported,
            chunks = summary.chunks_created,
            embeddings = summary.embeddings_generated,
            duplicates_skipped = summary.duplicates_skipped,
            errors = summary.errors.len(),
            duration_ms = summary.duration_ms,
            "Import complete"
        );
    }

    result
}

#[tauri::command]
pub async fn import_obsidian(
    vault_path: String,
    app_handle: AppHandle,
) -> Result<ImportSummary, String> {
    let mgr = app_handle.state::<TaskManager>();
    let id = format!("{:x}", md5_hash(&vault_path));
    let token = mgr.register_task(&id, "import", &format!("Import Obsidian: {}", vault_path));
    let _permit = mgr.acquire_import_permit().await;
    let ah2 = app_handle.clone();
    let id_clone = id.clone();

    let result = tokio::task::spawn_blocking(move || {
        let state = app_handle.state::<AppState>();
        let adapter = ObsidianAdapter;
        run_import(&app_handle, &state, &adapter, Path::new(&vault_path), &id, &token)
    }).await.map_err(|e| format!("Task join error: {}", e))?;

    let mgr2 = ah2.state::<TaskManager>();
    match &result {
        Ok(_) => mgr2.complete_task(&id_clone, None),
        Err(e) => mgr2.complete_task(&id_clone, Some(e)),
    }
    result
}

#[tauri::command]
pub async fn import_markdown(
    dir_path: String,
    app_handle: AppHandle,
) -> Result<ImportSummary, String> {
    let mgr = app_handle.state::<TaskManager>();
    let id = format!("{:x}", md5_hash(&dir_path));
    let token = mgr.register_task(&id, "import", &format!("Import Markdown: {}", dir_path));
    let _permit = mgr.acquire_import_permit().await;
    let ah2 = app_handle.clone();
    let id_clone = id.clone();

    let result = tokio::task::spawn_blocking(move || {
        let state = app_handle.state::<AppState>();
        let adapter = MarkdownAdapter;
        run_import(&app_handle, &state, &adapter, Path::new(&dir_path), &id, &token)
    }).await.map_err(|e| format!("Task join error: {}", e))?;

    let mgr2 = ah2.state::<TaskManager>();
    match &result {
        Ok(_) => mgr2.complete_task(&id_clone, None),
        Err(e) => mgr2.complete_task(&id_clone, Some(e)),
    }
    result
}

#[tauri::command]
pub async fn import_dayone(
    file_path: String,
    app_handle: AppHandle,
) -> Result<ImportSummary, String> {
    let mgr = app_handle.state::<TaskManager>();
    let id = format!("{:x}", md5_hash(&file_path));
    let token = mgr.register_task(&id, "import", &format!("Import DayOne: {}", file_path));
    let _permit = mgr.acquire_import_permit().await;
    let ah2 = app_handle.clone();
    let id_clone = id.clone();

    let result = tokio::task::spawn_blocking(move || {
        let state = app_handle.state::<AppState>();
        let adapter = DayOneAdapter;
        run_import(&app_handle, &state, &adapter, Path::new(&file_path), &id, &token)
    }).await.map_err(|e| format!("Task join error: {}", e))?;

    let mgr2 = ah2.state::<TaskManager>();
    match &result {
        Ok(_) => mgr2.complete_task(&id_clone, None),
        Err(e) => mgr2.complete_task(&id_clone, Some(e)),
    }
    result
}

/// List all available source adapters with metadata (for the frontend import UI).
#[tauri::command]
pub fn list_sources() -> Vec<SourceAdapterMeta> {
    registry::all_adapter_metadata()
}

/// Detect the default profile folder for a given browser on the current OS.
/// Returns the path as a string if found, or null if not found.
#[tauri::command]
pub fn detect_browser_path(browser: String) -> Option<String> {
    let home = dirs::home_dir()?;
    let candidates: Vec<std::path::PathBuf> = match browser.as_str() {
        "chrome_history" => {
            #[cfg(target_os = "windows")]
            {
                let local = dirs::data_local_dir()?;
                vec![local.join("Google").join("Chrome").join("User Data").join("Default")]
            }
            #[cfg(target_os = "macos")]
            {
                vec![home.join("Library").join("Application Support").join("Google").join("Chrome").join("Default")]
            }
            #[cfg(target_os = "linux")]
            {
                vec![home.join(".config").join("google-chrome").join("Default")]
            }
        }
        "edge_history" => {
            #[cfg(target_os = "windows")]
            {
                let local = dirs::data_local_dir()?;
                vec![local.join("Microsoft").join("Edge").join("User Data").join("Default")]
            }
            #[cfg(target_os = "macos")]
            {
                vec![home.join("Library").join("Application Support").join("Microsoft Edge").join("Default")]
            }
            #[cfg(target_os = "linux")]
            {
                vec![home.join(".config").join("microsoft-edge").join("Default")]
            }
        }
        "firefox_history" => {
            #[cfg(target_os = "windows")]
            {
                let roaming = dirs::data_dir()?;
                let profiles_dir = roaming.join("Mozilla").join("Firefox").join("Profiles");
                return find_first_profile_dir(&profiles_dir);
            }
            #[cfg(target_os = "macos")]
            {
                let profiles_dir = home.join("Library").join("Application Support").join("Firefox").join("Profiles");
                return find_first_profile_dir(&profiles_dir);
            }
            #[cfg(target_os = "linux")]
            {
                let profiles_dir = home.join(".mozilla").join("firefox");
                return find_first_profile_dir(&profiles_dir);
            }
        }
        "safari_history" => {
            vec![home.join("Library").join("Safari")]
        }
        _ => return None,
    };
    candidates.into_iter().find(|p| p.exists()).map(|p| p.to_string_lossy().into_owned())
}

fn find_first_profile_dir(profiles_dir: &std::path::Path) -> Option<String> {
    if !profiles_dir.exists() { return None; }
    std::fs::read_dir(profiles_dir).ok()?.flatten()
        .find(|e| {
            let name = e.file_name();
            let s = name.to_string_lossy();
            e.path().is_dir() && (s.ends_with(".default-release") || s.ends_with(".default") || s.contains("default"))
        })
        .map(|e| e.path().to_string_lossy().into_owned())
}

/// Exploratory import: accepts any file/folder/zip path and an optional adapter_id.
#[tauri::command]
pub async fn import_source(
    path: String,
    adapter_id: Option<String>,
    import_id: Option<String>,
    app_handle: AppHandle,
) -> Result<ImportSummary, String> {
    let mgr = app_handle.state::<TaskManager>();
    let id = import_id.unwrap_or_else(|| format!("{:x}", md5_hash(&path)));
    let token = mgr.register_task(&id, "import", &format!("Import: {}", path));
    let _permit = mgr.acquire_import_permit().await;
    let ah2 = app_handle.clone();
    let id_clone = id.clone();

    let result = tokio::task::spawn_blocking(move || {
        let state = app_handle.state::<AppState>();
        import_source_blocking(&path, adapter_id, &app_handle, &state, &id, &token)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?;

    let mgr2 = ah2.state::<TaskManager>();
    match &result {
        Ok(_) => mgr2.complete_task(&id_clone, None),
        Err(e) => mgr2.complete_task(&id_clone, Some(e)),
    }
    result
}

fn md5_hash(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut hasher);
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis().hash(&mut hasher);
    hasher.finish()
}

fn import_source_blocking(
    path: &str,
    adapter_id: Option<String>,
    app_handle: &AppHandle,
    state: &AppState,
    import_id: &str,
    cancel_token: &CancellationToken,
) -> Result<ImportSummary, String> {
    let input_path = Path::new(path);

    // Check cancellation
    if cancel_token.is_cancelled() {
        return Err("Task cancelled".to_string());
    }

    // If it's a ZIP, extract first and build file listing for detection
    let (work_dir, file_listing, _temp_dir) = if input_path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
    {
        let (temp_dir, files) =
            zip_handler::extract_zip(input_path).map_err(|e| e.to_string())?;
        let listing: Vec<String> = files;
        (temp_dir.clone(), listing, Some(temp_dir))
    } else {
        let listing = if input_path.is_dir() {
            zip_handler::list_dir_contents(input_path).map_err(|e| e.to_string())?
        } else {
            vec![input_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()]
        };
        (input_path.to_path_buf(), listing, None)
    };

    let listing_refs: Vec<&str> = file_listing.iter().map(|s| s.as_str()).collect();

    // Log structural scan for transparency
    let file_count = file_listing.len();
    let ext_counts = count_extensions(&file_listing);
    log::info!(
        "Import scan: {} files found. Extensions: {:?}",
        file_count,
        ext_counts
    );

    // Emit scan info to frontend
    let _ = app_handle.emit(
        "import-progress",
        ImportProgress {
            import_id: import_id.to_string(),
            stage: "scanning".to_string(),
            current: 0,
            total: file_count,
            message: format!(
                "Found {} files ({})",
                file_count,
                ext_counts
                    .iter()
                    .map(|(ext, count)| format!("{} {}", count, ext))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        },
    );

    // Find primary adapter
    let primary_adapter: Box<dyn SourceAdapter> = if let Some(id) = adapter_id {
        registry::all_adapters()
            .into_iter()
            .find(|a| a.metadata().id == id)
            .ok_or_else(|| format!("Unknown adapter: {}", id))?
    } else {
        registry::detect_adapter(&listing_refs)
            .ok_or_else(|| {
                "Could not detect source format. Try selecting the source type manually."
                    .to_string()
            })?
    };

    let primary_name = primary_adapter.metadata().display_name.clone();
    let primary_id = primary_adapter.metadata().id.clone();

    // Pass 1: Run platform-specific adapter
    log::info!("Pass 1: Running {} adapter", primary_name);
    let mut result = run_import(&app_handle, &state, primary_adapter.as_ref(), &work_dir, import_id, cancel_token)?;

    // Check cancellation between passes
    if cancel_token.is_cancelled() {
        return Err("Task cancelled".to_string());
    }

    // Pass 2: Sweep remaining files with GenericAdapter (unless primary IS generic)
    if primary_id != "generic" && primary_id != "markdown" {
        log::info!(
            "Pass 2: Generic sweep for remaining text files (primary found {} docs)",
            result.documents_imported
        );

        let _ = app_handle.emit(
            "import-progress",
            ImportProgress {
                import_id: import_id.to_string(),
                stage: "sweep".to_string(),
                current: 0,
                total: 0,
                message: format!(
                    "Sweeping for additional files missed by {} adapter...",
                    primary_name
                ),
            },
        );

        let generic = crate::pipeline::ingestion::source_adapters::generic::GenericAdapter;
        match run_import(&app_handle, &state, &generic, &work_dir, import_id, cancel_token) {
            Ok(sweep) => {
                result.documents_imported += sweep.documents_imported;
                result.chunks_created += sweep.chunks_created;
                result.embeddings_generated += sweep.embeddings_generated;
                result.duplicates_skipped += sweep.duplicates_skipped;
                result.errors.extend(sweep.errors);
                result.duration_ms += sweep.duration_ms;

                if sweep.documents_imported > 0 {
                    log::info!(
                        "Sweep found {} additional documents ({} were duplicates)",
                        sweep.documents_imported,
                        sweep.duplicates_skipped
                    );
                }
            }
            Err(e) if e.contains("cancelled") => {
                return Err(e);
            }
            Err(e) => {
                log::warn!("Generic sweep failed (non-fatal): {}", e);
                result.errors.push(format!("Sweep: {}", e));
            }
        }
    }

    // Log activity
    let _ = state.activity_store.log_activity(&ActivityEntry {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        action_type: "import".to_string(),
        title: format!("Imported from {}", primary_name),
        description: format!("Source: {}", path),
        result_summary: format!(
            "{} docs, {} chunks",
            result.documents_imported, result.chunks_created
        ),
        metadata: serde_json::json!({
            "adapter": primary_id,
            "duplicates_skipped": result.duplicates_skipped,
            "errors": result.errors.len(),
        }),
        duration_ms: result.duration_ms as i64,
        status: if result.errors.is_empty() { "success".to_string() } else { "warning".to_string() },
    });

    // Emit complete event — frontend will prompt user to run analysis
    if result.documents_imported > 0 {
        let _ = app_handle.emit("import-progress", ImportProgress {
            import_id: import_id.to_string(),
            stage: "complete".to_string(),
            current: result.documents_imported,
            total: result.documents_imported,
            message: format!(
                "Import complete: {} documents ready for analysis",
                result.documents_imported
            ),
        });
    }

    Ok(result)
}

/// Create an embedding provider from the user's saved config.
/// Returns None if no embedding provider is configured or available.
fn create_embedding_provider(
    state: &AppState,
) -> Option<std::sync::Arc<dyn crate::domain::ports::embedding_provider::IEmbeddingProvider>> {
    let cs = &state.config_store;
    let active = cs.get("llm.active_provider").ok().flatten().unwrap_or_default();

    if active == "openai_compat" {
        let base_url = cs.get("llm.openai_compat_base_url").ok().flatten()?;
        let api_key = state.keychain.get_secret(crate::adapters::keychain::keys::OPENAI_COMPAT_API_KEY).ok().flatten()
            .or_else(|| cs.get("llm.openai_compat_api_key").ok().flatten())
            .unwrap_or_default();
        let embed_model = cs.get("llm.openai_compat_embedding_model").ok().flatten()?;
        let provider_id = cs.get("llm.openai_compat_provider_id").ok().flatten()
            .unwrap_or_else(|| "openai_compat".into());

        let provider = crate::adapters::llm::openai_compat::OpenAiCompatProvider::new(
            &base_url, &api_key, "", &provider_id,
        ).with_embedding_model(&embed_model, 3072);

        tracing::info!(provider = %provider_id, model = %embed_model, "Using configured embedding provider for import");
        Some(std::sync::Arc::new(provider))
    } else {
        // Default: try Ollama
        let ollama_url = cs.get("llm.ollama_url").ok().flatten()
            .unwrap_or_else(|| "http://localhost:11434".into());
        let model = cs.get("llm.model").ok().flatten()
            .unwrap_or_else(|| "llama3.1:8b".into());
        let embed_model = cs.get("llm.embedding_model").ok().flatten()
            .unwrap_or_else(|| "nomic-embed-text".into());

        tracing::info!(model = %embed_model, "Using Ollama embedding provider for import");
        Some(std::sync::Arc::new(
            crate::adapters::llm::ollama::OllamaProvider::new(&ollama_url, &model, &embed_model),
        ))
    }
}

/// Count file extensions in a listing for structural reporting.
fn count_extensions(files: &[String]) -> Vec<(String, usize)> {
    let mut counts = std::collections::HashMap::new();
    for f in files {
        let ext = f
            .rsplit('.')
            .next()
            .unwrap_or("other")
            .to_lowercase();
        *counts.entry(ext).or_insert(0usize) += 1;
    }
    let mut sorted: Vec<_> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted.truncate(8);
    sorted
}
