use std::sync::Arc;

use rusqlite::{params, OptionalExtension};

use crate::domain::models::common::TimeRange;
use crate::domain::models::memory::{FactCategory, MemoryFact};
use crate::domain::ports::memory_store::IMemoryStore;
use crate::error::AppError;

use super::connection::SqliteConnection;

pub struct SqliteMemoryStore {
    db: Arc<SqliteConnection>,
}

impl SqliteMemoryStore {
    pub fn new(db: Arc<SqliteConnection>) -> Self {
        Self { db }
    }
}

impl IMemoryStore for SqliteMemoryStore {
    fn store(&self, fact: &MemoryFact) -> Result<(), AppError> {
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO memory_facts (id, fact_text, source_chunks, confidence, category, first_seen, last_updated, contradicted_by, is_active)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    fact.id,
                    fact.fact_text,
                    serde_json::to_string(&fact.source_chunks)?,
                    fact.confidence,
                    category_to_str(&fact.category),
                    fact.first_seen.to_rfc3339(),
                    fact.last_updated.to_rfc3339(),
                    serde_json::to_string(&fact.contradicted_by)?,
                    fact.is_active as i32,
                ],
            )?;
            Ok(())
        })
    }

    fn recall(&self, query: &str, top_k: usize) -> Result<Vec<MemoryFact>, AppError> {
        // Simple text-based recall using LIKE matching
        // In the full system, this would also use vector similarity via IVectorStore
        self.db.with_conn(|conn| {
            let pattern = format!("%{}%", query);
            let mut stmt = conn.prepare(
                "SELECT id, fact_text, source_chunks, confidence, category, first_seen, last_updated, contradicted_by, is_active
                 FROM memory_facts
                 WHERE is_active = 1 AND fact_text LIKE ?1
                 ORDER BY confidence DESC
                 LIMIT ?2",
            )?;
            let rows = stmt.query_map(params![pattern, top_k as i64], |row| {
                Ok(row_to_fact(row))
            })?;
            let mut facts = Vec::new();
            for row in rows {
                facts.push(row??);
            }
            Ok(facts)
        })
    }

    fn update(&self, id: &str, updated_fact: &MemoryFact) -> Result<(), AppError> {
        self.db.with_conn(|conn| {
            conn.execute(
                "UPDATE memory_facts SET fact_text = ?1, confidence = ?2, category = ?3, last_updated = ?4, source_chunks = ?5
                 WHERE id = ?6",
                params![
                    updated_fact.fact_text,
                    updated_fact.confidence,
                    category_to_str(&updated_fact.category),
                    updated_fact.last_updated.to_rfc3339(),
                    serde_json::to_string(&updated_fact.source_chunks)?,
                    id,
                ],
            )?;
            Ok(())
        })
    }

    fn contradict(&self, id: &str, contradicting_fact_id: &str) -> Result<(), AppError> {
        self.db.with_conn(|conn| {
            // Get current contradicted_by list
            let current: String = conn.query_row(
                "SELECT contradicted_by FROM memory_facts WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )?;
            let mut list: Vec<String> = serde_json::from_str(&current)?;
            if !list.contains(&contradicting_fact_id.to_string()) {
                list.push(contradicting_fact_id.to_string());
            }
            conn.execute(
                "UPDATE memory_facts SET contradicted_by = ?1 WHERE id = ?2",
                params![serde_json::to_string(&list)?, id],
            )?;
            Ok(())
        })
    }

    fn forget(&self, id: &str) -> Result<(), AppError> {
        self.db.with_conn(|conn| {
            conn.execute(
                "UPDATE memory_facts SET is_active = 0 WHERE id = ?1",
                params![id],
            )?;
            Ok(())
        })
    }

    fn get_all(
        &self,
        category: Option<&FactCategory>,
        time_range: Option<&TimeRange>,
    ) -> Result<Vec<MemoryFact>, AppError> {
        self.db.with_conn(|conn| {
            let mut sql =
                "SELECT id, fact_text, source_chunks, confidence, category, first_seen, last_updated, contradicted_by, is_active
                 FROM memory_facts WHERE is_active = 1".to_string();
            let mut dynamic_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
            let mut param_idx = 1;

            if let Some(cat) = category {
                sql.push_str(&format!(" AND category = ?{}", param_idx));
                dynamic_params.push(Box::new(category_to_str(cat).to_string()));
                param_idx += 1;
            }
            if let Some(range) = time_range {
                sql.push_str(&format!(
                    " AND first_seen >= ?{} AND first_seen <= ?{}",
                    param_idx,
                    param_idx + 1
                ));
                dynamic_params.push(Box::new(range.start.to_rfc3339()));
                dynamic_params.push(Box::new(range.end.to_rfc3339()));
            }
            sql.push_str(" ORDER BY last_updated DESC");

            let mut stmt = conn.prepare(&sql)?;
            let param_refs: Vec<&dyn rusqlite::types::ToSql> =
                dynamic_params.iter().map(|p| p.as_ref()).collect();
            let rows =
                stmt.query_map(param_refs.as_slice(), |row| Ok(row_to_fact(row)))?;
            let mut facts = Vec::new();
            for row in rows {
                facts.push(row??);
            }
            Ok(facts)
        })
    }

    fn get_by_id(&self, id: &str) -> Result<Option<MemoryFact>, AppError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, fact_text, source_chunks, confidence, category, first_seen, last_updated, contradicted_by, is_active
                 FROM memory_facts WHERE id = ?1",
            )?;
            let result = stmt
                .query_row(params![id], |row| Ok(row_to_fact(row)))
                .optional()?;
            match result {
                Some(fact) => Ok(Some(fact?)),
                None => Ok(None),
            }
        })
    }
}

fn category_to_str(cat: &FactCategory) -> &'static str {
    match cat {
        FactCategory::Belief => "belief",
        FactCategory::Preference => "preference",
        FactCategory::Fact => "fact",
        FactCategory::SelfDescription => "self_description",
        FactCategory::Insight => "insight",
    }
}

fn str_to_category(s: &str) -> FactCategory {
    match s {
        "belief" => FactCategory::Belief,
        "preference" => FactCategory::Preference,
        "fact" => FactCategory::Fact,
        "self_description" => FactCategory::SelfDescription,
        "insight" => FactCategory::Insight,
        _ => FactCategory::Fact,
    }
}

fn row_to_fact(row: &rusqlite::Row) -> Result<MemoryFact, AppError> {
    let source_chunks_str: String = row.get(2)?;
    let category_str: String = row.get(4)?;
    let first_seen_str: String = row.get(5)?;
    let last_updated_str: String = row.get(6)?;
    let contradicted_by_str: String = row.get(7)?;
    let is_active: i32 = row.get(8)?;

    Ok(MemoryFact {
        id: row.get(0)?,
        fact_text: row.get(1)?,
        source_chunks: serde_json::from_str(&source_chunks_str)?,
        confidence: row.get(3)?,
        category: str_to_category(&category_str),
        first_seen: chrono::DateTime::parse_from_rfc3339(&first_seen_str)
            .map_err(|e| AppError::Other(e.to_string()))?
            .with_timezone(&chrono::Utc),
        last_updated: chrono::DateTime::parse_from_rfc3339(&last_updated_str)
            .map_err(|e| AppError::Other(e.to_string()))?
            .with_timezone(&chrono::Utc),
        contradicted_by: serde_json::from_str(&contradicted_by_str)?,
        is_active: is_active != 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_fact(id: &str, text: &str, cat: FactCategory) -> MemoryFact {
        MemoryFact {
            id: id.to_string(),
            fact_text: text.to_string(),
            source_chunks: vec!["c1".to_string()],
            confidence: 0.9,
            category: cat,
            first_seen: Utc::now(),
            last_updated: Utc::now(),
            contradicted_by: vec![],
            is_active: true,
        }
    }

    #[test]
    fn test_store_and_recall() {
        let db = Arc::new(SqliteConnection::open_in_memory().unwrap());
        let store = SqliteMemoryStore::new(db);

        let fact = make_fact("f1", "I enjoy hiking in the mountains", FactCategory::Preference);
        store.store(&fact).unwrap();

        let results = store.recall("hiking", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "f1");
    }

    #[test]
    fn test_forget() {
        let db = Arc::new(SqliteConnection::open_in_memory().unwrap());
        let store = SqliteMemoryStore::new(db);

        let fact = make_fact("f1", "Old belief", FactCategory::Belief);
        store.store(&fact).unwrap();
        store.forget("f1").unwrap();

        let results = store.recall("belief", 10).unwrap();
        assert_eq!(results.len(), 0); // filtered by is_active
    }

    #[test]
    fn test_contradict() {
        let db = Arc::new(SqliteConnection::open_in_memory().unwrap());
        let store = SqliteMemoryStore::new(db);

        let f1 = make_fact("f1", "I love coffee", FactCategory::Preference);
        let f2 = make_fact("f2", "I hate coffee", FactCategory::Preference);
        store.store(&f1).unwrap();
        store.store(&f2).unwrap();

        store.contradict("f1", "f2").unwrap();

        let updated = store.get_by_id("f1").unwrap().unwrap();
        assert!(updated.contradicted_by.contains(&"f2".to_string()));
    }
}
