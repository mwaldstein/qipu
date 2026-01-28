//! SQLite database module for qipu

mod edges;
pub mod indexing;
mod notes;
mod repair;
mod schema;
mod search;
mod traverse;
mod validate;

use crate::lib::error::{QipuError, Result};
use crate::lib::note::Note;
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
            db.rebuild(store_root, None)?;
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
                db.rebuild(store_root, None)?;
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

    /// Rebuild the database from scratch by scanning all notes
    #[tracing::instrument(skip(self, store_root, progress), fields(store_root = %store_root.display()))]
    #[allow(clippy::type_complexity)]
    pub fn rebuild(
        &self,
        store_root: &Path,
        mut progress: Option<&mut dyn FnMut(usize, usize, &Note)>,
    ) -> Result<()> {
        use crate::lib::note::Note;
        use crate::lib::store::paths::{MOCS_DIR, NOTES_DIR};
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;
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

        let interrupted = Arc::new(AtomicBool::new(false));
        let interrupted_clone = Arc::clone(&interrupted);

        let _ = ctrlc::set_handler(move || {
            interrupted_clone.store(true, Ordering::SeqCst);
        });

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

        let total_notes = notes.len();
        let batch_size = 1000;

        let mut current_tx = Some(tx);

        for (i, note) in notes.iter().enumerate() {
            let tx_ref = current_tx
                .as_ref()
                .ok_or_else(|| QipuError::Other("No active transaction".to_string()))?;

            Self::insert_note_internal(tx_ref, note)?;
            Self::insert_edges_internal(tx_ref, note, &ids)?;

            // Report progress every 100 notes and at the end
            if (i + 1) % 100 == 0 || (i + 1) == total_notes {
                if let Some(cb) = progress.as_mut() {
                    cb(i + 1, total_notes, note);
                }
            }

            // Check for interruption after each note
            if interrupted.load(Ordering::SeqCst) {
                let tx = current_tx
                    .take()
                    .ok_or_else(|| QipuError::Other("No active transaction".to_string()))?;

                tx.commit().map_err(|e| {
                    QipuError::Other(format!("failed to commit transaction: {}", e))
                })?;

                tracing::info!(
                    indexed = i + 1,
                    total = total_notes,
                    "Index interrupted, partial save complete"
                );
                return Err(QipuError::Interrupted);
            }

            // Batch checkpoint: commit every N notes
            if (i + 1) % batch_size == 0 && (i + 1) < total_notes {
                let tx = current_tx
                    .take()
                    .ok_or_else(|| QipuError::Other("No active transaction".to_string()))?;

                tx.commit()
                    .map_err(|e| QipuError::Other(format!("failed to commit checkpoint: {}", e)))?;
                tracing::info!(indexed = i + 1, total = total_notes, "Checkpoint committed");

                current_tx = Some(self.conn.unchecked_transaction().map_err(|e| {
                    QipuError::Other(format!("failed to start transaction: {}", e))
                })?);
            }
        }

        let tx = current_tx.ok_or_else(|| QipuError::Other("No active transaction".to_string()))?;

        tx.commit()
            .map_err(|e| QipuError::Other(format!("failed to commit transaction: {}", e)))?;

        Ok(())
    }

    /// Resume rebuild from last checkpoint by skipping already-indexed notes
    #[tracing::instrument(skip(self, store_root, progress), fields(store_root = %store_root.display()))]
    #[allow(clippy::type_complexity)]
    pub fn rebuild_resume(
        &self,
        store_root: &Path,
        mut progress: Option<&mut dyn FnMut(usize, usize, &Note)>,
    ) -> Result<()> {
        use crate::lib::note::Note;
        use crate::lib::store::paths::{MOCS_DIR, NOTES_DIR};
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;
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

        let existing_ids: std::collections::HashSet<String> =
            self.list_note_ids()?.into_iter().collect();
        let notes_to_index: Vec<Note> = notes
            .into_iter()
            .filter(|n| !existing_ids.contains(n.id()))
            .collect();

        let ids: std::collections::HashSet<String> =
            notes_to_index.iter().map(|n| n.id().to_string()).collect();

        let total_notes = notes_to_index.len();
        let batch_size = 1000;

        if total_notes == 0 {
            tracing::info!("All notes already indexed, nothing to resume");
            return Ok(());
        }

        let interrupted = Arc::new(AtomicBool::new(false));
        let interrupted_clone = Arc::clone(&interrupted);

        let _ = ctrlc::set_handler(move || {
            interrupted_clone.store(true, Ordering::SeqCst);
        });

        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| QipuError::Other(format!("failed to start transaction: {}", e)))?;

        let mut current_tx = Some(tx);

        for (i, note) in notes_to_index.iter().enumerate() {
            let tx_ref = current_tx
                .as_ref()
                .ok_or_else(|| QipuError::Other("No active transaction".to_string()))?;

            Self::insert_note_internal(tx_ref, note)?;
            Self::insert_edges_internal(tx_ref, note, &ids)?;

            if (i + 1) % 100 == 0 || (i + 1) == total_notes {
                if let Some(cb) = progress.as_mut() {
                    cb(i + 1, total_notes, note);
                }
            }

            if interrupted.load(Ordering::SeqCst) {
                let tx = current_tx
                    .take()
                    .ok_or_else(|| QipuError::Other("No active transaction".to_string()))?;

                tx.commit().map_err(|e| {
                    QipuError::Other(format!("failed to commit transaction: {}", e))
                })?;

                tracing::info!(
                    indexed = i + 1,
                    total = total_notes,
                    "Resume interrupted, partial save complete"
                );
                return Err(QipuError::Interrupted);
            }

            if (i + 1) % batch_size == 0 && (i + 1) < total_notes {
                let tx = current_tx
                    .take()
                    .ok_or_else(|| QipuError::Other("No active transaction".to_string()))?;

                tx.commit()
                    .map_err(|e| QipuError::Other(format!("failed to commit checkpoint: {}", e)))?;
                tracing::info!(indexed = i + 1, total = total_notes, "Checkpoint committed");

                current_tx = Some(self.conn.unchecked_transaction().map_err(|e| {
                    QipuError::Other(format!("failed to start transaction: {}", e))
                })?);
            }
        }

        let tx = current_tx.ok_or_else(|| QipuError::Other("No active transaction".to_string()))?;

        tx.commit()
            .map_err(|e| QipuError::Other(format!("failed to commit transaction: {}", e)))?;

        Ok(())
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

    pub fn reindex_single_note(
        &self,
        store_root: &Path,
        note: &crate::lib::note::Note,
    ) -> Result<()> {
        use crate::lib::note::Note;
        use crate::lib::store::paths::{MOCS_DIR, NOTES_DIR};
        use walkdir::WalkDir;

        let mut all_notes = Vec::new();

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
                        Ok(n) => all_notes.push(n),
                        Err(e) => {
                            tracing::warn!(path = %path.display(), error = %e, "Failed to parse note");
                        }
                    }
                }
            }
        }

        let ids: std::collections::HashSet<String> =
            all_notes.iter().map(|n| n.id().to_string()).collect();

        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| QipuError::Other(format!("failed to start transaction: {}", e)))?;

        Self::insert_note_internal(&tx, note)?;
        Self::insert_edges_internal(&tx, note, &ids)?;

        tx.commit()
            .map_err(|e| QipuError::Other(format!("failed to commit transaction: {}", e)))?;

        Ok(())
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
