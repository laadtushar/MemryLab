use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio_util::sync::CancellationToken;

/// Status of a background task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum TaskStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Persisted record of a background task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskRecord {
    pub id: String,
    pub task_type: String,
    pub label: String,
    pub status: TaskStatus,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub error: Option<String>,
}

/// Manages background tasks: cancellation tokens, concurrency limits, and state persistence.
pub struct TaskManager {
    /// Active cancellation tokens keyed by task ID
    tokens: Arc<Mutex<HashMap<String, CancellationToken>>>,
    /// Import concurrency semaphore (max 3 concurrent imports)
    import_semaphore: Arc<tokio::sync::Semaphore>,
    /// Database connection for task state persistence
    db: Arc<crate::adapters::sqlite::connection::SqliteConnection>,
}

impl TaskManager {
    pub fn new(db: Arc<crate::adapters::sqlite::connection::SqliteConnection>) -> Self {
        // Create the task state table if it doesn't exist
        let _ = db.with_conn(|conn| {
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS background_tasks (
                    id TEXT PRIMARY KEY,
                    task_type TEXT NOT NULL,
                    label TEXT NOT NULL,
                    status TEXT NOT NULL DEFAULT 'Running',
                    started_at TEXT NOT NULL,
                    finished_at TEXT,
                    error TEXT
                );"
            ).map_err(crate::error::AppError::Database)?;
            Ok(())
        });

        // Mark any stale "Running" tasks as interrupted on startup
        let _ = db.with_conn(|conn| {
            conn.execute(
                "UPDATE background_tasks SET status = 'Failed', error = 'App restarted — task interrupted', finished_at = datetime('now') WHERE status = 'Running'",
                [],
            ).map_err(crate::error::AppError::Database)?;
            Ok(())
        });

        Self {
            tokens: Arc::new(Mutex::new(HashMap::new())),
            import_semaphore: Arc::new(tokio::sync::Semaphore::new(3)),
            db,
        }
    }

    /// Register a new task and get its cancellation token.
    pub fn register_task(&self, id: &str, task_type: &str, label: &str) -> CancellationToken {
        let token = CancellationToken::new();
        self.tokens.lock().unwrap().insert(id.to_string(), token.clone());

        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let _ = self.db.with_conn(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO background_tasks (id, task_type, label, status, started_at) VALUES (?1, ?2, ?3, 'Running', ?4)",
                rusqlite::params![id, task_type, label, now],
            ).map_err(crate::error::AppError::Database)?;
            Ok(())
        });

        token
    }

    /// Mark a task as completed
    pub fn complete_task(&self, id: &str, error: Option<&str>) {
        self.tokens.lock().unwrap().remove(id);

        let status = if error.is_some() { "Failed" } else { "Completed" };
        let _ = self.db.with_conn(|conn| {
            conn.execute(
                "UPDATE background_tasks SET status = ?1, finished_at = datetime('now'), error = ?2 WHERE id = ?3",
                rusqlite::params![status, error, id],
            ).map_err(crate::error::AppError::Database)?;
            Ok(())
        });
    }

    /// Mark a task as cancelled
    pub fn mark_cancelled(&self, id: &str) {
        self.tokens.lock().unwrap().remove(id);
        let _ = self.db.with_conn(|conn| {
            conn.execute(
                "UPDATE background_tasks SET status = 'Cancelled', finished_at = datetime('now') WHERE id = ?1",
                rusqlite::params![id],
            ).map_err(crate::error::AppError::Database)?;
            Ok(())
        });
    }

    /// Cancel a running task by ID
    pub fn cancel_task(&self, id: &str) -> bool {
        let tokens = self.tokens.lock().unwrap();
        if let Some(token) = tokens.get(id) {
            token.cancel();
            true
        } else {
            false
        }
    }

    /// Check if a task has been cancelled
    pub fn is_cancelled(&self, id: &str) -> bool {
        let tokens = self.tokens.lock().unwrap();
        tokens.get(id).map_or(true, |t| t.is_cancelled())
    }

    /// Get the cancellation token for a task (for passing to async work)
    pub fn get_token(&self, id: &str) -> Option<CancellationToken> {
        self.tokens.lock().unwrap().get(id).cloned()
    }

    /// Acquire an import permit (blocks if 3 imports already running)
    pub async fn acquire_import_permit(&self) -> tokio::sync::OwnedSemaphorePermit {
        self.import_semaphore.clone().acquire_owned().await.unwrap()
    }

    /// Get tasks that were interrupted (for frontend to show on restart)
    pub fn get_interrupted_tasks(&self) -> Vec<TaskRecord> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, task_type, label, status, started_at, finished_at, error FROM background_tasks WHERE status = 'Failed' AND error = 'App restarted — task interrupted' ORDER BY started_at DESC LIMIT 10"
            ).map_err(crate::error::AppError::Database)?;
            let rows = stmt.query_map([], |row| {
                Ok(TaskRecord {
                    id: row.get(0)?,
                    task_type: row.get(1)?,
                    label: row.get(2)?,
                    status: TaskStatus::Failed,
                    started_at: row.get(4)?,
                    finished_at: row.get(5)?,
                    error: row.get(6)?,
                })
            }).map_err(crate::error::AppError::Database)?;
            let mut results = Vec::new();
            for row in rows {
                if let Ok(r) = row {
                    results.push(r);
                }
            }
            Ok(results)
        }).unwrap_or_default()
    }

    /// Clean up old completed/failed tasks older than 7 days
    pub fn cleanup_old_tasks(&self) {
        let _ = self.db.with_conn(|conn| {
            conn.execute(
                "DELETE FROM background_tasks WHERE status != 'Running' AND finished_at < datetime('now', '-7 days')",
                [],
            ).map_err(crate::error::AppError::Database)?;
            Ok(())
        });
    }
}
