use crate::error::{QipuError, Result};
use crate::note::Note;
use rusqlite::params;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use walkdir::WalkDir;

fn file_mtime(path: &Path) -> i64 {
    std::fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_nanos() as i64)
        .unwrap_or(0)
}

impl super::Database {
    /// Incremental repair: update notes whose file mtime differs from the database mtime
    ///
    /// Compares file modification time (mtime) with stored database mtime for each note.
    /// Only re-indexes notes where file mtime differs from the stored mtime, or new notes.
    /// Removes entries for deleted files.
    ///
    /// Arguments:
    /// - `store_root`: Path to store root
    /// - `progress`: Optional callback for progress reporting (indexed, total, last_note)
    /// - `interrupted`: Optional atomic bool flag for interrupt signal handling
    #[allow(clippy::type_complexity)]
    pub fn incremental_repair(
        &self,
        store_root: &Path,
        mut progress: Option<&mut dyn FnMut(usize, usize, &Note)>,
        interrupted: Option<&std::sync::Arc<std::sync::atomic::AtomicBool>>,
    ) -> Result<()> {
        let (changed_notes, existing_paths) = self.collect_changed_notes(store_root)?;

        // Collect IDs from database and changed notes for edge resolution
        let mut all_ids: HashSet<String> = self.list_note_ids()?.into_iter().collect();
        for note in &changed_notes {
            all_ids.insert(note.id().to_string());
        }

        let total_notes = changed_notes.len();

        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| QipuError::Other(format!("failed to start transaction: {}", e)))?;

        if !changed_notes.is_empty() {
            for (i, note) in changed_notes.iter().enumerate() {
                Self::insert_note_internal(&tx, note)?;
                Self::insert_edges_internal(&tx, note, &all_ids)?;
                tracing::debug!(id = %note.id(), "Updated note in database");

                // Report progress every 100 notes and at the end
                if (i + 1) % 100 == 0 || (i + 1) == total_notes {
                    if let Some(cb) = progress.as_mut() {
                        cb(i + 1, total_notes, note);
                    }
                }

                // Check for interruption after each note
                if interrupted
                    .map(|i| i.load(Ordering::SeqCst))
                    .unwrap_or(false)
                {
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
            }
        }

        let deleted_ids = Self::delete_missing_notes(&tx, &existing_paths)?;

        tx.commit()
            .map_err(|e| QipuError::Other(format!("failed to commit transaction: {}", e)))?;

        tracing::info!(
            updated = changed_notes.len(),
            deleted = deleted_ids.len(),
            "Incremental repair complete"
        );

        Ok(())
    }

    fn collect_changed_notes(&self, store_root: &Path) -> Result<(Vec<Note>, HashSet<PathBuf>)> {
        use crate::store::paths::{MOCS_DIR, NOTES_DIR};

        let mut changed_notes = Vec::new();
        let mut existing_paths = HashSet::new();

        for dir in [store_root.join(NOTES_DIR), store_root.join(MOCS_DIR)] {
            if !dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&dir).follow_links(true).into_iter().flatten() {
                let path = entry.path();
                if path.extension().is_none_or(|e| e != "md") {
                    continue;
                }

                existing_paths.insert(path.to_path_buf());
                match Note::parse(&std::fs::read_to_string(path)?, Some(path.to_path_buf())) {
                    Ok(note) if self.note_needs_index(path, &note) => changed_notes.push(note),
                    Ok(_) => {}
                    Err(e) => {
                        tracing::warn!(path = %path.display(), error = %e, "Failed to parse note");
                    }
                }
            }
        }

        Ok((changed_notes, existing_paths))
    }

    fn note_needs_index(&self, path: &Path, note: &Note) -> bool {
        let note_id = note.id();
        let db_mtime: Option<i64> = self
            .conn
            .query_row(
                "SELECT mtime FROM notes WHERE id = ?1",
                params![note_id],
                |row| row.get(0),
            )
            .ok();
        let file_mtime = file_mtime(path);
        let needs_index = db_mtime.is_none_or(|stored_mtime| file_mtime != stored_mtime);

        if needs_index {
            tracing::debug!(
                note_id = %note_id,
                file_mtime = file_mtime,
                db_mtime = ?db_mtime,
                "Note needs re-indexing"
            );
        }

        needs_index
    }

    fn delete_missing_notes(
        tx: &rusqlite::Transaction<'_>,
        existing_paths: &HashSet<PathBuf>,
    ) -> Result<Vec<String>> {
        let mut deleted_ids = Vec::new();
        let mut stmt = tx
            .prepare("SELECT id, path FROM notes")
            .map_err(|e| QipuError::Other(format!("failed to prepare note query: {}", e)))?;
        let mut rows = stmt
            .query([])
            .map_err(|e| QipuError::Other(format!("failed to query notes: {}", e)))?;

        while let Some(row) = rows
            .next()
            .map_err(|e| QipuError::Other(format!("failed to read note: {}", e)))?
        {
            let note_id: String = row
                .get(0)
                .map_err(|e| QipuError::Other(format!("failed to get id: {}", e)))?;
            let db_path: String = row
                .get(1)
                .map_err(|e| QipuError::Other(format!("failed to get path: {}", e)))?;

            if !existing_paths.contains(Path::new(&db_path)) {
                Self::delete_note_internal(tx, &note_id)?;
                tracing::debug!(id = %note_id, path = %db_path, "Deleted missing note from database");
                deleted_ids.push(note_id);
            }
        }

        Ok(deleted_ids)
    }
}
