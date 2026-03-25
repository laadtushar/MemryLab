use std::sync::Arc;

use rusqlite::params;

use crate::domain::models::common::TimeRange;
use crate::domain::models::document::Document;
use crate::domain::ports::timeline_store::ITimelineStore;
use crate::error::AppError;

use super::connection::SqliteConnection;

pub struct SqliteTimelineStore {
    db: Arc<SqliteConnection>,
}

impl SqliteTimelineStore {
    pub fn new(db: Arc<SqliteConnection>) -> Self {
        Self { db }
    }
}

impl ITimelineStore for SqliteTimelineStore {
    fn index_document(&self, _doc: &Document) -> Result<(), AppError> {
        // Documents are already indexed by timestamp in the documents table.
        // This is a no-op since temporal queries use the documents table directly.
        Ok(())
    }

    fn get_documents_in_range(&self, range: &TimeRange) -> Result<Vec<String>, AppError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id FROM documents
                 WHERE timestamp >= ?1 AND timestamp <= ?2
                 ORDER BY timestamp ASC",
            )?;
            let rows = stmt.query_map(
                params![range.start.to_rfc3339(), range.end.to_rfc3339()],
                |row| row.get::<_, String>(0),
            )?;
            let mut ids = Vec::new();
            for row in rows {
                ids.push(row?);
            }
            Ok(ids)
        })
    }

    fn get_document_count_by_month(&self) -> Result<Vec<(String, usize)>, AppError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT strftime('%Y-%m', timestamp) as month, COUNT(*) as cnt
                 FROM documents
                 GROUP BY month
                 ORDER BY month ASC",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
            })?;
            let mut counts = Vec::new();
            for row in rows {
                counts.push(row?);
            }
            Ok(counts)
        })
    }

    fn get_document_count_by_day(&self) -> Result<Vec<(String, usize)>, AppError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT strftime('%Y-%m-%d', timestamp) as day, COUNT(*) as cnt
                 FROM documents
                 GROUP BY day
                 ORDER BY day ASC",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
            })?;
            let mut counts = Vec::new();
            for row in rows {
                counts.push(row?);
            }
            Ok(counts)
        })
    }

    fn get_document_count_by_week(&self) -> Result<Vec<(String, usize)>, AppError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT strftime('%Y-W%W', timestamp) as week, COUNT(*) as cnt
                 FROM documents
                 GROUP BY week
                 ORDER BY week ASC",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
            })?;
            let mut counts = Vec::new();
            for row in rows {
                counts.push(row?);
            }
            Ok(counts)
        })
    }

    fn get_document_count_by_year(&self) -> Result<Vec<(String, usize)>, AppError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT strftime('%Y', timestamp) as year, COUNT(*) as cnt
                 FROM documents
                 GROUP BY year
                 ORDER BY year ASC",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
            })?;
            let mut counts = Vec::new();
            for row in rows {
                counts.push(row?);
            }
            Ok(counts)
        })
    }

    fn get_date_range(&self) -> Result<Option<TimeRange>, AppError> {
        self.db.with_conn(|conn| {
            let result: Option<(String, String)> = conn
                .query_row(
                    "SELECT MIN(timestamp), MAX(timestamp) FROM documents",
                    [],
                    |row| {
                        let min: Option<String> = row.get(0)?;
                        let max: Option<String> = row.get(1)?;
                        Ok(min.zip(max))
                    },
                )
                .ok()
                .flatten();

            match result {
                Some((min_str, max_str)) => {
                    let start = chrono::DateTime::parse_from_rfc3339(&min_str)
                        .map_err(|e| AppError::Other(e.to_string()))?
                        .with_timezone(&chrono::Utc);
                    let end = chrono::DateTime::parse_from_rfc3339(&max_str)
                        .map_err(|e| AppError::Other(e.to_string()))?
                        .with_timezone(&chrono::Utc);
                    Ok(Some(TimeRange { start, end }))
                }
                None => Ok(None),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::sqlite::document_store::SqliteDocumentStore;
    use crate::domain::models::common::SourcePlatform;
    use crate::domain::ports::document_store::IDocumentStore;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_document_count_by_month() {
        let db = Arc::new(SqliteConnection::open_in_memory().unwrap());
        let doc_store = SqliteDocumentStore::new(db.clone());
        let timeline = SqliteTimelineStore::new(db);

        // Insert documents in different months
        for (i, month) in [1, 1, 2, 3, 3, 3].iter().enumerate() {
            let ts = Utc.with_ymd_and_hms(2024, *month, 15, 12, 0, 0).unwrap();
            let doc = Document {
                id: format!("d{}", i),
                source_platform: SourcePlatform::Obsidian,
                raw_text: format!("doc {}", i),
                timestamp: ts,
                participants: vec![],
                metadata: serde_json::json!({}),
                content_hash: format!("h{}", i),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            doc_store.save_document(&doc).unwrap();
        }

        let counts = timeline.get_document_count_by_month().unwrap();
        assert_eq!(counts.len(), 3);
        assert_eq!(counts[0], ("2024-01".to_string(), 2));
        assert_eq!(counts[1], ("2024-02".to_string(), 1));
        assert_eq!(counts[2], ("2024-03".to_string(), 3));
    }

    #[test]
    fn test_get_date_range() {
        let db = Arc::new(SqliteConnection::open_in_memory().unwrap());
        let doc_store = SqliteDocumentStore::new(db.clone());
        let timeline = SqliteTimelineStore::new(db);

        // No documents yet
        assert!(timeline.get_date_range().unwrap().is_none());

        let ts1 = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();
        let ts2 = Utc.with_ymd_and_hms(2024, 6, 15, 0, 0, 0).unwrap();
        for (i, ts) in [ts1, ts2].iter().enumerate() {
            let doc = Document {
                id: format!("d{}", i),
                source_platform: SourcePlatform::Markdown,
                raw_text: "test".to_string(),
                timestamp: *ts,
                participants: vec![],
                metadata: serde_json::json!({}),
                content_hash: format!("h{}", i),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            doc_store.save_document(&doc).unwrap();
        }

        let range = timeline.get_date_range().unwrap().unwrap();
        assert_eq!(range.start, ts1);
        assert_eq!(range.end, ts2);
    }
}
