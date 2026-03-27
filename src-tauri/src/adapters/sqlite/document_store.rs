use std::sync::Arc;

use rusqlite::params;

use crate::domain::models::common::{SourcePlatform, TimeRange};
use crate::domain::models::document::{Chunk, Document};
use crate::domain::ports::document_store::IDocumentStore;
use crate::error::AppError;

use super::connection::SqliteConnection;

pub struct SqliteDocumentStore {
    db: Arc<SqliteConnection>,
}

impl SqliteDocumentStore {
    pub fn new(db: Arc<SqliteConnection>) -> Self {
        Self { db }
    }
}

impl IDocumentStore for SqliteDocumentStore {
    fn save_document(&self, doc: &Document) -> Result<(), AppError> {
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO documents (id, source_platform, raw_text, timestamp, participants, metadata, content_hash, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    doc.id,
                    doc.source_platform.to_string(),
                    doc.raw_text,
                    doc.timestamp.map(|t| t.to_rfc3339()),
                    serde_json::to_string(&doc.participants)?,
                    doc.metadata.to_string(),
                    doc.content_hash,
                    doc.created_at.to_rfc3339(),
                    doc.updated_at.to_rfc3339(),
                ],
            )?;
            Ok(())
        })
    }

    fn get_by_id(&self, id: &str) -> Result<Option<Document>, AppError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, source_platform, raw_text, timestamp, participants, metadata, content_hash, created_at, updated_at
                 FROM documents WHERE id = ?1",
            )?;
            let result = stmt
                .query_row(params![id], |row| {
                    Ok(row_to_document(row))
                })
                .optional()?;
            match result {
                Some(doc) => Ok(Some(doc?)),
                None => Ok(None),
            }
        })
    }

    fn get_by_source(
        &self,
        platform: &SourcePlatform,
        time_range: Option<&TimeRange>,
    ) -> Result<Vec<Document>, AppError> {
        self.db.with_conn(|conn| {
            let (sql, dynamic_params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) =
                match time_range {
                    Some(range) => (
                        "SELECT id, source_platform, raw_text, timestamp, participants, metadata, content_hash, created_at, updated_at
                         FROM documents WHERE source_platform = ?1 AND timestamp >= ?2 AND timestamp <= ?3
                         ORDER BY timestamp ASC".to_string(),
                        vec![
                            Box::new(platform.to_string()) as Box<dyn rusqlite::types::ToSql>,
                            Box::new(range.start.to_rfc3339()),
                            Box::new(range.end.to_rfc3339()),
                        ],
                    ),
                    None => (
                        "SELECT id, source_platform, raw_text, timestamp, participants, metadata, content_hash, created_at, updated_at
                         FROM documents WHERE source_platform = ?1
                         ORDER BY timestamp ASC".to_string(),
                        vec![Box::new(platform.to_string()) as Box<dyn rusqlite::types::ToSql>],
                    ),
                };
            let mut stmt = conn.prepare(&sql)?;
            let param_refs: Vec<&dyn rusqlite::types::ToSql> =
                dynamic_params.iter().map(|p| p.as_ref()).collect();
            let rows = stmt.query_map(param_refs.as_slice(), |row| Ok(row_to_document(row)))?;
            let mut docs = Vec::new();
            for row in rows {
                docs.push(row??);
            }
            Ok(docs)
        })
    }

    fn get_by_content_hash(&self, hash: &str) -> Result<Option<Document>, AppError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, source_platform, raw_text, timestamp, participants, metadata, content_hash, created_at, updated_at
                 FROM documents WHERE content_hash = ?1",
            )?;
            let result = stmt
                .query_row(params![hash], |row| Ok(row_to_document(row)))
                .optional()?;
            match result {
                Some(doc) => Ok(Some(doc?)),
                None => Ok(None),
            }
        })
    }

    fn delete_document(&self, id: &str) -> Result<bool, AppError> {
        self.db.with_conn(|conn| {
            let rows = conn.execute("DELETE FROM documents WHERE id = ?1", params![id])?;
            Ok(rows > 0)
        })
    }

    fn save_chunk(&self, chunk: &Chunk) -> Result<(), AppError> {
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO chunks (id, document_id, text, token_count, position, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    chunk.id,
                    chunk.document_id,
                    chunk.text,
                    chunk.token_count as i64,
                    chunk.position as i64,
                    chunk.created_at.to_rfc3339(),
                ],
            )?;
            Ok(())
        })
    }

    fn save_chunks(&self, chunks: &[Chunk]) -> Result<(), AppError> {
        self.db.with_conn(|conn| {
            let tx = conn.unchecked_transaction()?;
            {
                let mut stmt = tx.prepare(
                    "INSERT OR REPLACE INTO chunks (id, document_id, text, token_count, position, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                )?;
                for chunk in chunks {
                    stmt.execute(params![
                        chunk.id,
                        chunk.document_id,
                        chunk.text,
                        chunk.token_count as i64,
                        chunk.position as i64,
                        chunk.created_at.to_rfc3339(),
                    ])?;
                }
            }
            tx.commit()?;
            Ok(())
        })
    }

    fn get_chunks_by_document(&self, document_id: &str) -> Result<Vec<Chunk>, AppError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, document_id, text, token_count, position, created_at
                 FROM chunks WHERE document_id = ?1 ORDER BY position ASC",
            )?;
            let rows = stmt.query_map(params![document_id], |row| Ok(row_to_chunk(row)))?;
            let mut chunks = Vec::new();
            for row in rows {
                chunks.push(row??);
            }
            Ok(chunks)
        })
    }

    fn get_chunk_by_id(&self, id: &str) -> Result<Option<Chunk>, AppError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, document_id, text, token_count, position, created_at
                 FROM chunks WHERE id = ?1",
            )?;
            let result = stmt
                .query_row(params![id], |row| Ok(row_to_chunk(row)))
                .optional()?;
            match result {
                Some(chunk) => Ok(Some(chunk?)),
                None => Ok(None),
            }
        })
    }

    fn get_chunks_by_ids(&self, ids: &[String]) -> Result<Vec<Chunk>, AppError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        self.db.with_conn(|conn| {
            let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("?{}", i)).collect();
            let sql = format!(
                "SELECT id, document_id, text, token_count, position, created_at
                 FROM chunks WHERE id IN ({})",
                placeholders.join(", ")
            );
            let mut stmt = conn.prepare(&sql)?;
            let param_refs: Vec<&dyn rusqlite::types::ToSql> =
                ids.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();
            let rows = stmt.query_map(param_refs.as_slice(), |row| Ok(row_to_chunk(row)))?;
            let mut chunks = Vec::new();
            for row in rows {
                chunks.push(row??);
            }
            Ok(chunks)
        })
    }
}

use rusqlite::OptionalExtension;

fn parse_source_platform(s: &str) -> SourcePlatform {
    match s {
        "obsidian" => SourcePlatform::Obsidian,
        "markdown" => SourcePlatform::Markdown,
        "dayone" => SourcePlatform::DayOne,
        "whatsapp" => SourcePlatform::WhatsApp,
        "telegram" => SourcePlatform::Telegram,
        "twitter" => SourcePlatform::Twitter,
        "instagram" => SourcePlatform::Instagram,
        "facebook" => SourcePlatform::Facebook,
        "reddit" => SourcePlatform::Reddit,
        "linkedin" => SourcePlatform::LinkedIn,
        "google_takeout" => SourcePlatform::GoogleTakeout,
        "apple_notes" => SourcePlatform::AppleNotes,
        "notion" => SourcePlatform::Notion,
        "plain_text" => SourcePlatform::PlainText,
        _ => SourcePlatform::Custom,
    }
}

fn row_to_document(row: &rusqlite::Row) -> Result<Document, AppError> {
    let timestamp_str: Option<String> = row.get(3)?;
    let created_str: String = row.get(7)?;
    let updated_str: String = row.get(8)?;
    let participants_str: String = row.get(4)?;
    let metadata_str: String = row.get(5)?;
    let platform_str: String = row.get(1)?;

    Ok(Document {
        id: row.get(0)?,
        source_platform: parse_source_platform(&platform_str),
        raw_text: row.get(2)?,
        timestamp: timestamp_str
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc)),
        participants: serde_json::from_str(&participants_str)?,
        metadata: serde_json::from_str(&metadata_str)?,
        content_hash: row.get(6)?,
        created_at: chrono::DateTime::parse_from_rfc3339(&created_str)
            .map_err(|e| AppError::Other(e.to_string()))?
            .with_timezone(&chrono::Utc),
        updated_at: chrono::DateTime::parse_from_rfc3339(&updated_str)
            .map_err(|e| AppError::Other(e.to_string()))?
            .with_timezone(&chrono::Utc),
    })
}

fn row_to_chunk(row: &rusqlite::Row) -> Result<Chunk, AppError> {
    let created_str: String = row.get(5)?;
    let token_count: i64 = row.get(3)?;
    let position: i64 = row.get(4)?;

    Ok(Chunk {
        id: row.get(0)?,
        document_id: row.get(1)?,
        text: row.get(2)?,
        token_count: token_count as usize,
        position: position as usize,
        created_at: chrono::DateTime::parse_from_rfc3339(&created_str)
            .map_err(|e| AppError::Other(e.to_string()))?
            .with_timezone(&chrono::Utc),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_test_doc(id: &str, hash: &str) -> Document {
        Document {
            id: id.to_string(),
            source_platform: SourcePlatform::Obsidian,
            raw_text: "Test document content".to_string(),
            timestamp: Some(Utc::now()),
            participants: vec![],
            metadata: serde_json::json!({}),
            content_hash: hash.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_save_and_get_document() {
        let db = Arc::new(SqliteConnection::open_in_memory().unwrap());
        let store = SqliteDocumentStore::new(db);

        let doc = make_test_doc("doc1", "hash1");
        store.save_document(&doc).unwrap();

        let retrieved = store.get_by_id("doc1").unwrap().unwrap();
        assert_eq!(retrieved.id, "doc1");
        assert_eq!(retrieved.raw_text, "Test document content");
    }

    #[test]
    fn test_get_by_content_hash() {
        let db = Arc::new(SqliteConnection::open_in_memory().unwrap());
        let store = SqliteDocumentStore::new(db);

        let doc = make_test_doc("doc1", "unique_hash");
        store.save_document(&doc).unwrap();

        let found = store.get_by_content_hash("unique_hash").unwrap();
        assert!(found.is_some());

        let not_found = store.get_by_content_hash("nonexistent").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_delete_document() {
        let db = Arc::new(SqliteConnection::open_in_memory().unwrap());
        let store = SqliteDocumentStore::new(db);

        let doc = make_test_doc("doc1", "hash1");
        store.save_document(&doc).unwrap();

        assert!(store.delete_document("doc1").unwrap());
        assert!(store.get_by_id("doc1").unwrap().is_none());
    }

    #[test]
    fn test_save_and_get_chunks() {
        let db = Arc::new(SqliteConnection::open_in_memory().unwrap());
        let store = SqliteDocumentStore::new(db);

        let doc = make_test_doc("doc1", "hash1");
        store.save_document(&doc).unwrap();

        let chunks = vec![
            Chunk {
                id: "c1".to_string(),
                document_id: "doc1".to_string(),
                text: "First chunk".to_string(),
                token_count: 10,
                position: 0,
                created_at: Utc::now(),
            },
            Chunk {
                id: "c2".to_string(),
                document_id: "doc1".to_string(),
                text: "Second chunk".to_string(),
                token_count: 12,
                position: 1,
                created_at: Utc::now(),
            },
        ];
        store.save_chunks(&chunks).unwrap();

        let retrieved = store.get_chunks_by_document("doc1").unwrap();
        assert_eq!(retrieved.len(), 2);
        assert_eq!(retrieved[0].text, "First chunk");
        assert_eq!(retrieved[1].text, "Second chunk");
    }
}
