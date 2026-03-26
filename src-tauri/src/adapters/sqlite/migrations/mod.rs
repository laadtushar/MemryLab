use rusqlite::Connection;

use crate::error::AppError;

/// All migrations in order. Each is (version, description, SQL).
const MIGRATIONS: &[(i32, &str, &str)] = &[
    (1, "initial schema", include_str!("v001_initial.sql")),
    (2, "fts5 full-text index", include_str!("v002_fts5.sql")),
    (3, "llm usage log", include_str!("v003_usage_log.sql")),
    (4, "pii scan results", include_str!("v004_pii_flags.sql")),
    (5, "prompt registry", include_str!("v005_prompt_registry.sql")),
];

/// Run all pending migrations.
pub fn run_migrations(conn: &Connection) -> Result<(), AppError> {
    // Create schema_version table if it doesn't exist
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            description TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )?;

    let current_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    for &(version, description, sql) in MIGRATIONS {
        if version > current_version {
            log::info!("Running migration v{:03}: {}", version, description);
            conn.execute_batch(sql)?;
            conn.execute(
                "INSERT INTO schema_version (version, description) VALUES (?1, ?2)",
                rusqlite::params![version, description],
            )?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_run_on_fresh_db() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        run_migrations(&conn).unwrap();

        let version: i32 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(version, 5);
    }

    #[test]
    fn test_migrations_are_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap(); // should not fail
    }
}
