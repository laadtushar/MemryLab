use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use walkdir;
use tauri::{AppHandle, Emitter, Manager};

use crate::app_state::AppState;
use crate::pipeline::ingestion::orchestrator::IngestionOrchestrator;
use crate::pipeline::ingestion::source_adapters::registry;

/// Watched folder configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WatchedFolder {
    pub path: String,
    pub adapter_id: Option<String>,
    pub enabled: bool,
}

/// Progress event for watched folder imports
#[derive(Clone, serde::Serialize)]
pub struct WatchProgress {
    pub path: String,
    pub files_found: usize,
    pub message: String,
}

/// Manages file system watchers for configured folders.
pub struct FolderWatcherService {
    watchers: Arc<Mutex<HashMap<String, RecommendedWatcher>>>,
    app_handle: AppHandle,
    /// Debounce: track last event time per file to avoid duplicate processing
    last_events: Arc<Mutex<HashMap<PathBuf, Instant>>>,
}

impl FolderWatcherService {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            watchers: Arc::new(Mutex::new(HashMap::new())),
            app_handle,
            last_events: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start watching all saved folders from config
    pub fn start_saved_watches(&self) {
        let folders = self.get_saved_folders();
        for folder in folders {
            if folder.enabled {
                if let Err(e) = self.watch_folder(&folder.path, folder.adapter_id.as_deref()) {
                    tracing::warn!(path = %folder.path, error = %e, "Failed to start watch");
                }
            }
        }
    }

    /// Add a new watched folder and start watching
    pub fn watch_folder(&self, path: &str, adapter_id: Option<&str>) -> Result<(), String> {
        let folder_path = Path::new(path);
        if !folder_path.exists() || !folder_path.is_dir() {
            return Err(format!("Path does not exist or is not a directory: {}", path));
        }

        let path_str = path.to_string();
        let handle = self.app_handle.clone();
        let last_events = self.last_events.clone();
        let adapter_id_owned = adapter_id.map(|s| s.to_string());

        // Create watcher with debounce
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    match event.kind {
                        EventKind::Create(_) | EventKind::Modify(_) => {
                            for path in &event.paths {
                                // Debounce: skip if we processed this file within 5s
                                let mut last = last_events.lock().unwrap();
                                let now = Instant::now();
                                if let Some(prev) = last.get(path) {
                                    if now.duration_since(*prev) < Duration::from_secs(5) {
                                        continue;
                                    }
                                }
                                last.insert(path.clone(), now);
                                drop(last);

                                // Only process known text file types
                                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                                    let supported = ["md", "txt", "json", "csv", "html", "xml", "enex"];
                                    if supported.contains(&ext.to_lowercase().as_str()) {
                                        tracing::info!(file = %path.display(), "Watch: file changed, importing");
                                        let _ = handle.emit("watch-progress", WatchProgress {
                                            path: path_str.clone(),
                                            files_found: 1,
                                            message: format!("Auto-importing: {}", path.file_name().unwrap_or_default().to_string_lossy()),
                                        });

                                        // Import the single file
                                        let state = handle.state::<AppState>();
                                        let parent = path.parent().unwrap_or(path);
                                        let listing = vec![path.file_name().unwrap_or_default().to_string_lossy().to_string()];
                                        let listing_refs: Vec<&str> = listing.iter().map(|s| s.as_str()).collect();

                                        let adapter = if let Some(ref id) = adapter_id_owned {
                                            registry::all_adapters().into_iter().find(|a| &a.metadata().id == id)
                                        } else {
                                            registry::detect_adapter(&listing_refs)
                                        };

                                        if let Some(adapter) = adapter {
                                            match adapter.parse(parent) {
                                                Ok(docs) if !docs.is_empty() => {
                                                    let orch = IngestionOrchestrator::new(
                                                        state.document_store.as_ref(),
                                                        state.timeline_store.as_ref(),
                                                        state.page_index.as_ref(),
                                                    );
                                                    let _ = tauri::async_runtime::block_on(
                                                        orch.ingest_documents(docs, None)
                                                    );
                                                    tracing::info!(file = %path.display(), "Watch: import complete");
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            },
            Config::default().with_poll_interval(Duration::from_secs(2)),
        ).map_err(|e| format!("Watcher creation failed: {}", e))?;

        watcher.watch(Path::new(path), RecursiveMode::Recursive)
            .map_err(|e| format!("Watch failed: {}", e))?;

        tracing::info!(path = %path, "Started watching folder");
        self.watchers.lock().unwrap().insert(path.to_string(), watcher);

        // Save to config
        self.save_folder(path, adapter_id, true);

        // Initial full import of existing files in the folder
        let handle = self.app_handle.clone();
        let path_owned = path.to_string();
        let adapter_id_owned = adapter_id.map(|s| s.to_string());
        std::thread::spawn(move || {
            tracing::info!(path = %path_owned, "Watch: running initial import of existing files");
            let _ = handle.emit("watch-progress", WatchProgress {
                path: path_owned.clone(),
                files_found: 0,
                message: "Scanning existing files...".to_string(),
            });
            let state = handle.state::<AppState>();
            let listing: Vec<String> = walkdir::WalkDir::new(&path_owned)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
                .collect();
            let listing_refs: Vec<&str> = listing.iter().map(|s| s.as_str()).collect();
            let adapter = if let Some(ref id) = adapter_id_owned {
                registry::all_adapters().into_iter().find(|a| &a.metadata().id == id)
            } else {
                registry::detect_adapter(&listing_refs)
            };
            if let Some(adapter) = adapter {
                match adapter.parse(Path::new(&path_owned)) {
                    Ok(docs) if !docs.is_empty() => {
                        let count = docs.len();
                        let orch = IngestionOrchestrator::new(
                            state.document_store.as_ref(),
                            state.timeline_store.as_ref(),
                            state.page_index.as_ref(),
                        );
                        let _ = tauri::async_runtime::block_on(orch.ingest_documents(docs, None));
                        let _ = handle.emit("watch-progress", WatchProgress {
                            path: path_owned.clone(),
                            files_found: count,
                            message: format!("Initial import complete: {} documents", count),
                        });
                        tracing::info!(path = %path_owned, count, "Watch: initial import complete");
                    }
                    _ => {
                        tracing::info!(path = %path_owned, "Watch: no documents found in initial scan");
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop watching a folder
    pub fn unwatch_folder(&self, path: &str) -> Result<(), String> {
        let mut watchers = self.watchers.lock().unwrap();
        if watchers.remove(path).is_some() {
            tracing::info!(path = %path, "Stopped watching folder");
        }
        self.remove_folder(path);
        Ok(())
    }

    /// List all watched folders
    pub fn get_saved_folders(&self) -> Vec<WatchedFolder> {
        let state = self.app_handle.state::<AppState>();
        state.config_store.get("watch.folders").ok().flatten()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn save_folder(&self, path: &str, adapter_id: Option<&str>, enabled: bool) {
        let state = self.app_handle.state::<AppState>();
        let mut folders = self.get_saved_folders();
        folders.retain(|f| f.path != path);
        folders.push(WatchedFolder {
            path: path.to_string(),
            adapter_id: adapter_id.map(|s| s.to_string()),
            enabled,
        });
        let _ = state.config_store.set("watch.folders", &serde_json::to_string(&folders).unwrap_or_default());
    }

    fn remove_folder(&self, path: &str) {
        let state = self.app_handle.state::<AppState>();
        let mut folders = self.get_saved_folders();
        folders.retain(|f| f.path != path);
        let _ = state.config_store.set("watch.folders", &serde_json::to_string(&folders).unwrap_or_default());
    }
}
