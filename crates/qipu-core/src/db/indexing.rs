use crate::error::{QipuError, Result};
use crate::note::Note;
use rusqlite::{params, Connection};

/// Indexing strategy for auto-indexing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexingStrategy {
    /// Quick index (basic only for MOCs + N recent notes)
    Quick,
}

impl IndexingStrategy {
    pub fn from_str(s: &str) -> Option<Self> {
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
    #[allow(dead_code)]
    Full = 2,
}

impl IndexLevel {
    #[allow(dead_code)]
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => IndexLevel::Basic,
            _ => IndexLevel::Full,
        }
    }
}

impl super::Database {
    /// Insert note at basic index level (metadata only, no body/FTS5)
    pub fn insert_note_basic(conn: &Connection, note: &Note) -> Result<()> {
        let path_str = note
            .path
            .as_ref()
            .and_then(|p| p.to_str())
            .ok_or_else(|| QipuError::Other(format!("invalid path for note {}", note.id())))?;

        let created_str = note.frontmatter.created.map(|d| d.to_rfc3339());
        let updated_str = note.frontmatter.updated.map(|d| d.to_rfc3339());
        let mtime = note
            .path
            .as_ref()
            .and_then(|p| std::fs::metadata(p).ok())
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_nanos() as i64)
            .unwrap_or(0);

        let compacts_json =
            serde_json::to_string(&note.frontmatter.compacts).unwrap_or_else(|_| "[]".to_string());
        let sources_json =
            serde_json::to_string(&note.frontmatter.sources).unwrap_or_else(|_| "[]".to_string());
        let verified_int = note.frontmatter.verified.map(|b| if b { 1 } else { 0 });
        let custom_json =
            serde_json::to_string(&note.frontmatter.custom).unwrap_or_else(|_| "{}".to_string());

        conn.execute(
            "INSERT OR REPLACE INTO notes (id, title, type, path, created, updated, body, mtime, value, compacts, author, verified, source, sources, generated_by, prompt_hash, custom_json, index_level)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
            params![
                note.id(),
                note.title(),
                note.note_type().to_string(),
                path_str,
                created_str,
                updated_str,
                &note.body,
                mtime,
                note.frontmatter.value.or(Some(50)),
                compacts_json,
                note.frontmatter.author.as_ref(),
                verified_int,
                note.frontmatter.source.as_ref(),
                sources_json,
                note.frontmatter.generated_by.as_ref(),
                note.frontmatter.prompt_hash.as_ref(),
                custom_json,
                IndexLevel::Basic as i32,
            ],
        )
        .map_err(|e| QipuError::Other(format!("failed to insert note {}: {}", note.id(), e)))?;

        let rowid: i64 = conn.last_insert_rowid();

        // Skip FTS5 indexing for basic level
        conn.execute(
            "INSERT OR REPLACE INTO notes_fts(rowid, title, body, tags) VALUES (?1, ?2, ?3, ?4)",
            params![
                rowid,
                note.title(),
                "", // Empty body for basic index
                note.frontmatter.tags.join(" "),
            ],
        )
        .map_err(|e| {
            QipuError::Other(format!(
                "failed to insert note {} into FTS: {}",
                note.id(),
                e
            ))
        })?;

        for tag in &note.frontmatter.tags {
            conn.execute(
                "INSERT OR REPLACE INTO tags (note_id, tag) VALUES (?1, ?2)",
                params![note.id(), tag],
            )
            .map_err(|e| {
                QipuError::Other(format!(
                    "failed to insert tag '{}' for note {}: {}",
                    tag,
                    note.id(),
                    e
                ))
            })?;
        }

        Ok(())
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
            .map_err(|e| QipuError::Other(format!("failed to start transaction: {}", e)))?;

        tx.execute("DELETE FROM tags", [])
            .map_err(|e| QipuError::Other(format!("failed to clear tags: {}", e)))?;

        tx.execute("DELETE FROM edges", [])
            .map_err(|e| QipuError::Other(format!("failed to clear edges: {}", e)))?;

        tx.execute("DELETE FROM notes", [])
            .map_err(|e| QipuError::Other(format!("failed to clear notes: {}", e)))?;

        for note in &notes {
            Self::insert_note_basic(&tx, note)?;
            Self::insert_edges_internal(&tx, note, &ids)?;
        }

        tx.commit()
            .map_err(|e| QipuError::Other(format!("failed to commit transaction: {}", e)))?;

        Ok(())
    }

    /// Get index level for a note
    #[allow(dead_code)]
    pub fn get_note_index_level(&self, note_id: &str) -> Result<Option<IndexLevel>> {
        match self.conn.query_row(
            "SELECT index_level FROM notes WHERE id = ?1",
            params![note_id],
            |row| {
                let level: i32 = row.get(0)?;
                Ok(IndexLevel::from_i32(level))
            },
        ) {
            Ok(level) => Ok(Some(level)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(QipuError::Other(format!(
                "failed to query index level: {}",
                e
            ))),
        }
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

    /// Upgrade notes from basic to full-text index level
    #[allow(dead_code)]
    pub fn upgrade_to_full_text(&self, note_ids: &[String]) -> Result<usize> {
        let mut upgraded = 0;

        for note_id in note_ids {
            if let Some(note) = self.get_note(note_id)? {
                let tx = self
                    .conn
                    .unchecked_transaction()
                    .map_err(|e| QipuError::Other(format!("failed to start transaction: {}", e)))?;

                // Update FTS5 with full body
                let rowid: i64 = tx
                    .query_row(
                        "SELECT rowid FROM notes WHERE id = ?1",
                        params![note_id],
                        |row| row.get(0),
                    )
                    .map_err(|e| QipuError::Other(format!("failed to get rowid: {}", e)))?;

                tx.execute(
                    "UPDATE notes_fts SET body = ?1 WHERE rowid = ?2",
                    params![&note.body, rowid],
                )
                .map_err(|e| {
                    QipuError::Other(format!("failed to update FTS5 for note {}: {}", note_id, e))
                })?;

                // Update index_level
                tx.execute(
                    "UPDATE notes SET index_level = 2 WHERE id = ?1",
                    params![note_id],
                )
                .map_err(|e| {
                    QipuError::Other(format!(
                        "failed to update index_level for note {}: {}",
                        note_id, e
                    ))
                })?;

                tx.commit().map_err(|e| {
                    QipuError::Other(format!("failed to commit transaction: {}", e))
                })?;

                upgraded += 1;
            }
        }

        Ok(upgraded)
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
                IndexingStrategy::from_str(&config.strategy)
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
            .map_err(|e| QipuError::Other(format!("failed to start transaction: {}", e)))?;

        tx.execute("DELETE FROM tags", [])
            .map_err(|e| QipuError::Other(format!("failed to clear tags: {}", e)))?;

        tx.execute("DELETE FROM edges", [])
            .map_err(|e| QipuError::Other(format!("failed to clear edges: {}", e)))?;

        tx.execute("DELETE FROM notes", [])
            .map_err(|e| QipuError::Other(format!("failed to clear notes: {}", e)))?;

        for note in &notes {
            Self::insert_note_basic(&tx, note)?;
            Self::insert_edges_internal(&tx, note, &ids)?;
        }

        tx.commit()
            .map_err(|e| QipuError::Other(format!("failed to commit transaction: {}", e)))?;

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
#[allow(dead_code)]
pub struct IndexingResult {
    /// Number of notes indexed
    pub notes_indexed: usize,
    /// Strategy used
    pub strategy: IndexingStrategy,
}
