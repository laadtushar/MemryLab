use std::sync::Arc;

use rusqlite::params;

use crate::error::AppError;

use super::connection::SqliteConnection;

pub struct PromptVersion {
    pub id: String,
    pub name: String,
    pub version: String,
    pub template: String,
    pub is_active: bool,
    pub created_at: String,
}

pub struct SqlitePromptStore {
    db: Arc<SqliteConnection>,
}

impl SqlitePromptStore {
    pub fn new(db: Arc<SqliteConnection>) -> Self {
        Self { db }
    }

    pub fn list_all(&self) -> Result<Vec<PromptVersion>, AppError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, version, template, is_active, created_at FROM prompt_registry ORDER BY name, version",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(PromptVersion {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    version: row.get(2)?,
                    template: row.get(3)?,
                    is_active: row.get::<_, i32>(4)? != 0,
                    created_at: row.get(5)?,
                })
            })?;
            let mut results = Vec::new();
            for row in rows {
                results.push(row?);
            }
            Ok(results)
        })
    }

    pub fn get_active(&self, name: &str) -> Result<Option<PromptVersion>, AppError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, version, template, is_active, created_at FROM prompt_registry WHERE name = ?1 AND is_active = 1 LIMIT 1",
            )?;
            let result = stmt
                .query_row(params![name], |row| {
                    Ok(PromptVersion {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        version: row.get(2)?,
                        template: row.get(3)?,
                        is_active: row.get::<_, i32>(4)? != 0,
                        created_at: row.get(5)?,
                    })
                })
                .optional();
            match result {
                Ok(val) => Ok(val),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(AppError::from(e)),
            }
        })
    }

    pub fn save(&self, prompt: &PromptVersion) -> Result<(), AppError> {
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO prompt_registry (id, name, version, template, is_active, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    prompt.id,
                    prompt.name,
                    prompt.version,
                    prompt.template,
                    prompt.is_active as i32,
                    prompt.created_at,
                ],
            )?;
            Ok(())
        })
    }

    pub fn set_active(&self, name: &str, version: &str) -> Result<(), AppError> {
        self.db.with_conn(|conn| {
            // Deactivate all versions of this prompt
            conn.execute(
                "UPDATE prompt_registry SET is_active = 0 WHERE name = ?1",
                params![name],
            )?;
            // Activate the specified version
            conn.execute(
                "UPDATE prompt_registry SET is_active = 1 WHERE name = ?1 AND version = ?2",
                params![name, version],
            )?;
            Ok(())
        })
    }

    pub fn seed_defaults(&self) -> Result<(), AppError> {
        self.db.with_conn(|conn| {
            let count: i32 = conn.query_row(
                "SELECT COUNT(*) FROM prompt_registry",
                [],
                |row| row.get(0),
            )?;

            if count > 0 {
                return Ok(());
            }

            use crate::prompts::templates::*;

            let defaults: Vec<(&str, &str)> = vec![
                ("theme_extraction", THEME_EXTRACTION_V1),
                ("sentiment", SENTIMENT_V1),
                ("belief_extraction", BELIEF_EXTRACTION_V1),
                ("entity_extraction", ENTITY_EXTRACTION_V1),
                ("insight_generation", INSIGHT_GENERATION_V1),
                ("query_classification", QUERY_CLASSIFICATION_V1),
                ("rag_response", RAG_RESPONSE_V1),
                ("evolution_diff", EVOLUTION_DIFF_V1),
                ("contradiction_check", CONTRADICTION_CHECK_V1),
                ("narrative_generation", NARRATIVE_GENERATION_V1),
            ];

            for (name, template) in defaults {
                let id = uuid::Uuid::new_v4().to_string();
                conn.execute(
                    "INSERT INTO prompt_registry (id, name, version, template, is_active, created_at) VALUES (?1, ?2, 'v1', ?3, 1, datetime('now'))",
                    params![id, name, template],
                )?;
            }

            Ok(())
        })
    }
}

use rusqlite::OptionalExtension;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_and_list() {
        let db = Arc::new(SqliteConnection::open_in_memory().unwrap());
        let store = SqlitePromptStore::new(db);
        store.seed_defaults().unwrap();
        let all = store.list_all().unwrap();
        assert!(all.len() >= 10);
    }

    #[test]
    fn test_get_active() {
        let db = Arc::new(SqliteConnection::open_in_memory().unwrap());
        let store = SqlitePromptStore::new(db);
        store.seed_defaults().unwrap();
        let active = store.get_active("sentiment").unwrap();
        assert!(active.is_some());
        assert_eq!(active.unwrap().version, "v1");
    }

    #[test]
    fn test_set_active() {
        let db = Arc::new(SqliteConnection::open_in_memory().unwrap());
        let store = SqlitePromptStore::new(db);
        store.seed_defaults().unwrap();

        // Save a v2
        store
            .save(&PromptVersion {
                id: uuid::Uuid::new_v4().to_string(),
                name: "sentiment".to_string(),
                version: "v2".to_string(),
                template: "new template".to_string(),
                is_active: false,
                created_at: "2024-01-01T00:00:00".to_string(),
            })
            .unwrap();

        store.set_active("sentiment", "v2").unwrap();

        let active = store.get_active("sentiment").unwrap().unwrap();
        assert_eq!(active.version, "v2");
    }

    #[test]
    fn test_seed_idempotent() {
        let db = Arc::new(SqliteConnection::open_in_memory().unwrap());
        let store = SqlitePromptStore::new(db);
        store.seed_defaults().unwrap();
        let count1 = store.list_all().unwrap().len();
        store.seed_defaults().unwrap(); // should not add more
        let count2 = store.list_all().unwrap().len();
        assert_eq!(count1, count2);
    }
}
