pub mod adapters;
pub mod app_state;
pub mod commands;
pub mod domain;
pub mod error;
pub mod pipeline;
pub mod prompts;
pub mod query;

use app_state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");

            let state = AppState::new(data_dir)
                .expect("failed to initialize application state");

            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Import
            commands::import_obsidian,
            commands::import_markdown,
            commands::import_dayone,
            // Search
            commands::keyword_search,
            commands::semantic_search,
            commands::hybrid_search,
            commands::get_document_text,
            // RAG
            commands::ask,
            // Timeline + Insights
            commands::get_timeline_data,
            commands::get_memory_facts,
            commands::delete_memory_fact,
            // Analysis
            commands::run_analysis,
            // Entities
            commands::list_entities,
            commands::get_entity_graph,
            // Provider config
            commands::get_llm_config,
            commands::save_llm_config,
            // Evolution
            commands::get_evolution_data,
            // Embeddings
            commands::generate_embeddings,
            // Export
            commands::export_memory_json,
            commands::export_memory_markdown,
            // Settings
            commands::test_ollama_connection,
            commands::get_app_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
