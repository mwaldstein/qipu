//! SQLite database module for qipu

mod edges;
pub mod indexing;
mod notes;
mod rebuild;
mod repair;
mod schema;
mod search;
mod traverse;
mod validate;

use crate::error::{QipuError, Result};
use rusqlite::Connection;
use std::path::Path;

pub use schema::create_schema;

/// SQLite database for qipu
#[derive(Debug)]
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open or create the database at the given store root
    ///
    /// If auto_repair is true, validates consistency and triggers incremental repair if needed.
    /// Set to false for operations like `doctor` that want to detect issues without fixing them.
    ///
    /// If database operations fail due to corruption, attempts to delete and rebuild automatically.
    pub fn open(store_root: &Path, auto_repair: bool) -> Result<Self> {
        let db_path = store_root.join("qipu.db");

        let result = Self::open_internal(&db_path, store_root, auto_repair);

        match result {
            Ok(db) => Ok(db),
            Err(e) => {
                if Self::is_corruption_error(&e) && db_path.exists() {
                    tracing::error!(
                        "Database corruption detected at {}: {}. Attempting auto-rebuild...",
                        db_path.display(),
                        e
                    );

                    // Try to delete the corrupted database and rebuild
                    if let Err(delete_err) = std::fs::remove_file(&db_path) {
                        return Err(QipuError::Other(format!(
                            "failed to delete corrupted database: {} (original error: {})",
                            delete_err, e
                        )));
                    }

                    // Also remove WAL files if they exist
                    let wal_path = db_path.with_extension("db-wal");
                    let shm_path = db_path.with_extension("db-shm");
                    let _ = std::fs::remove_file(&wal_path);
                    let _ = std::fs::remove_file(&shm_path);

                    tracing::info!("Deleted corrupted database, attempting rebuild...");
                    Self::open_internal(&db_path, store_root, auto_repair).map_err(|rebuild_err| {
                        QipuError::Other(format!(
                            "failed to rebuild database after corruption: {} (original error: {})",
                            rebuild_err, e
                        ))
                    })
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Internal implementation of database open
    fn open_internal(db_path: &Path, store_root: &Path, auto_repair: bool) -> Result<Self> {
        let conn = Connection::open(db_path).map_err(|e| {
            QipuError::Other(format!(
                "failed to open database at {}: {}",
                db_path.display(),
                e
            ))
        })?;

        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| QipuError::Other(format!("failed to enable WAL mode: {}", e)))?;

        let needs_rebuild = create_schema(&conn)
            .map_err(|e| QipuError::Other(format!("failed to create database schema: {}", e)))?;

        let db = Database { conn };

        if needs_rebuild == schema::SchemaCreateResult::NeedsRebuild {
            tracing::info!("Rebuilding database after schema update...");
            db.rebuild(store_root, None, None)?;
        }

        let note_count: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM notes", [], |r| r.get(0))
            .unwrap_or(0);

        if note_count == 0 {
            let fs_count = Self::count_note_files(store_root)?;
            if fs_count > 0 {
                tracing::info!(
                    "Database is empty but {} note(s) found on filesystem, rebuilding...",
                    fs_count
                );
                db.rebuild(store_root, None, None)?;
            }
        } else {
            // Checkpoint WAL to ensure we see all committed changes from other processes
            // before validating consistency. Without this, rapid sequential commands may
            // see stale data in WAL mode.
            let _ = db.conn.pragma_update(None, "wal_checkpoint", "PASSIVE");

            // Validate consistency and trigger incremental repair if needed (unless disabled)
            if auto_repair && !db.validate_consistency(store_root)? {
                tracing::info!("Database inconsistent, triggering incremental repair");
                db.incremental_repair(store_root, None)?;
            }
        }

        Ok(db)
    }

    /// Check if an error indicates database corruption
    fn is_corruption_error(error: &QipuError) -> bool {
        match error {
            QipuError::Other(msg) => {
                let msg_lower = msg.to_lowercase();
                msg_lower.contains("database disk image is malformed")
                    || msg_lower.contains("corrupt")
                    || msg_lower.contains("malformed")
                    || msg_lower.contains("database is malformed")
                    || msg_lower.contains("file is not a database")
                    || msg_lower.contains("database is locked")
            }
            _ => false,
        }
    }

    pub fn get_note_count(&self) -> Result<i64> {
        self.conn
            .query_row("SELECT COUNT(*) FROM notes", [], |r| r.get(0))
            .map_err(|e| QipuError::Other(format!("failed to get note count: {}", e)))
    }

    pub fn get_tag_count(&self) -> Result<i64> {
        self.conn
            .query_row("SELECT COUNT(DISTINCT tag) FROM tags", [], |r| r.get(0))
            .map_err(|e| QipuError::Other(format!("failed to get tag count: {}", e)))
    }

    pub fn get_edge_count(&self) -> Result<i64> {
        self.conn
            .query_row("SELECT COUNT(*) FROM edges", [], |r| r.get(0))
            .map_err(|e| QipuError::Other(format!("failed to get edge count: {}", e)))
    }

    pub fn get_unresolved_count(&self) -> Result<i64> {
        self.conn
            .query_row("SELECT COUNT(*) FROM unresolved", [], |r| r.get(0))
            .map_err(|e| QipuError::Other(format!("failed to get unresolved count: {}", e)))
    }

    pub fn get_schema_version(&self) -> Result<i64> {
        self.conn
            .query_row(
                "SELECT value FROM index_meta WHERE key = 'schema_version'",
                [],
                |r| {
                    let s: String = r.get(0)?;
                    Ok(s.parse().unwrap_or(6))
                },
            )
            .map_err(|e| QipuError::Other(format!("failed to get schema version: {}", e)))
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        // Ensure all WAL changes are checkpointed before closing
        // This is critical for test reliability when database connections
        // are opened and closed rapidly
        let _ = self.conn.pragma_update(None, "wal_checkpoint", "TRUNCATE");
    }
}

#[cfg(test)]
mod tests;
