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

    tauri::async_runtime::block_on(
        orchestrator.ingest(adapter, path, Some(&progress_cb)),
    )
    .map_err(|e| e.to_string())
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

/// Generic import: accepts any file/folder/zip path and an optional adapter_id.
/// If adapter_id is provided, uses that adapter. Otherwise, auto-detects from file contents.
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

    // Find adapter
    let adapter: Box<dyn SourceAdapter> = if let Some(id) = adapter_id {
        registry::all_adapters()
            .into_iter()
            .find(|a| a.metadata().id == id)
            .ok_or_else(|| format!("Unknown adapter: {}", id))?
    } else {
        registry::detect_adapter(&listing_refs)
            .ok_or_else(|| "Could not detect source format. Try selecting the source type manually.".to_string())?
    };

    run_import(&app_handle, &state, adapter.as_ref(), &work_dir)
}
