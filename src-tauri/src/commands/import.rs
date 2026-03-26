use std::path::Path;

use tauri::{AppHandle, Emitter, State};

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

    // Always wire embedding provider — if Ollama isn't running, embedding errors are
    // collected in ImportSummary.errors but don't block the import.
    let embed_provider: std::sync::Arc<dyn crate::domain::ports::embedding_provider::IEmbeddingProvider> =
        std::sync::Arc::new(crate::adapters::llm::ollama::OllamaProvider::new(
            "http://localhost:11434",
            "llama3.1:8b",
            "nomic-embed-text",
        ));

    let orchestrator = IngestionOrchestrator::new(
        state.document_store.as_ref(),
        state.timeline_store.as_ref(),
        state.page_index.as_ref(),
    )
    .with_vector_store(state.vector_store.as_ref())
    .with_embedding_provider(embed_provider);

    let result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(
            orchestrator.ingest_documents(documents, Some(&progress_cb)),
        )
    })
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

    Ok(result)
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
