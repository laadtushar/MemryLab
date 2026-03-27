use std::sync::Arc;

use rusqlite::params;

use crate::domain::ports::page_index::{FtsResult, IPageIndex};
use crate::error::AppError;

use super::connection::SqliteConnection;

pub struct SqliteFts5Index {
    db: Arc<SqliteConnection>,
}

impl SqliteFts5Index {
    pub fn new(db: Arc<SqliteConnection>) -> Self {
        Self { db }
    }
}

impl IPageIndex for SqliteFts5Index {
    fn index_text(&self, chunk_id: &str, text: &str) -> Result<(), AppError> {
        // FTS5 is automatically synced via triggers on the chunks table.
        // This method is a no-op if the chunk was inserted via IDocumentStore.
        // It's here for manual indexing if needed.
        let _ = (chunk_id, text);
        Ok(())
    }

    fn index_batch(&self, items: &[(String, String)]) -> Result<(), AppError> {
        // Same as above — FTS5 syncs via triggers
        let _ = items;
        Ok(())
    }

    fn search(&self, query: &str, top_k: usize) -> Result<Vec<FtsResult>, AppError> {
        self.db.with_conn(|conn| {
            // Sanitize query for FTS5: wrap each word in double quotes to escape operators
            let sanitized = sanitize_fts5_query(query);
            if sanitized.is_empty() {
                return Ok(Vec::new());
            }

            // Use BM25 ranking. Lower bm25 score = more relevant.
            let mut stmt = conn.prepare(
                "SELECT c.id, snippet(chunks_fts, 0, '<b>', '</b>', '...', 32), bm25(chunks_fts)
                 FROM chunks_fts
                 JOIN chunks c ON c.rowid = chunks_fts.rowid
                 WHERE chunks_fts MATCH ?1
                 ORDER BY bm25(chunks_fts)
                 LIMIT ?2",
            )?;
            let rows = stmt.query_map(params![sanitized, top_k as i64], |row| {
                Ok(FtsResult {
                    chunk_id: row.get(0)?,
                    snippet: row.get(1)?,
                    rank_score: row.get::<_, f64>(2)?.abs(), // bm25 returns negative, abs for display
                })
            })?;
            let mut results = Vec::new();
            for row in rows {
                results.push(row?);
            }
            Ok(results)
        })
    }

    fn remove(&self, chunk_id: &str) -> Result<(), AppError> {
        // FTS5 entries are removed via trigger when chunk is deleted
        let _ = chunk_id;
        Ok(())
    }

    fn search_highlighted(&self, query: &str, top_k: usize) -> Result<Vec<FtsResult>, AppError> {
        self.db.with_conn(|conn| {
            let sanitized = sanitize_fts5_query(query);
            if sanitized.is_empty() {
                return Ok(Vec::new());
            }
            let mut stmt = conn.prepare(
                "SELECT c.id, highlight(chunks_fts, 0, '<mark>', '</mark>'), bm25(chunks_fts)
                 FROM chunks_fts
                 JOIN chunks c ON c.rowid = chunks_fts.rowid
                 WHERE chunks_fts MATCH ?1
                 ORDER BY bm25(chunks_fts)
                 LIMIT ?2",
            )?;
            let rows = stmt.query_map(params![sanitized, top_k as i64], |row| {
                Ok(FtsResult {
                    chunk_id: row.get(0)?,
                    snippet: row.get(1)?,
                    rank_score: row.get::<_, f64>(2)?.abs(),
                })
            })?;
            let mut results = Vec::new();
            for row in rows {
                results.push(row?);
            }
            Ok(results)
        })
    }

    fn suggest(&self, prefix: &str, top_k: usize) -> Result<Vec<String>, AppError> {
        self.db.with_conn(|conn| {
            let clean: String = prefix.chars().filter(|c| c.is_alphanumeric() || *c == ' ').collect();
            if clean.is_empty() {
                return Ok(Vec::new());
            }
            // FTS5 prefix query: "term"*
            let fts_query = format!("\"{}\"*", clean);
            let mut stmt = conn.prepare(
                "SELECT DISTINCT snippet(chunks_fts, 0, '', '', '', 8)
                 FROM chunks_fts
                 WHERE chunks_fts MATCH ?1
                 ORDER BY bm25(chunks_fts)
                 LIMIT ?2",
            )?;
            let rows = stmt.query_map(params![fts_query, top_k as i64], |row| {
                row.get::<_, String>(0)
            })?;
            let mut results = Vec::new();
            for row in rows {
                results.push(row?);
            }
            Ok(results)
        })
    }

    fn find_related(&self, chunk_id: &str, top_k: usize) -> Result<Vec<FtsResult>, AppError> {
        self.db.with_conn(|conn| {
            // Get the text of the source chunk
            let text: String = conn.query_row(
                "SELECT text FROM chunks WHERE id = ?1",
                params![chunk_id],
                |row| row.get(0),
            ).map_err(|_| AppError::Other(format!("Chunk {} not found", chunk_id)))?;

            // Extract key terms: longest words (likely most meaningful)
            let mut words: Vec<&str> = text.split_whitespace()
                .filter(|w| w.len() > 3)
                .collect();
            words.sort_by(|a, b| b.len().cmp(&a.len()));
            words.truncate(6);

            if words.is_empty() {
                return Ok(Vec::new());
            }

            let fts_query = words.iter()
                .map(|w| {
                    let clean: String = w.chars().filter(|c| c.is_alphanumeric()).collect();
                    format!("\"{}\"", clean)
                })
                .filter(|s| s.len() > 2)
                .collect::<Vec<_>>()
                .join(" OR ");

            if fts_query.is_empty() {
                return Ok(Vec::new());
            }

            let mut stmt = conn.prepare(
                "SELECT c.id, snippet(chunks_fts, 0, '<mark>', '</mark>', '...', 20), bm25(chunks_fts)
                 FROM chunks_fts
                 JOIN chunks c ON c.rowid = chunks_fts.rowid
                 WHERE chunks_fts MATCH ?1 AND c.id != ?2
                 ORDER BY bm25(chunks_fts)
                 LIMIT ?3",
            )?;
            let rows = stmt.query_map(params![fts_query, chunk_id, top_k as i64], |row| {
                Ok(FtsResult {
                    chunk_id: row.get(0)?,
                    snippet: row.get(1)?,
                    rank_score: row.get::<_, f64>(2)?.abs(),
                })
            })?;
            let mut results = Vec::new();
            for row in rows {
                results.push(row?);
            }
            Ok(results)
        })
    }
}

/// Sanitize a user query for FTS5 MATCH syntax.
/// FTS5 treats ?, *, -, (, ), :, ^, etc. as operators.
/// We quote each word to make them literal search terms.
fn sanitize_fts5_query(query: &str) -> String {
    query
        .split_whitespace()
        .map(|word| {
            // Strip FTS5 special characters, keep alphanumeric and common chars
            let clean: String = word
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '\'' || *c == '-')
                .collect();
            if clean.is_empty() {
                String::new()
            } else {
                // Quote each term to prevent FTS5 operator interpretation
                format!("\"{}\"", clean)
            }
        })
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::sqlite::document_store::SqliteDocumentStore;
    use crate::domain::models::common::SourcePlatform;
    use crate::domain::models::document::{Chunk, Document};
    use crate::domain::ports::document_store::IDocumentStore;
    use chrono::Utc;

    #[test]
    fn test_fts5_search() {
        let db = Arc::new(SqliteConnection::open_in_memory().unwrap());
        let doc_store = SqliteDocumentStore::new(db.clone());
        let fts = SqliteFts5Index::new(db);

        // Insert a document and chunks
        let doc = Document {
            id: "d1".to_string(),
            source_platform: SourcePlatform::Obsidian,
            raw_text: "test".to_string(),
            timestamp: Some(Utc::now()),
            participants: vec![],
            metadata: serde_json::json!({}),
            content_hash: "h1".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        doc_store.save_document(&doc).unwrap();

        let chunks = vec![
            Chunk {
                id: "c1".to_string(),
                document_id: "d1".to_string(),
                text: "I went hiking in the beautiful mountains yesterday".to_string(),
                token_count: 9,
                position: 0,
                created_at: Utc::now(),
            },
            Chunk {
                id: "c2".to_string(),
                document_id: "d1".to_string(),
                text: "The weather was perfect for a beach day".to_string(),
                token_count: 8,
                position: 1,
                created_at: Utc::now(),
            },
        ];
        doc_store.save_chunks(&chunks).unwrap();

        // Search for hiking
        let results = fts.search("hiking", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].chunk_id, "c1");

        // Search for beach
        let results = fts.search("beach", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].chunk_id, "c2");
    }
}
