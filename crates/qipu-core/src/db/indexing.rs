use crate::error::{QipuError, Result};
use crate::map_db_err;
use crate::note::Note;

use super::notes::insert_helper::{insert_note_with_options, InsertOptions};

/// Indexing strategy for auto-indexing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexingStrategy {
    /// Quick index (basic only for MOCs + N recent notes)
    Quick,
}

impl IndexingStrategy {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "quick" => Some(IndexingStrategy::Quick),
            _ => None,
        }
    }
}

/// Index level for tracking which parts of a note are indexed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexLevel {
    /// Level 1: Basic index (metadata only - title, type, tags, links, sources, timestamps)
    /// Skips body and FTS5 indexing
    Basic = 1,
    /// Level 2: Full-text index (includes body content and FTS5)
    #[cfg(test)]
    Full = 2,
}

impl IndexLevel {
    #[cfg(test)]
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => IndexLevel::Basic,
            _ => IndexLevel::Full,
        }
    }
}

impl super::Database {
    /// Insert note at basic index level (metadata only, no body/FTS5)
    pub fn insert_note_basic(conn: &rusqlite::Connection, note: &Note) -> Result<()> {
        insert_note_with_options(conn, note, InsertOptions::Basic)
    }

    /// Rebuild database at basic index level (metadata only)
    pub fn rebuild_basic(&self, store_root: &std::path::Path) -> Result<()> {
        use crate::store::paths::{MOCS_DIR, NOTES_DIR};
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

        let ids: std::collections::HashSet<String> =
            notes.iter().map(|n| n.id().to_string()).collect();

        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| QipuError::transaction("start", e))?;

        tx.execute("DELETE FROM tags", [])
            .map_err(|e| map_db_err!("clear tags", e))?;

        tx.execute("DELETE FROM edges", [])
            .map_err(|e| map_db_err!("clear edges", e))?;

        tx.execute("DELETE FROM notes", [])
            .map_err(|e| map_db_err!("clear notes", e))?;

        for note in &notes {
            Self::insert_note_basic(&tx, note)?;
            Self::insert_edges_internal(&tx, note, &ids)?;
        }

        tx.commit()
            .map_err(|e| QipuError::transaction("commit", e))?;

        Ok(())
    }

    /// Count notes at basic index level
    pub fn count_basic_indexed(&self) -> Result<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM notes WHERE index_level = 1",
                [],
                |row| row.get(0),
            )
            .map_err(|e| QipuError::Other(format!("failed to count basic indexed notes: {}", e)))
    }

    /// Count notes at full-text index level
    pub fn count_full_indexed(&self) -> Result<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM notes WHERE index_level = 2",
                [],
                |row| row.get(0),
            )
            .map_err(|e| QipuError::Other(format!("failed to count full indexed notes: {}", e)))
    }

    /// Perform adaptive indexing based on note count and strategy
    pub fn adaptive_index(
        &self,
        store_root: &std::path::Path,
        config: &crate::config::AutoIndexConfig,
        force_strategy: Option<IndexingStrategy>,
    ) -> Result<IndexingResult> {
        let note_count = Self::count_note_files(store_root)?;

        if note_count == 0 {
            tracing::info!("No notes to index");
            return Ok(IndexingResult {
                notes_indexed: 0,
                strategy: IndexingStrategy::Quick,
            });
        }

        let db_count = self.get_note_count().unwrap_or(0);

        if db_count > 0 {
            tracing::info!(
                "Database already has {} notes, skipping auto-index",
                db_count
            );
            return Ok(IndexingResult {
                notes_indexed: db_count as usize,
                strategy: IndexingStrategy::Quick,
            });
        }

        let strategy = force_strategy.or_else(|| {
            if config.strategy == "adaptive" {
                if note_count < config.adaptive_threshold {
                    None
                } else {
                    Some(IndexingStrategy::Quick)
                }
            } else {
                IndexingStrategy::parse(&config.strategy)
            }
        });

        match strategy {
            Some(IndexingStrategy::Quick) => {
                tracing::info!(
                    "Auto-indexing with QUICK strategy: MOCs + {} recent notes from {} total",
                    config.quick_notes,
                    note_count
                );
                self.quick_index(store_root, config.quick_notes)?;
                Ok(IndexingResult {
                    notes_indexed: 0,
                    strategy: IndexingStrategy::Quick,
                })
            }
            None => {
                tracing::info!("Auto-indexing with BASIC strategy: {} notes", note_count);
                self.rebuild_basic(store_root)?;
                Ok(IndexingResult {
                    notes_indexed: note_count,
                    strategy: IndexingStrategy::Quick,
                })
            }
        }
    }

    /// Quick index: MOCs + N recent notes
    fn quick_index(&self, store_root: &std::path::Path, recent_count: usize) -> Result<()> {
        use crate::store::paths::{MOCS_DIR, NOTES_DIR};
        use walkdir::WalkDir;

        let mut notes = Vec::new();
        let mut moc_notes: Vec<(std::time::SystemTime, Note)> = Vec::new();
        let mut regular_notes: Vec<(std::time::SystemTime, Note)> = Vec::new();

        for dir in [store_root.join(MOCS_DIR), store_root.join(NOTES_DIR)] {
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
                        Ok(note) => {
                            let mtime = std::fs::metadata(path)
                                .and_then(|m| m.modified())
                                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

                            if note.note_type().is_moc() {
                                moc_notes.push((mtime, note));
                            } else {
                                regular_notes.push((mtime, note));
                            }
                        }
                        Err(e) => {
                            tracing::warn!(path = %path.display(), error = %e, "Failed to parse note");
                        }
                    }
                }
            }
        }

        // Add all MOCs
        for (_, note) in moc_notes {
            notes.push(note);
        }

        // Sort regular notes by mtime (most recent first) and take top N
        regular_notes.sort_by(|a, b| b.0.cmp(&a.0));
        for (_, note) in regular_notes.into_iter().take(recent_count) {
            notes.push(note);
        }

        let ids: std::collections::HashSet<String> =
            notes.iter().map(|n| n.id().to_string()).collect();

        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| QipuError::transaction("start", e))?;

        tx.execute("DELETE FROM tags", [])
            .map_err(|e| map_db_err!("clear tags", e))?;

        tx.execute("DELETE FROM edges", [])
            .map_err(|e| map_db_err!("clear edges", e))?;

        tx.execute("DELETE FROM notes", [])
            .map_err(|e| map_db_err!("clear notes", e))?;

        for note in &notes {
            Self::insert_note_basic(&tx, note)?;
            Self::insert_edges_internal(&tx, note, &ids)?;
        }

        tx.commit()
            .map_err(|e| QipuError::transaction("commit", e))?;

        tracing::info!(
            "Quick-indexed {} notes (MOCs + {} recent)",
            notes.len(),
            recent_count
        );

        Ok(())
    }
}

/// Result of adaptive indexing operation
#[derive(Debug)]
pub struct IndexingResult {
    /// Number of notes indexed
    pub notes_indexed: usize,
    /// Strategy used
    pub strategy: IndexingStrategy,
}
