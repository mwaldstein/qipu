use crate::lib::error::{QipuError, Result};
use rusqlite::params;
use std::path::Path;

impl super::Database {
    /// Incremental repair: update only notes with file mtime newer than database mtime
    ///
    /// Compares file modification time (mtime) with stored database mtime for each note.
    /// Only re-indexes notes where file mtime > database mtime, or new notes.
    /// Removes entries for deleted files.
    ///
    /// Arguments:
    /// - `store_root`: Path to store root
    /// - `progress`: Optional callback for progress reporting (indexed, total, last_note_id)
    pub fn incremental_repair(
        &self,
        store_root: &Path,
        progress: Option<&dyn Fn(usize, usize, &str)>,
    ) -> Result<()> {
        use crate::lib::note::Note;
        use crate::lib::store::paths::{MOCS_DIR, NOTES_DIR};
        use walkdir::WalkDir;

        let mut changed_notes = Vec::new();
        let mut existing_paths = std::collections::HashSet::new();

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
                    let file_mtime = std::fs::metadata(path)
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_nanos() as i64)
                        .unwrap_or(0);

                    existing_paths.insert(path.to_path_buf());

                    match Note::parse(&std::fs::read_to_string(path)?, Some(path.to_path_buf())) {
                        Ok(note) => {
                            let note_id = note.id();
                            let db_mtime: Option<i64> = self
                                .conn
                                .query_row(
                                    "SELECT mtime FROM notes WHERE id = ?1",
                                    params![note_id],
                                    |row| row.get(0),
                                )
                                .ok();

                            let needs_index = match db_mtime {
                                Some(stored_mtime) => file_mtime > stored_mtime,
                                None => true,
                            };

                            if needs_index {
                                tracing::debug!(
                                    note_id = %note_id,
                                    file_mtime = file_mtime,
                                    db_mtime = ?db_mtime,
                                    "Note needs re-indexing"
                                );
                                changed_notes.push(note);
                            }
                        }
                        Err(e) => {
                            tracing::warn!(path = %path.display(), error = %e, "Failed to parse note");
                        }
                    }
                }
            }
        }

        // Collect IDs from database and changed notes for edge resolution
        let mut all_ids: std::collections::HashSet<String> =
            self.list_note_ids()?.into_iter().collect();
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
                    if let Some(cb) = progress {
                        cb(i + 1, total_notes, note.id());
                    }
                }
            }
        }

        let mut deleted_ids = Vec::new();
        {
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
                    tracing::debug!(id = %note_id, path = %db_path, "File missing from filesystem");
                    deleted_ids.push(note_id);
                }
            }
        }

        for id in &deleted_ids {
            Self::delete_note_internal(&tx, id)?;
            tracing::debug!(id = %id, "Deleted note from database");
        }

        tx.commit()
            .map_err(|e| QipuError::Other(format!("failed to commit transaction: {}", e)))?;

        tracing::info!(
            updated = changed_notes.len(),
            deleted = deleted_ids.len(),
            "Incremental repair complete"
        );

        Ok(())
    }
}
