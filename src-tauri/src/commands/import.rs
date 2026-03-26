use std::path::Path;

use tauri::{AppHandle, Emitter, State};

use crate::adapters::sqlite::activity_store::ActivityEntry;
use crate::app_state::AppState;
use crate::pipeline::ingestion::orchestrator::{ImportSummary, IngestionOrchestrator};
use crate::pipeline::ingestion::source_adapters::dayone::DayOneAdapter;
use crate::pipeline::ingestion::source_adapters::markdown::MarkdownAdapter;
use crate::pipeline::ingestion::source_adapters::obsidian::ObsidianAdapter;
use crate::pipeline::ingestion::source_adapters::registry;
use crate::pipeline::ingestion::source_adapters::{SourceAdapter, SourceAdapterMeta};
use crate::pipeline::ingestion::zip_handler;

#[derive(Clone, serde::Serialize)]
struct ImportProgress {
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
) -> Result<ImportSummary, String> {
    tracing::info!(adapter = adapter.name(), path = %path.display(), "Starting import");
    let handle = app_handle.clone();
    let progress_cb: Box<dyn Fn(&str, usize, usize, &str) + Send> =
        Box::new(move |stage, current, total, message| {
            let _ = handle.emit(
                "import-progress",
                ImportProgress {
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
            stage: "parsing".to_string(),
            current: 0,
            total: 0,
            message: format!("Scanning {} files...", adapter_name),
        },
    );

    // Parse synchronously (adapters walk the filesystem)
    let documents = adapter.parse(path).map_err(|e| e.to_string())?;
    let doc_count = documents.len();

    let _ = app_handle.emit(
        "import-progress",
        ImportProgress {
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
    .with_vector_store(state.vector_store.as_ref());

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
pub fn import_obsidian(
    vault_path: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<ImportSummary, String> {
    let adapter = ObsidianAdapter;
    run_import(&app_handle, &state, &adapter, Path::new(&vault_path))
}

#[tauri::command]
pub fn import_markdown(
    dir_path: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<ImportSummary, String> {
    let adapter = MarkdownAdapter;
    run_import(&app_handle, &state, &adapter, Path::new(&dir_path))
}

#[tauri::command]
pub fn import_dayone(
    file_path: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<ImportSummary, String> {
    let adapter = DayOneAdapter;
    run_import(&app_handle, &state, &adapter, Path::new(&file_path))
}

/// List all available source adapters with metadata (for the frontend import UI).
#[tauri::command]
pub fn list_sources() -> Vec<SourceAdapterMeta> {
    registry::all_adapter_metadata()
}

/// Exploratory import: accepts any file/folder/zip path and an optional adapter_id.
///
/// Two-pass strategy:
/// 1. Run the best-matching platform adapter (or selected one) to parse platform-specific formats
/// 2. Run the GenericAdapter as a sweep to catch ALL remaining text files the primary adapter missed
/// 3. Deduplication (by content hash) ensures no double-counting
///
/// This means ANY folder structure — no matter how nested — gets fully explored.
#[tauri::command]
pub fn import_source(
    path: String,
    adapter_id: Option<String>,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<ImportSummary, String> {
    let input_path = Path::new(&path);

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
    let mut result = run_import(&app_handle, &state, primary_adapter.as_ref(), &work_dir)?;

    // Pass 2: Sweep remaining files with GenericAdapter (unless primary IS generic)
    if primary_id != "generic" && primary_id != "markdown" {
        log::info!(
            "Pass 2: Generic sweep for remaining text files (primary found {} docs)",
            result.documents_imported
        );

        let _ = app_handle.emit(
            "import-progress",
            ImportProgress {
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
        match run_import(&app_handle, &state, &generic, &work_dir) {
            Ok(sweep) => {
                // Merge results — duplicates are already handled by content hash dedup
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

    // Auto-trigger analysis if documents were imported
    if result.documents_imported > 0 {
        let _ = app_handle.emit("import-progress", ImportProgress {
            stage: "analysis".to_string(),
            current: 0,
            total: 0,
            message: "Running analysis on imported documents...".to_string(),
        });

        // Run analysis in the same call — it will block but the import is already done
        if let Ok(llm) = state.llm_provider.read() {
            let analysis_result = tauri::async_runtime::block_on(
                crate::pipeline::analysis::orchestrator::run_analysis(
                    state.document_store.as_ref(),
                    state.timeline_store.as_ref(),
                    state.memory_store.as_ref(),
                    state.graph_store.as_ref(),
                    llm.as_ref(),
                    None,
                )
            );
            match analysis_result {
                Ok(ar) => {
                    tracing::info!(
                        themes = ar.themes_extracted,
                        beliefs = ar.beliefs_extracted,
                        entities = ar.entities_extracted,
                        "Auto-analysis after import complete"
                    );
                    let _ = app_handle.emit("import-progress", ImportProgress {
                        stage: "analysis-complete".to_string(),
                        current: 0,
                        total: 0,
                        message: format!(
                            "Analysis: {} themes, {} beliefs, {} entities extracted",
                            ar.themes_extracted, ar.beliefs_extracted, ar.entities_extracted
                        ),
                    });
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Auto-analysis failed (non-fatal)");
                }
            }
        }
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
    sorted.truncate(8); // Top 8 extensions
    sorted
}
