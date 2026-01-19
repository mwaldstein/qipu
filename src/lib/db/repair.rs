use crate::lib::error::{QipuError, Result};
use rusqlite::params;
use std::path::Path;

impl super::Database {
    /// Incremental repair: update only changed notes since last sync
    ///
    /// Finds files changed since the last sync timestamp, re-parses and updates
    /// those entries, and removes entries for deleted files.
    pub fn incremental_repair(&self, store_root: &Path) -> Result<()> {
        use crate::lib::note::Note;
        use crate::lib::store::paths::{MOCS_DIR, NOTES_DIR};
        use walkdir::WalkDir;

        let last_sync: i64 = match self.conn.query_row(
            "SELECT value FROM index_meta WHERE key = 'last_sync'",
            [],
            |row| row.get::<_, String>(0),
        ) {
            Ok(s) => match s.parse::<i64>() {
                Ok(v) => v,
                Err(_) => {
                    tracing::warn!("Invalid last_sync value in database, resetting to 0");
                    0
                }
            },
            Err(rusqlite::Error::QueryReturnedNoRows) => 0,
            Err(e) => {
                return Err(QipuError::Other(format!(
                    "failed to query last_sync: {}",
                    e
                )));
            }
        };

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
                    let mtime = std::fs::metadata(path)
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0);

                    existing_paths.insert(path.to_path_buf());

                    if mtime > last_sync {
                        tracing::debug!(
                            path = %path.display(),
                            "File changed since last sync, re-parsing"
                        );

                        match Note::parse(&std::fs::read_to_string(path)?, Some(path.to_path_buf()))
                        {
                            Ok(note) => changed_notes.push(note),
                            Err(e) => {
                                tracing::warn!(path = %path.display(), error = %e, "Failed to parse note");
                            }
                        }
                    }
                }
            }
        }

        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| QipuError::Other(format!("failed to start transaction: {}", e)))?;

        if !changed_notes.is_empty() {
            for note in &changed_notes {
                Self::insert_note_internal(&tx, note)?;
                Self::insert_edges_internal(&tx, note, store_root)?;
                tracing::debug!(id = %note.id(), "Updated note in database");
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

        let now = chrono::Utc::now().timestamp();
        tx.execute(
            "INSERT OR REPLACE INTO index_meta (key, value) VALUES ('last_sync', ?1)",
            params![now.to_string()],
        )
        .map_err(|e| QipuError::Other(format!("failed to update last_sync: {}", e)))?;

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
