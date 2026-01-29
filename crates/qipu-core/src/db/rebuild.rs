use crate::error::{QipuError, Result};
use crate::note::Note;
use crate::store::paths::{MOCS_DIR, NOTES_DIR};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use walkdir::WalkDir;

use super::Database;

impl Database {
    pub(super) fn count_note_files(store_root: &Path) -> Result<usize> {
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

    #[tracing::instrument(skip(self, store_root, progress), fields(store_root = %store_root.display()))]
    #[allow(clippy::type_complexity)]
    pub fn rebuild(
        &self,
        store_root: &Path,
        mut progress: Option<&mut dyn FnMut(usize, usize, &Note)>,
    ) -> Result<()> {
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

            Database::insert_note_internal(tx_ref, note)?;
            Database::insert_edges_internal(tx_ref, note, &ids)?;

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
                    "Index interrupted, partial save complete"
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

    #[tracing::instrument(skip(self, store_root, progress), fields(store_root = %store_root.display()))]
    #[allow(clippy::type_complexity)]
    pub fn rebuild_resume(
        &self,
        store_root: &Path,
        mut progress: Option<&mut dyn FnMut(usize, usize, &Note)>,
    ) -> Result<()> {
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

            Database::insert_note_internal(tx_ref, note)?;
            Database::insert_edges_internal(tx_ref, note, &ids)?;

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

    pub fn reindex_single_note(&self, store_root: &Path, note: &Note) -> Result<()> {
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

        Database::insert_note_internal(&tx, note)?;
        Database::insert_edges_internal(&tx, note, &ids)?;

        tx.commit()
            .map_err(|e| QipuError::Other(format!("failed to commit transaction: {}", e)))?;

        Ok(())
    }
}
