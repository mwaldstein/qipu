//! SQLite database module for qipu

mod edges;
mod notes;
mod repair;
mod schema;
mod search;
mod traverse;
mod validate;

use crate::lib::error::{QipuError, Result};
use rusqlite::Connection;
use std::path::Path;

pub use schema::create_schema;

/// SQLite database for qipu
#[derive(Debug)]
pub struct Database {
    conn: Connection,
}

impl Database {
    pub(super) fn count_note_files(store_root: &Path) -> Result<usize> {
        use crate::lib::store::paths::{MOCS_DIR, NOTES_DIR};
        use walkdir::WalkDir;

        let mut count = 0;

        for dir in [store_root.join(NOTES_DIR), store_root.join(MOCS_DIR)] {
            if !dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&dir)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.path().extension().is_some_and(|e| e == "md") {
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Open or create the database at the given store root
    pub fn open(store_root: &Path) -> Result<Self> {
        let db_path = store_root.join("qipu.db");

        let conn = Connection::open(&db_path).map_err(|e| {
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
            db.rebuild(store_root)?;
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
                db.rebuild(store_root)?;
            }
        } else {
            // Just validate consistency and warn if issues found
            // Doctor command will check and optionally fix with --fix flag
            let _ = db.validate_consistency(store_root)?;
        }

        Ok(db)
    }

    /// Rebuild the database from scratch by scanning all notes
    #[allow(dead_code)]
    #[tracing::instrument(skip(self, store_root), fields(store_root = %store_root.display()))]
    pub fn rebuild(&self, store_root: &Path) -> Result<()> {
        use crate::lib::note::Note;
        use crate::lib::store::paths::{MOCS_DIR, NOTES_DIR};
        use walkdir::WalkDir;

        let mut notes = Vec::new();

        for dir in [store_root.join(NOTES_DIR), store_root.join(MOCS_DIR)] {
            if !dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&dir)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "md") {
                    match Note::parse(&std::fs::read_to_string(path)?, Some(path.to_path_buf())) {
                        Ok(note) => notes.push(note),
                        Err(e) => {
                            tracing::warn!(path = %path.display(), error = %e, "Failed to parse note");
                        }
                    }
                }
            }
        }

        // Collect all IDs from the notes we're about to insert
        let ids: std::collections::HashSet<String> =
            notes.iter().map(|n| n.id().to_string()).collect();

        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| QipuError::Other(format!("failed to start transaction: {}", e)))?;

        tx.execute("DELETE FROM tags", [])
            .map_err(|e| QipuError::Other(format!("failed to clear tags: {}", e)))?;

        tx.execute("DELETE FROM edges", [])
            .map_err(|e| QipuError::Other(format!("failed to clear edges: {}", e)))?;

        tx.execute("DELETE FROM notes", [])
            .map_err(|e| QipuError::Other(format!("failed to clear notes: {}", e)))?;

        for note in notes {
            Self::insert_note_internal(&tx, &note)?;
            Self::insert_edges_internal(&tx, &note, &ids)?;
        }

        tx.commit()
            .map_err(|e| QipuError::Other(format!("failed to commit transaction: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests;
