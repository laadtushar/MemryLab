pub mod adapters;
pub mod app_state;
pub mod commands;
pub mod domain;
pub mod error;
pub mod pipeline;
pub mod prompts;
pub mod query;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            // Auth (no AppState required)
            commands::is_first_run,
            commands::is_database_locked,
            commands::unlock_database,
            commands::set_passphrase,
            // Import
            commands::import_obsidian,
            commands::import_markdown,
            commands::import_dayone,
            commands::import_source,
            commands::list_sources,
            // Search
            commands::keyword_search,
            commands::semantic_search,
            commands::hybrid_search,
            commands::get_document_text,
            // RAG
            commands::ask,
            // Timeline + Insights
            commands::get_timeline_data,
            commands::get_detailed_timeline,
            commands::get_memory_facts,
            commands::delete_memory_fact,
            // Analysis
            commands::run_analysis,
            // Entities
            commands::list_entities,
            commands::get_entity_graph,
            commands::get_full_graph,
            // Provider config
            commands::get_llm_config,
            commands::save_llm_config,
            commands::list_provider_presets,
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
            commands::get_usage_log,
            // Boundaries
            commands::list_boundaries,
            commands::add_boundary,
            commands::delete_boundary,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
