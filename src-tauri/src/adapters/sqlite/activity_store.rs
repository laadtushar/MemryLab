use std::sync::Arc;

use rusqlite::params;
use serde::{Deserialize, Serialize};

use super::connection::SqliteConnection;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEntry {
    pub id: String,
    pub timestamp: String,
    pub action_type: String,
    pub title: String,
    pub description: String,
    pub result_summary: String,
    pub metadata: serde_json::Value,
    pub duration_ms: i64,
    pub status: String,
}

pub struct SqliteActivityStore {
    db: Arc<SqliteConnection>,
}

impl SqliteActivityStore {
    pub fn new(db: Arc<SqliteConnection>) -> Self {
        Self { db }
    }

    pub fn log_activity(&self, entry: &ActivityEntry) -> Result<(), AppError> {
        let metadata_str = serde_json::to_string(&entry.metadata)?;
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO activity_log (id, timestamp, action_type, title, description, result_summary, metadata, duration_ms, status)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    entry.id,
                    entry.timestamp,
                    entry.action_type,
                    entry.title,
                    entry.description,
                    entry.result_summary,
                    metadata_str,
                    entry.duration_ms,
                    entry.status,
                ],
            )?;
            Ok(())
        })
    }

    pub fn get_recent(
        &self,
        limit: usize,
        action_type_filter: Option<&str>,
    ) -> Result<Vec<ActivityEntry>, AppError> {
        self.db.with_conn(|conn| {
            let (sql, needs_filter) = if action_type_filter.is_some() {
                (
                    "SELECT id, timestamp, action_type, title, description, result_summary, metadata, duration_ms, status
                     FROM activity_log
                     WHERE action_type = ?1
                     ORDER BY timestamp DESC
                     LIMIT ?2",
                    true,
                )
            } else {
                (
                    "SELECT id, timestamp, action_type, title, description, result_summary, metadata, duration_ms, status
                     FROM activity_log
                     ORDER BY timestamp DESC
                     LIMIT ?1",
                    false,
                )
            };

            let mut stmt = conn.prepare(sql)?;
            let rows = if needs_filter {
                stmt.query_map(
                    params![action_type_filter.unwrap_or(""), limit as i64],
                    map_row,
                )?
            } else {
                stmt.query_map(params![limit as i64], map_row)?
            };

            let mut entries = Vec::new();
            for row in rows {
                entries.push(row.map_err(AppError::Database)?);
            }
            Ok(entries)
        })
    }

    pub fn get_all_count(&self) -> Result<usize, AppError> {
        self.db.with_conn(|conn| {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM activity_log",
                [],
                |row| row.get(0),
            )?;
            Ok(count as usize)
        })
    }
}

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<ActivityEntry> {
    let metadata_str: String = row.get(6)?;
    let metadata: serde_json::Value =
        serde_json::from_str(&metadata_str).unwrap_or(serde_json::json!({}));
    Ok(ActivityEntry {
        id: row.get(0)?,
        timestamp: row.get(1)?,
        action_type: row.get(2)?,
        title: row.get(3)?,
        description: row.get(4)?,
        result_summary: row.get(5)?,
        metadata,
        duration_ms: row.get(7)?,
        status: row.get(8)?,
    })
}
