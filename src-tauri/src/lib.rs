pub mod adapters;
pub mod app_state;
pub mod commands;
pub mod domain;
pub mod error;
pub mod pipeline;
pub mod prompts;
pub mod query;

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_appender::rolling;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Set up file logging to the app data directory
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("com.memorypalace.app");
    std::fs::create_dir_all(&data_dir).ok();

    let file_appender = rolling::daily(&data_dir, "memory_palace.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,memory_palace_lib=debug"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true),
        )
        .with(
            fmt::layer()
                .with_writer(std::io::stdout)
                .with_target(false)
                .compact(),
        )
        .init();

    tracing::info!(log_dir = %data_dir.display(), "Memory Palace logging initialized");

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            use tauri::Manager;
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");

            tracing::info!(data_dir = %data_dir.display(), "Setting up application state");

            let state = app_state::AppState::new(data_dir)
                .expect("failed to initialize application state");

            app.manage(state);
            tracing::info!("Application state initialized successfully");
            Ok(())
        })
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
            commands::get_evolution_diff,
            // Embeddings
            commands::generate_embeddings,
            // Export
            commands::export_memory_json,
            commands::export_memory_markdown,
            // Settings
            commands::test_ollama_connection,
            commands::get_app_stats,
            commands::get_usage_log,
            // Logs
            commands::get_app_logs,
            commands::get_log_path,
            // Boundaries
            commands::list_boundaries,
            commands::add_boundary,
            commands::delete_boundary,
            // PII
            commands::scan_pii,
            commands::get_pii_flags,
            // Prompts
            commands::list_prompts,
            commands::update_prompt,
            commands::set_active_prompt,
            // Activity log
            commands::get_activity_log,
            // Chat history
            commands::list_conversations,
            commands::get_conversation_messages,
            commands::create_conversation,
            commands::delete_conversation,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
