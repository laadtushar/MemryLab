use std::path::Path;

use tauri::{AppHandle, Emitter, State};

use crate::app_state::AppState;
use crate::pipeline::ingestion::orchestrator::{ImportSummary, IngestionOrchestrator};
use crate::pipeline::ingestion::source_adapters::dayone::DayOneAdapter;
use crate::pipeline::ingestion::source_adapters::markdown::MarkdownAdapter;
use crate::pipeline::ingestion::source_adapters::obsidian::ObsidianAdapter;
use crate::pipeline::ingestion::source_adapters::SourceAdapter;

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
