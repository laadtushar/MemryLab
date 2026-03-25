use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

use crate::error::AppError;

use super::migrations;

/// Manages a SQLite connection with WAL mode and optional encryption.
pub struct SqliteConnection {
    conn: Mutex<Connection>,
}

impl SqliteConnection {
    /// Open or create a database at the given path with WAL mode.
    /// Delegates to `open_encrypted` with an empty passphrase (no encryption).
    pub fn open(db_path: &Path) -> Result<Self, AppError> {
        Self::open_encrypted(db_path, "")
    }

    /// Open or create a database with SQLCipher encryption.
    /// The `PRAGMA key` must be issued before any other operations.
    /// An empty passphrase opens the database without encryption (for backward compat).
    pub fn open_encrypted(db_path: &Path, passphrase: &str) -> Result<Self, AppError> {
        let conn = Connection::open(db_path)?;

        // SQLCipher: set the encryption key BEFORE any other PRAGMAs
        if !passphrase.is_empty() {
            // Escape single quotes in passphrase to prevent SQL injection
            let escaped = passphrase.replace('\'', "''");
            conn.execute_batch(&format!("PRAGMA key = '{}';", escaped))?;
        }

        // Enable WAL mode for concurrent reads during import
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        // Enforce foreign keys
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        // Reasonable busy timeout (5 seconds)
        conn.busy_timeout(std::time::Duration::from_secs(5))?;

        // Verify the key is correct by reading from the database
        // This will fail with "not a database" if the passphrase is wrong
        if !passphrase.is_empty() {
            conn.execute_batch("SELECT count(*) FROM sqlite_master;")
                .map_err(|_| AppError::Other("Invalid passphrase or corrupted database".to_string()))?;
        }

        let db = Self {
            conn: Mutex::new(conn),
        };

        // Run migrations
        db.with_conn(|conn| migrations::run_migrations(conn))?;

        Ok(db)
    }

    /// Change the encryption passphrase on an already-open database.
    pub fn change_passphrase(&self, new_passphrase: &str) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Other(e.to_string()))?;
        let escaped = new_passphrase.replace('\'', "''");
        conn.execute_batch(&format!("PRAGMA rekey = '{}';", escaped))?;
        Ok(())
    }

    /// Open an in-memory database (for testing).
    pub fn open_in_memory() -> Result<Self, AppError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        let db = Self {
            conn: Mutex::new(conn),
        };

        db.with_conn(|conn| migrations::run_migrations(conn))?;

        Ok(db)
    }

    /// Execute a closure with the database connection.
    pub fn with_conn<F, T>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&Connection) -> Result<T, AppError>,
    {
        let conn = self.conn.lock().map_err(|e| AppError::Other(e.to_string()))?;
        f(&conn)
    }
}
