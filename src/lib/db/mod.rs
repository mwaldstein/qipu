//! SQLite database module for qipu

mod schema;

use crate::lib::error::{QipuError, Result};
use crate::lib::graph::types::Direction;
use crate::lib::index::types::{Edge, LinkSource, NoteMetadata, SearchResult};
use crate::lib::note::LinkType;
use crate::lib::note::Note;
use crate::lib::note::NoteType;
use chrono::Utc;
use rusqlite::{params, Connection};
use std::path::Path;
use std::str::FromStr;

pub use schema::create_schema;

/// SQLite database for qipu
#[derive(Debug)]
pub struct Database {
    conn: Connection,
}

/// Parse tags from a space-separated string
#[allow(dead_code)]
fn parse_tags(tags_str: &str) -> Vec<String> {
    tags_str
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

impl Database {
    fn count_note_files(store_root: &Path) -> Result<usize> {
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

        create_schema(&conn)
            .map_err(|e| QipuError::Other(format!("failed to create database schema: {}", e)))?;

        let db = Database { conn };

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
        }

        Ok(db)
    }

    /// Rebuild the database from scratch by scanning all notes
    #[allow(dead_code)]
    pub fn rebuild(&self, store_root: &Path) -> Result<()> {
        use crate::lib::index::links;
        use crate::lib::note::Note;
        use crate::lib::store::paths::{MOCS_DIR, NOTES_DIR};
        use std::collections::{HashMap, HashSet};
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
            Self::insert_edges_internal(&tx, &note, store_root)?;
        }

        tx.commit()
            .map_err(|e| QipuError::Other(format!("failed to commit transaction: {}", e)))?;

        Ok(())
    }

    fn insert_note_internal(conn: &Connection, note: &Note) -> Result<()> {
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
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        conn.execute(
            "INSERT OR REPLACE INTO notes (id, title, type, path, created, updated, body, mtime)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                note.id(),
                note.title(),
                note.note_type().to_string(),
                path_str,
                created_str,
                updated_str,
                &note.body,
                mtime,
            ],
        )
        .map_err(|e| QipuError::Other(format!("failed to insert note {}: {}", note.id(), e)))?;

        let rowid: i64 = conn.last_insert_rowid();

        conn.execute(
            "INSERT OR REPLACE INTO notes_fts(rowid, title, body, tags) VALUES (?1, ?2, ?3, ?4)",
            params![
                rowid,
                note.title(),
                &note.body,
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
            .map_err(|e| QipuError::Other(format!("failed to insert tag {}: {}", tag, e)))?;
        }

        Ok(())
    }

    fn insert_edges_internal(conn: &Connection, note: &Note, store_root: &Path) -> Result<()> {
        use crate::lib::index::links;
        use crate::lib::store::paths::{MOCS_DIR, NOTES_DIR};
        use std::collections::{HashMap, HashSet};

        let mut unresolved = HashSet::new();
        let path_to_id = HashMap::new();

        if note.path.is_some() {
            let parent_path = note.path.as_ref().unwrap().parent().unwrap();

            if let Ok(existing_ids) = crate::lib::store::Store::discover(parent_path) {
                let ids = existing_ids.existing_ids().unwrap_or_default();
                let edges = links::extract_links(
                    note,
                    &ids,
                    &mut unresolved,
                    note.path.as_deref(),
                    &path_to_id,
                );

                // Delete all existing edges for this note before inserting new ones
                conn.execute("DELETE FROM edges WHERE source_id = ?1", params![note.id()])
                    .map_err(|e| {
                        QipuError::Other(format!(
                            "failed to delete edges for note {}: {}",
                            note.id(),
                            e
                        ))
                    })?;

                for edge in edges {
                    let link_type_str = edge.link_type.to_string();
                    let inline_flag =
                        if matches!(edge.source, crate::lib::index::types::LinkSource::Inline) {
                            1
                        } else {
                            0
                        };

                    conn.execute(
                        "INSERT INTO edges (source_id, target_id, link_type, inline) VALUES (?1, ?2, ?3, ?4)",
                        params![edge.from, edge.to, link_type_str, inline_flag],
                    )
                    .map_err(|e| {
                        QipuError::Other(format!("failed to insert edge {} -> {}: {}", edge.from, edge.to, e))
                    })?;
                }

                // Delete all existing unresolved references for this note
                conn.execute(
                    "DELETE FROM unresolved WHERE source_id = ?1",
                    params![note.id()],
                )
                .map_err(|e| {
                    QipuError::Other(format!(
                        "failed to delete unresolved for note {}: {}",
                        note.id(),
                        e
                    ))
                })?;

                // Insert unresolved references
                for target_ref in unresolved {
                    conn.execute(
                        "INSERT OR IGNORE INTO unresolved (source_id, target_ref) VALUES (?1, ?2)",
                        params![note.id(), target_ref],
                    )
                    .map_err(|e| {
                        QipuError::Other(format!(
                            "failed to insert unresolved {} -> {}: {}",
                            note.id(),
                            target_ref,
                            e
                        ))
                    })?;
                }
            }
        }

        Ok(())
    }

    pub fn get_note_metadata(&self, note_id: &str) -> Result<Option<NoteMetadata>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title, type, path, created, updated FROM notes WHERE id = ?1")
            .map_err(|e| QipuError::Other(format!("failed to prepare query: {}", e)))?;

        let note_opt = stmt.query_row(params![note_id], |row| {
            let id: String = row.get(0)?;
            let title: String = row.get(1)?;
            let type_str: String = row.get(2)?;
            let path: String = row.get(3)?;
            let created: Option<String> = row.get(4)?;
            let updated: Option<String> = row.get(5)?;

            let note_type = NoteType::from_str(&type_str).unwrap_or(NoteType::Fleeting);

            let created_dt = created
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));
            let updated_dt = updated
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));

            Ok((id, title, note_type, path, created_dt, updated_dt))
        });

        match note_opt {
            Ok((id, title, note_type, path, created, updated)) => {
                // Get tags for this note
                let mut tag_stmt = self
                    .conn
                    .prepare("SELECT tag FROM tags WHERE note_id = ?1")
                    .map_err(|e| QipuError::Other(format!("failed to prepare tag query: {}", e)))?;

                let mut tags = Vec::new();
                let mut tag_rows = tag_stmt
                    .query(params![&id])
                    .map_err(|e| QipuError::Other(format!("failed to query tags: {}", e)))?;

                while let Some(row) = tag_rows
                    .next()
                    .map_err(|e| QipuError::Other(format!("failed to read tag: {}", e)))?
                {
                    tags.push(
                        row.get(0)
                            .map_err(|e| QipuError::Other(format!("failed to read tag: {}", e)))?,
                    );
                }

                Ok(Some(NoteMetadata {
                    id,
                    title,
                    note_type,
                    tags,
                    path,
                    created,
                    updated,
                }))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(QipuError::Other(format!(
                "failed to query note metadata: {}",
                e
            ))),
        }
    }

    pub fn insert_note(&self, note: &Note) -> Result<()> {
        let created_str = note.frontmatter.created.map(|dt| dt.to_rfc3339());
        let updated_str = note.frontmatter.updated.map(|dt| dt.to_rfc3339());
        let mtime = note
            .path
            .as_ref()
            .and_then(|p| std::fs::metadata(p).ok())
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let tags_str = note.frontmatter.tags.join(" ");

        // Update notes table
        self.conn
            .execute(
                "INSERT OR REPLACE INTO notes (id, title, type, path, created, updated, body, mtime) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    note.id(),
                    note.frontmatter.title,
                    note.frontmatter.note_type.unwrap_or(NoteType::Fleeting).to_string(),
                    note.path.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
                    created_str,
                    updated_str,
                    &note.body,
                    mtime,
                ],
            )
            .map_err(|e| QipuError::Other(format!("failed to insert note {}: {}", note.id(), e)))?;

        let rowid: i64 = self.conn.last_insert_rowid();

        self.conn
            .execute(
                "INSERT OR REPLACE INTO notes_fts(rowid, title, body, tags) VALUES (?1, ?2, ?3, ?4)",
                params![
                    rowid,
                    note.frontmatter.title,
                    &note.body,
                    tags_str,
                ],
            )
            .map_err(|e| QipuError::Other(format!("failed to insert note into FTS5 {}: {}", note.id(), e)))?;

        // Update tags
        self.conn
            .execute("DELETE FROM tags WHERE note_id = ?1", params![note.id()])
            .map_err(|e| {
                QipuError::Other(format!(
                    "failed to delete tags for note {}: {}",
                    note.id(),
                    e
                ))
            })?;

        for tag in &note.frontmatter.tags {
            self.conn
                .execute(
                    "INSERT INTO tags (note_id, tag) VALUES (?1, ?2)",
                    params![note.id(), tag],
                )
                .map_err(|e| {
                    QipuError::Other(format!(
                        "failed to insert tag {} for note {}: {}",
                        tag,
                        note.id(),
                        e
                    ))
                })?;
        }

        Ok(())
    }

    /// Insert edges (links) for a note into the database
    pub fn insert_edges(&self, note: &Note) -> Result<()> {
        use crate::lib::index::links;
        use std::collections::{HashMap, HashSet};

        let mut unresolved = HashSet::new();
        let path_to_id = HashMap::new();

        if note.path.is_some() {
            let parent_path = note.path.as_ref().unwrap().parent().unwrap();

            if let Ok(existing_ids) = crate::lib::store::Store::discover(parent_path) {
                let ids = existing_ids.existing_ids().unwrap_or_default();
                let edges = links::extract_links(
                    note,
                    &ids,
                    &mut unresolved,
                    note.path.as_deref(),
                    &path_to_id,
                );

                // Delete all existing edges for this note before inserting new ones
                self.conn
                    .execute("DELETE FROM edges WHERE source_id = ?1", params![note.id()])
                    .map_err(|e| {
                        QipuError::Other(format!(
                            "failed to delete edges for note {}: {}",
                            note.id(),
                            e
                        ))
                    })?;

                for edge in edges {
                    let link_type_str = edge.link_type.to_string();
                    let inline_flag =
                        if matches!(edge.source, crate::lib::index::types::LinkSource::Inline) {
                            1
                        } else {
                            0
                        };

                    self.conn
                        .execute(
                            "INSERT INTO edges (source_id, target_id, link_type, inline) VALUES (?1, ?2, ?3, ?4)",
                            params![edge.from, edge.to, link_type_str, inline_flag],
                        )
                        .map_err(|e| {
                            QipuError::Other(format!("failed to insert edge {} -> {}: {}", edge.from, edge.to, e))
                        })?;
                }

                // Force WAL checkpoint to ensure changes are written to disk
                let _ = self.conn.pragma_update(None, "wal_checkpoint", "TRUNCATE");

                // Delete all existing unresolved references for this note
                self.conn
                    .execute(
                        "DELETE FROM unresolved WHERE source_id = ?1",
                        params![note.id()],
                    )
                    .map_err(|e| {
                        QipuError::Other(format!(
                            "failed to delete unresolved refs for note {}: {}",
                            note.id(),
                            e
                        ))
                    })?;

                for unresolved_ref in unresolved {
                    self.conn
                        .execute(
                            "INSERT INTO unresolved (source_id, target_ref) VALUES (?1, ?2)",
                            params![note.id(), unresolved_ref],
                        )
                        .map_err(|e| {
                            QipuError::Other(format!(
                                "failed to insert unresolved ref {}: {}",
                                unresolved_ref, e
                            ))
                        })?;
                }
            }
        }

        Ok(())
    }

    /// Perform full-text search using FTS5 with BM25 ranking
    ///
    /// Field weights for BM25:
    /// - Title: 2.0x boost
    /// - Body: 1.0x (baseline)
    /// - Tags: 1.5x boost
    #[allow(dead_code)]
    pub fn search(
        &self,
        query: &str,
        type_filter: Option<NoteType>,
        tag_filter: Option<&str>,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        // Wrap query in double quotes to treat it as a phrase search
        // This prevents FTS5 from interpreting hyphens as column references
        let fts_query = format!("\"{}\"", query.replace('"', "\"\""));

        let limit_i64 = limit as i64;

        let mut sql = String::from(
            r#"
            SELECT n.rowid, n.id, n.title, n.path, n.type, notes_fts.tags,
                   bm25(notes_fts, 2.0, 1.0, 1.5) AS rank
            FROM notes_fts
            JOIN notes n ON notes_fts.rowid = n.rowid
            WHERE notes_fts MATCH ?
        "#,
        );

        let type_filter_str = type_filter.map(|t| t.to_string());
        let tag_filter_str = tag_filter.map(|t| t.to_string());

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(fts_query)];

        if type_filter_str.is_some() {
            sql.push_str(" AND n.type = ?");
            params.push(Box::new(type_filter_str.unwrap()));
        }

        if tag_filter_str.is_some() {
            sql.push_str(
                " AND EXISTS (SELECT 1 FROM tags WHERE tags.note_id = n.id AND tags.tag = ?)",
            );
            params.push(Box::new(tag_filter_str.unwrap()));
        }

        sql.push_str(" ORDER BY rank LIMIT ?;");
        params.push(Box::new(limit_i64));

        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| QipuError::Other(format!("failed to prepare search query: {}", e)))?;

        let mut results = Vec::new();

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let mut rows = stmt
            .query(param_refs.as_slice())
            .map_err(|e| QipuError::Other(format!("failed to execute search query: {}", e)))?;

        while let Some(row) = rows
            .next()
            .map_err(|e| QipuError::Other(format!("failed to read search results: {}", e)))?
        {
            let _rowid: i64 = row
                .get(0)
                .map_err(|e| QipuError::Other(format!("failed to get rowid: {}", e)))?;
            let id: String = row
                .get(1)
                .map_err(|e| QipuError::Other(format!("failed to get id: {}", e)))?;
            let title: String = row
                .get(2)
                .map_err(|e| QipuError::Other(format!("failed to get title: {}", e)))?;
            let path: String = row
                .get(3)
                .map_err(|e| QipuError::Other(format!("failed to get path: {}", e)))?;
            let note_type_str: String = row
                .get(4)
                .map_err(|e| QipuError::Other(format!("failed to get type: {}", e)))?;
            let tags_str: String = row
                .get(5)
                .map_err(|e| QipuError::Other(format!("failed to get tags: {}", e)))?;
            let rank: f64 = row
                .get(6)
                .map_err(|e| QipuError::Other(format!("failed to get rank: {}", e)))?;

            let note_type = NoteType::from_str(&note_type_str).unwrap_or(NoteType::Fleeting);
            let tags = parse_tags(&tags_str);

            results.push(SearchResult {
                id,
                title,
                note_type,
                tags,
                path,
                match_context: None,
                relevance: rank,
                via: None,
            });
        }

        Ok(results)
    }

    pub fn delete_note(&self, note_id: &str) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM edges WHERE source_id = ?1 OR target_id = ?1",
                params![note_id],
            )
            .map_err(|e| {
                QipuError::Other(format!(
                    "failed to delete edges for note {}: {}",
                    note_id, e
                ))
            })?;

        self.conn
            .execute(
                "DELETE FROM unresolved WHERE source_id = ?1",
                params![note_id],
            )
            .map_err(|e| {
                QipuError::Other(format!(
                    "failed to delete unresolved refs for note {}: {}",
                    note_id, e
                ))
            })?;

        self.conn
            .execute("DELETE FROM tags WHERE note_id = ?1", params![note_id])
            .map_err(|e| {
                QipuError::Other(format!("failed to delete tags for note {}: {}", note_id, e))
            })?;

        let deleted_rows = self
            .conn
            .execute("DELETE FROM notes WHERE id = ?1", params![note_id])
            .map_err(|e| QipuError::Other(format!("failed to delete note {}: {}", note_id, e)))?;

        if deleted_rows == 0 {
            return Err(QipuError::NoteNotFound {
                id: note_id.to_string(),
            });
        }

        Ok(())
    }

    /// List notes with optional filters
    ///
    /// Returns note metadata sorted by created date (newest first), then by id
    pub fn list_notes(
        &self,
        type_filter: Option<NoteType>,
        tag_filter: Option<&str>,
        since: Option<chrono::DateTime<Utc>>,
    ) -> Result<Vec<NoteMetadata>> {
        let mut sql = String::from(
            r#"
            SELECT n.id, n.title, n.type, n.path, n.created, n.updated
            FROM notes n
        "#,
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        let mut has_where = false;

        if type_filter.is_some() {
            sql.push_str(" WHERE n.type = ?");
            params.push(Box::new(type_filter.unwrap().to_string()));
            has_where = true;
        }

        if tag_filter.is_some() {
            if has_where {
                sql.push_str(" AND");
            } else {
                sql.push_str(" WHERE");
                has_where = true;
            }
            sql.push_str(" EXISTS (SELECT 1 FROM tags WHERE tags.note_id = n.id AND tags.tag = ?)");
            params.push(Box::new(tag_filter.unwrap().to_string()));
        }

        if since.is_some() {
            if has_where {
                sql.push_str(" AND");
            } else {
                sql.push_str(" WHERE");
                has_where = true;
            }
            sql.push_str(" n.created >= ?");
            params.push(Box::new(since.unwrap().to_rfc3339()));
        }

        sql.push_str(" ORDER BY n.created DESC, n.id");

        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| QipuError::Other(format!("failed to prepare list query: {}", e)))?;

        let mut results = Vec::new();

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let mut rows = stmt
            .query(param_refs.as_slice())
            .map_err(|e| QipuError::Other(format!("failed to execute list query: {}", e)))?;

        while let Some(row) = rows
            .next()
            .map_err(|e| QipuError::Other(format!("failed to read list results: {}", e)))?
        {
            let id: String = row
                .get(0)
                .map_err(|e| QipuError::Other(format!("failed to get id: {}", e)))?;
            let title: String = row
                .get(1)
                .map_err(|e| QipuError::Other(format!("failed to get title: {}", e)))?;
            let type_str: String = row
                .get(2)
                .map_err(|e| QipuError::Other(format!("failed to get type: {}", e)))?;
            let path: String = row
                .get(3)
                .map_err(|e| QipuError::Other(format!("failed to get path: {}", e)))?;
            let created: Option<String> = row
                .get(4)
                .map_err(|e| QipuError::Other(format!("failed to get created: {}", e)))?;
            let updated: Option<String> = row
                .get(5)
                .map_err(|e| QipuError::Other(format!("failed to get updated: {}", e)))?;

            let note_type = NoteType::from_str(&type_str).unwrap_or(NoteType::Fleeting);

            let created_dt = created
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));
            let updated_dt = updated
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));

            let mut tag_stmt = self
                .conn
                .prepare("SELECT tag FROM tags WHERE note_id = ?1")
                .map_err(|e| QipuError::Other(format!("failed to prepare tag query: {}", e)))?;

            let mut tags = Vec::new();
            let mut tag_rows = tag_stmt
                .query(params![&id])
                .map_err(|e| QipuError::Other(format!("failed to query tags: {}", e)))?;

            while let Some(row) = tag_rows
                .next()
                .map_err(|e| QipuError::Other(format!("failed to read tag: {}", e)))?
            {
                tags.push(
                    row.get(0)
                        .map_err(|e| QipuError::Other(format!("failed to read tag: {}", e)))?,
                );
            }

            results.push(NoteMetadata {
                id,
                title,
                note_type,
                tags,
                path,
                created: created_dt,
                updated: updated_dt,
            });
        }

        Ok(results)
    }

    /// Get backlinks (inbound edges) to a note
    pub fn get_backlinks(&self, note_id: &str) -> Result<Vec<Edge>> {
        let mut stmt = self
            .conn
            .prepare("SELECT source_id, link_type, inline FROM edges WHERE target_id = ?1")
            .map_err(|e| QipuError::Other(format!("failed to prepare backlinks query: {}", e)))?;

        let mut rows = stmt
            .query(params![note_id])
            .map_err(|e| QipuError::Other(format!("failed to execute backlinks query: {}", e)))?;

        let mut backlinks = Vec::new();

        while let Some(row) = rows
            .next()
            .map_err(|e| QipuError::Other(format!("failed to read backlink: {}", e)))?
        {
            let source_id: String = row
                .get(0)
                .map_err(|e| QipuError::Other(format!("failed to get source_id: {}", e)))?;
            let link_type_str: String = row
                .get(1)
                .map_err(|e| QipuError::Other(format!("failed to get link_type: {}", e)))?;
            let inline: i64 = row
                .get(2)
                .map_err(|e| QipuError::Other(format!("failed to get inline: {}", e)))?;

            let link_type = LinkType::from(link_type_str);
            let source = if inline == 1 {
                LinkSource::Inline
            } else {
                LinkSource::Typed
            };

            backlinks.push(Edge {
                from: source_id,
                to: note_id.to_string(),
                link_type,
                source,
            });
        }

        Ok(backlinks)
    }

    /// Get outbound edges from a note (links FROM this note)
    pub fn get_outbound_edges(&self, note_id: &str) -> Result<Vec<Edge>> {
        let mut stmt = self
            .conn
            .prepare("SELECT target_id, link_type, inline FROM edges WHERE source_id = ?1")
            .map_err(|e| {
                QipuError::Other(format!("failed to prepare outbound edges query: {}", e))
            })?;

        let mut rows = stmt.query(params![note_id]).map_err(|e| {
            QipuError::Other(format!("failed to execute outbound edges query: {}", e))
        })?;

        let mut edges = Vec::new();

        while let Some(row) = rows
            .next()
            .map_err(|e| QipuError::Other(format!("failed to read outbound edge: {}", e)))?
        {
            let target_id: String = row
                .get(0)
                .map_err(|e| QipuError::Other(format!("failed to get target_id: {}", e)))?;
            let link_type_str: String = row
                .get(1)
                .map_err(|e| QipuError::Other(format!("failed to get link_type: {}", e)))?;
            let inline: i64 = row
                .get(2)
                .map_err(|e| QipuError::Other(format!("failed to get inline: {}", e)))?;

            let link_type = LinkType::from(link_type_str);
            let source = if inline == 1 {
                LinkSource::Inline
            } else {
                LinkSource::Typed
            };

            edges.push(Edge {
                from: note_id.to_string(),
                to: target_id,
                link_type,
                source,
            });
        }

        Ok(edges)
    }

    /// Perform graph traversal using recursive CTE
    pub fn traverse(
        &self,
        start_id: &str,
        direction: Direction,
        max_hops: u32,
        max_nodes: Option<usize>,
    ) -> Result<Vec<String>> {
        let sql = match direction {
            Direction::Out => {
                "WITH RECURSIVE reachable(id, depth) AS (
                    SELECT ?1, 0
                    UNION
                    SELECT e.target_id, r.depth + 1
                    FROM reachable r JOIN edges e ON e.source_id = r.id
                    WHERE r.depth < ?2
                ) SELECT DISTINCT id FROM reachable"
            }
            Direction::In => {
                "WITH RECURSIVE reachable(id, depth) AS (
                    SELECT ?1, 0
                    UNION
                    SELECT e.source_id, r.depth + 1
                    FROM reachable r JOIN edges e ON e.target_id = r.id
                    WHERE r.depth < ?2
                ) SELECT DISTINCT id FROM reachable"
            }
            Direction::Both => {
                "WITH RECURSIVE reachable(id, depth) AS (
                    SELECT ?1, 0
                    UNION
                    SELECT e.target_id, r.depth + 1
                    FROM reachable r JOIN edges e ON e.source_id = r.id
                    WHERE r.depth < ?2
                    UNION
                    SELECT e.source_id, r.depth + 1
                    FROM reachable r JOIN edges e ON e.target_id = r.id
                    WHERE r.depth < ?2
                ) SELECT DISTINCT id FROM reachable"
            }
        };

        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|e| QipuError::Other(format!("failed to prepare traversal query: {}", e)))?;

        let mut rows = stmt
            .query(params![start_id, max_hops])
            .map_err(|e| QipuError::Other(format!("failed to execute traversal query: {}", e)))?;

        let mut reachable = Vec::new();

        while let Some(row) = rows
            .next()
            .map_err(|e| QipuError::Other(format!("failed to read traversal result: {}", e)))?
        {
            let id: String = row
                .get(0)
                .map_err(|e| QipuError::Other(format!("failed to get note id: {}", e)))?;
            reachable.push(id);
        }

        if let Some(max) = max_nodes {
            if reachable.len() > max {
                reachable.truncate(max);
            }
        }

        Ok(reachable)
    }

    /// Find notes with duplicate IDs
    pub fn get_duplicate_ids(&self) -> Result<Vec<(String, Vec<String>)>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, GROUP_CONCAT(path, ', ') as paths
                 FROM notes
                 GROUP BY id
                 HAVING COUNT(*) > 1",
            )
            .map_err(|e| QipuError::Other(format!("failed to prepare duplicate query: {}", e)))?;

        let duplicates = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let paths_str: String = row.get(1)?;
                let paths: Vec<String> = paths_str.split(", ").map(|s| s.to_string()).collect();
                Ok((id, paths))
            })
            .map_err(|e| QipuError::Other(format!("failed to query duplicates: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| QipuError::Other(format!("failed to read duplicate rows: {}", e)))?;

        Ok(duplicates)
    }

    /// Get all broken links from the unresolved table
    pub fn get_broken_links(&self) -> Result<Vec<(String, String)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT source_id, target_ref FROM unresolved")
            .map_err(|e| {
                QipuError::Other(format!("failed to prepare broken links query: {}", e))
            })?;

        let broken_links = stmt
            .query_map([], |row| {
                let source_id: String = row.get(0)?;
                let target_ref: String = row.get(1)?;
                Ok((source_id, target_ref))
            })
            .map_err(|e| QipuError::Other(format!("failed to query broken links: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| QipuError::Other(format!("failed to read broken link rows: {}", e)))?;

        Ok(broken_links)
    }

    /// Find notes that are in database but missing from filesystem
    pub fn get_missing_files(&self) -> Result<Vec<(String, String)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, path FROM notes")
            .map_err(|e| {
                QipuError::Other(format!("failed to prepare missing files query: {}", e))
            })?;

        let missing = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let path: String = row.get(1)?;
                Ok((id, path))
            })
            .map_err(|e| QipuError::Other(format!("failed to query notes: {}", e)))?
            .filter_map(|r| r.ok())
            .filter(|(_, path)| !Path::new(path).exists())
            .collect();

        Ok(missing)
    }

    /// Find orphaned notes (notes with no incoming links)
    pub fn get_orphaned_notes(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id FROM notes
                 WHERE id NOT IN (SELECT DISTINCT target_id FROM edges)",
            )
            .map_err(|e| {
                QipuError::Other(format!("failed to prepare orphaned notes query: {}", e))
            })?;

        let orphaned = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| QipuError::Other(format!("failed to query orphaned notes: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| QipuError::Other(format!("failed to read orphaned note rows: {}", e)))?;

        Ok(orphaned)
    }

    /// Quick consistency check between database and filesystem
    ///
    /// Returns true if database is consistent with filesystem, false otherwise
    pub fn validate_consistency(&self, store_root: &Path) -> Result<bool> {
        let db_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM notes", [], |r| r.get(0))
            .map_err(|e| QipuError::Other(format!("failed to count notes in DB: {}", e)))?;

        let fs_count = Self::count_note_files(store_root)?;

        if db_count != fs_count as i64 {
            tracing::warn!(
                "Consistency check failed: DB has {} notes, filesystem has {}",
                db_count,
                fs_count
            );
            return Ok(false);
        }

        let mut stmt = self
            .conn
            .prepare("SELECT path, mtime FROM notes ORDER BY RANDOM() LIMIT 5")
            .map_err(|e| QipuError::Other(format!("failed to prepare mtime query: {}", e)))?;

        let mut rows = stmt
            .query([])
            .map_err(|e| QipuError::Other(format!("failed to query mtime samples: {}", e)))?;

        while let Some(row) = rows
            .next()
            .map_err(|e| QipuError::Other(format!("failed to read mtime sample: {}", e)))?
        {
            let path_str: String = row
                .get(0)
                .map_err(|e| QipuError::Other(format!("failed to get path: {}", e)))?;
            let db_mtime: i64 = row
                .get(1)
                .map_err(|e| QipuError::Other(format!("failed to get mtime: {}", e)))?;

            let path = Path::new(&path_str);
            if !path.exists() {
                tracing::warn!("Consistency check failed: file {} missing", path_str);
                return Ok(false);
            }

            let fs_mtime = std::fs::metadata(path)
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            if db_mtime != fs_mtime {
                tracing::warn!(
                    "Consistency check failed: file {} mtime mismatch (DB: {}, FS: {})",
                    path_str,
                    db_mtime,
                    fs_mtime
                );
                return Ok(false);
            }
        }

        Ok(true)
    }

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

    fn delete_note_internal(conn: &Connection, note_id: &str) -> Result<()> {
        conn.execute("DELETE FROM edges WHERE source_id = ?1", params![note_id])
            .map_err(|e| {
                QipuError::Other(format!(
                    "failed to delete edges for note {}: {}",
                    note_id, e
                ))
            })?;

        conn.execute("DELETE FROM edges WHERE target_id = ?1", params![note_id])
            .map_err(|e| {
                QipuError::Other(format!(
                    "failed to delete backlinks for note {}: {}",
                    note_id, e
                ))
            })?;

        conn.execute(
            "DELETE FROM unresolved WHERE source_id = ?1",
            params![note_id],
        )
        .map_err(|e| {
            QipuError::Other(format!(
                "failed to delete unresolved for note {}: {}",
                note_id, e
            ))
        })?;

        conn.execute("DELETE FROM tags WHERE note_id = ?1", params![note_id])
            .map_err(|e| {
                QipuError::Other(format!("failed to delete tags for note {}: {}", note_id, e))
            })?;

        conn.execute("DELETE FROM notes WHERE id = ?1", params![note_id])
            .map_err(|e| QipuError::Other(format!("failed to delete note {}: {}", note_id, e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::store::Store;
    use tempfile::tempdir;

    #[test]
    fn test_database_open_creates_tables() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        let db = Database::open(store.root()).unwrap();

        let count: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
                [],
                |row: &rusqlite::Row| row.get(0),
            )
            .unwrap();

        assert!(count >= 6);
    }

    #[test]
    fn test_database_rebuild_populates_notes() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        store
            .create_note("Test Note 1", None, &["tag1".to_string()], None)
            .unwrap();
        store
            .create_note("Test Note 2", None, &["tag2".to_string()], None)
            .unwrap();

        let db = Database::open(store.root()).unwrap();
        db.rebuild(store.root()).unwrap();

        let note_count: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM notes", [], |row: &rusqlite::Row| {
                row.get(0)
            })
            .unwrap();

        assert_eq!(note_count, 2);

        let tag_count: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM tags", [], |row: &rusqlite::Row| {
                row.get(0)
            })
            .unwrap();

        assert_eq!(tag_count, 2);
    }

    #[test]
    fn test_database_rebuild_cleans_old_data() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        let mut note = store
            .create_note("Test Note", None, &["tag1".to_string()], None)
            .unwrap();

        let db = Database::open(store.root()).unwrap();
        db.rebuild(store.root()).unwrap();

        let initial_count: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM tags", [], |row: &rusqlite::Row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(initial_count, 1);

        note.frontmatter.tags = vec!["tag2".to_string()];
        store.save_note(&mut note).unwrap();

        db.rebuild(store.root()).unwrap();

        let final_count: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM tags", [], |row: &rusqlite::Row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(final_count, 1);

        let tag_value: String = db
            .conn
            .query_row("SELECT tag FROM tags", [], |row: &rusqlite::Row| row.get(0))
            .unwrap();
        assert_eq!(tag_value, "tag2");
    }

    #[test]
    fn test_insert_note_with_fts() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        let _note = store
            .create_note_with_content(
                "Test Note",
                None,
                &["test-tag".to_string()],
                "This is a test body with some content",
                None,
            )
            .unwrap();

        let db = Database::open(store.root()).unwrap();
        db.rebuild(store.root()).unwrap();

        let fts_count: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM notes_fts",
                [],
                |row: &rusqlite::Row| row.get(0),
            )
            .unwrap();

        assert_eq!(fts_count, 1);

        let title: String = db
            .conn
            .query_row("SELECT title FROM notes_fts", [], |row: &rusqlite::Row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(title, "Test Note");
    }

    #[test]
    fn test_empty_store_rebuild() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        let db = Database::open(store.root()).unwrap();
        db.rebuild(store.root()).unwrap();

        let note_count: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM notes", [], |row: &rusqlite::Row| {
                row.get(0)
            })
            .unwrap();

        assert_eq!(note_count, 0);
    }

    #[test]
    fn test_search_fts_basic() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        store
            .create_note_with_content(
                "Test Note One",
                None,
                &["test-tag".to_string()],
                "This is a test body with some content",
                None,
            )
            .unwrap();

        store
            .create_note_with_content(
                "Another Note",
                None,
                &["other-tag".to_string()],
                "Different content here",
                None,
            )
            .unwrap();

        let db = Database::open(store.root()).unwrap();
        db.rebuild(store.root()).unwrap();

        let results = db.search("test", None, None, 10).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Test Note One");
        assert!(results[0].id.starts_with("qp-"));
    }

    #[test]
    fn test_search_fts_tag_boost() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        store
            .create_note_with_content(
                "Test Note",
                None,
                &["test-tag".to_string()],
                "content",
                None,
            )
            .unwrap();

        store
            .create_note_with_content(
                "Other Note",
                None,
                &["other-tag".to_string()],
                "test test test",
                None,
            )
            .unwrap();

        let db = Database::open(store.root()).unwrap();
        db.rebuild(store.root()).unwrap();

        let results = db.search("test", None, None, 10).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Test Note");
    }

    #[test]
    fn test_search_with_type_filter() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        store
            .create_note_with_content("Test Note", Some(NoteType::Fleeting), &[], "test", None)
            .unwrap();

        store
            .create_note_with_content("Test MOC", Some(NoteType::Moc), &[], "test", None)
            .unwrap();

        let db = Database::open(store.root()).unwrap();
        db.rebuild(store.root()).unwrap();

        let results = db
            .search("test", Some(NoteType::Fleeting), None, 10)
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Test Note");
    }

    #[test]
    fn test_search_with_tag_filter() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        store
            .create_note_with_content(
                "Test Note One",
                None,
                &["test-tag".to_string()],
                "content",
                None,
            )
            .unwrap();

        store
            .create_note_with_content(
                "Test Note Two",
                None,
                &["other-tag".to_string()],
                "content",
                None,
            )
            .unwrap();

        let db = Database::open(store.root()).unwrap();
        db.rebuild(store.root()).unwrap();

        let results = db.search("test", None, Some("test-tag"), 10).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Test Note One");
    }

    #[test]
    fn test_search_empty_query() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        store
            .create_note("Test Note", None, &["test-tag".to_string()], None)
            .unwrap();

        let db = Database::open(store.root()).unwrap();
        db.rebuild(store.root()).unwrap();

        let results = db.search("", None, None, 10).unwrap();

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_limit() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        for i in 0..5 {
            store
                .create_note_with_content(
                    &format!("Test Note {}", i),
                    None,
                    &[],
                    "test content",
                    None,
                )
                .unwrap();
        }

        let db = Database::open(store.root()).unwrap();
        db.rebuild(store.root()).unwrap();

        let results = db.search("test", None, None, 3).unwrap();

        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_get_backlinks() {
        use crate::lib::note::TypedLink;

        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        let note1 = store.create_note("Source Note", None, &[], None).unwrap();
        let note2 = store.create_note("Target Note", None, &[], None).unwrap();
        let note3 = store
            .create_note("Another Source", None, &[], None)
            .unwrap();

        let note1_id = note1.id();
        let note2_id = note2.id();
        let note3_id = note3.id();

        let mut note1 = store.get_note(note1_id).unwrap();
        note1.frontmatter.links.push(TypedLink {
            link_type: LinkType::from("related"),
            id: note2_id.to_string(),
        });
        store.save_note(&mut note1).unwrap();

        let mut note3 = store.get_note(note3_id).unwrap();
        note3.frontmatter.links.push(TypedLink {
            link_type: LinkType::from("related"),
            id: note2_id.to_string(),
        });
        store.save_note(&mut note3).unwrap();

        let db = store.db();
        let backlinks = db.get_backlinks(note2_id).unwrap();

        assert_eq!(backlinks.len(), 2);

        let backlink1 = backlinks
            .iter()
            .find(|e| e.from == note1_id)
            .expect("Expected backlink from note1");
        assert_eq!(backlink1.to, note2_id);
        assert_eq!(backlink1.link_type.as_str(), "related");
        assert_eq!(backlink1.source, LinkSource::Typed);

        let backlink2 = backlinks
            .iter()
            .find(|e| e.from == note3_id)
            .expect("Expected backlink from note3");
        assert_eq!(backlink2.to, note2_id);
        assert_eq!(backlink2.link_type.as_str(), "related");
        assert_eq!(backlink2.source, LinkSource::Typed);
    }

    #[test]
    fn test_traverse_outbound() {
        use crate::lib::note::TypedLink;

        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        let note1 = store.create_note("Note 1", None, &[], None).unwrap();
        let note2 = store.create_note("Note 2", None, &[], None).unwrap();
        let note3 = store.create_note("Note 3", None, &[], None).unwrap();
        let note4 = store.create_note("Note 4", None, &[], None).unwrap();

        let note1_id = note1.id();
        let note2_id = note2.id();
        let note3_id = note3.id();
        let note4_id = note4.id();

        let mut note1 = store.get_note(note1_id).unwrap();
        note1.frontmatter.links.push(TypedLink {
            link_type: LinkType::from("related"),
            id: note2_id.to_string(),
        });
        store.save_note(&mut note1).unwrap();

        let mut note2 = store.get_note(note2_id).unwrap();
        note2.frontmatter.links.push(TypedLink {
            link_type: LinkType::from("supports"),
            id: note3_id.to_string(),
        });
        store.save_note(&mut note2).unwrap();

        let mut note3 = store.get_note(note3_id).unwrap();
        note3.frontmatter.links.push(TypedLink {
            link_type: LinkType::from("related"),
            id: note4_id.to_string(),
        });
        store.save_note(&mut note3).unwrap();

        let db = store.db();
        let reachable = db.traverse(note1_id, Direction::Out, 3, None).unwrap();

        assert_eq!(reachable.len(), 4);
        assert!(reachable.iter().any(|id| id == note1_id));
        assert!(reachable.iter().any(|id| id == note2_id));
        assert!(reachable.iter().any(|id| id == note3_id));
        assert!(reachable.iter().any(|id| id == note4_id));
    }

    #[test]
    fn test_traverse_inbound() {
        use crate::lib::note::TypedLink;

        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        let note1 = store.create_note("Note 1", None, &[], None).unwrap();
        let note2 = store.create_note("Note 2", None, &[], None).unwrap();
        let note3 = store.create_note("Note 3", None, &[], None).unwrap();

        let note1_id = note1.id();
        let note2_id = note2.id();
        let note3_id = note3.id();

        let mut note1 = store.get_note(note1_id).unwrap();
        note1.frontmatter.links.push(TypedLink {
            link_type: LinkType::from("related"),
            id: note2_id.to_string(),
        });
        store.save_note(&mut note1).unwrap();

        let mut note3 = store.get_note(note3_id).unwrap();
        note3.frontmatter.links.push(TypedLink {
            link_type: LinkType::from("supports"),
            id: note2_id.to_string(),
        });
        store.save_note(&mut note3).unwrap();

        let db = store.db();
        let reachable = db.traverse(note2_id, Direction::In, 3, None).unwrap();

        assert_eq!(reachable.len(), 3);
        assert!(reachable.iter().any(|id| id == note1_id));
        assert!(reachable.iter().any(|id| id == note2_id));
        assert!(reachable.iter().any(|id| id == note3_id));
    }

    #[test]
    fn test_traverse_both_directions() {
        use crate::lib::note::TypedLink;

        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        let note1 = store.create_note("Note 1", None, &[], None).unwrap();
        let note2 = store.create_note("Note 2", None, &[], None).unwrap();
        let note3 = store.create_note("Note 3", None, &[], None).unwrap();

        let note1_id = note1.id();
        let note2_id = note2.id();
        let note3_id = note3.id();

        let mut note1 = store.get_note(note1_id).unwrap();
        note1.frontmatter.links.push(TypedLink {
            link_type: LinkType::from("related"),
            id: note2_id.to_string(),
        });
        store.save_note(&mut note1).unwrap();

        let mut note3 = store.get_note(note3_id).unwrap();
        note3.frontmatter.links.push(TypedLink {
            link_type: LinkType::from("related"),
            id: note2_id.to_string(),
        });
        store.save_note(&mut note3).unwrap();

        let db = store.db();
        let reachable = db.traverse(note2_id, Direction::Both, 3, None).unwrap();

        assert_eq!(reachable.len(), 3);
        assert!(reachable.iter().any(|id| id == note1_id));
        assert!(reachable.iter().any(|id| id == note2_id));
        assert!(reachable.iter().any(|id| id == note3_id));
    }

    #[test]
    fn test_traverse_max_hops() {
        use crate::lib::note::TypedLink;

        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        let note1 = store.create_note("Note 1", None, &[], None).unwrap();
        let note2 = store.create_note("Note 2", None, &[], None).unwrap();
        let note3 = store.create_note("Note 3", None, &[], None).unwrap();

        let note1_id = note1.id();
        let note2_id = note2.id();
        let note3_id = note3.id();

        let mut note1 = store.get_note(note1_id).unwrap();
        note1.frontmatter.links.push(TypedLink {
            link_type: LinkType::from("related"),
            id: note2_id.to_string(),
        });
        store.save_note(&mut note1).unwrap();

        let mut note2 = store.get_note(note2_id).unwrap();
        note2.frontmatter.links.push(TypedLink {
            link_type: LinkType::from("related"),
            id: note3_id.to_string(),
        });
        store.save_note(&mut note2).unwrap();

        let db = store.db();
        let reachable = db.traverse(note1_id, Direction::Out, 1, None).unwrap();

        assert_eq!(reachable.len(), 2);
        assert!(reachable.iter().any(|id| id == note1_id));
        assert!(reachable.iter().any(|id| id == note2_id));
        assert!(!reachable.iter().any(|id| id == note3_id));
    }

    #[test]
    fn test_traverse_max_nodes() {
        use crate::lib::note::TypedLink;

        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        let note1 = store.create_note("Note 1", None, &[], None).unwrap();
        let note2 = store.create_note("Note 2", None, &[], None).unwrap();
        let note3 = store.create_note("Note 3", None, &[], None).unwrap();

        let note1_id = note1.id();
        let note2_id = note2.id();
        let note3_id = note3.id();

        let mut note1 = store.get_note(note1_id).unwrap();
        note1.frontmatter.links.push(TypedLink {
            link_type: LinkType::from("related"),
            id: note2_id.to_string(),
        });
        store.save_note(&mut note1).unwrap();

        let mut note2 = store.get_note(note2_id).unwrap();
        note2.frontmatter.links.push(TypedLink {
            link_type: LinkType::from("related"),
            id: note3_id.to_string(),
        });
        store.save_note(&mut note2).unwrap();

        let db = store.db();
        let reachable = db.traverse(note1_id, Direction::Out, 3, Some(2)).unwrap();

        assert_eq!(reachable.len(), 2);
        assert!(reachable.iter().any(|id| id == note1_id));
        assert!(reachable.iter().any(|id| id == note2_id));
    }

    #[test]
    fn test_startup_validation_rebuilds_if_empty_db_has_notes() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        store
            .create_note("Test Note 1", None, &["tag1".to_string()], None)
            .unwrap();
        store
            .create_note("Test Note 2", None, &["tag2".to_string()], None)
            .unwrap();

        let db_path = store.root().join("qipu.db");

        let _ = std::fs::remove_file(&db_path);

        let db = Database::open(store.root()).unwrap();

        let note_count: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
            .unwrap();

        assert_eq!(note_count, 2);
    }

    #[test]
    fn test_startup_validation_skips_rebuild_if_empty_db_no_notes() {
        let dir = tempdir().unwrap();
        Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        let db_path = dir.path().join(".qipu").join("qipu.db");

        let _ = std::fs::remove_file(&db_path);

        let db = Database::open(&dir.path().join(".qipu")).unwrap();

        let note_count: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
            .unwrap();

        assert_eq!(note_count, 0);
    }

    #[test]
    fn test_validate_consistency_matching_state() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        store
            .create_note("Test Note", None, &["tag".to_string()], None)
            .unwrap();

        let db = store.db();
        assert!(db.validate_consistency(store.root()).unwrap());
    }

    #[test]
    fn test_validate_consistency_count_mismatch() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        store
            .create_note("Test Note", None, &["tag".to_string()], None)
            .unwrap();

        let db = store.db();

        let db_count: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
            .unwrap();

        db.conn
            .execute(
                "INSERT INTO notes (id, title, type, path, mtime) VALUES (?1, ?2, ?3, ?4, ?5)",
                params!["qp-fake-id", "Fake Note", "Fleeting", "/fake/path.md", 0],
            )
            .unwrap();

        let new_db_count: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
            .unwrap();

        assert_eq!(new_db_count, db_count + 1);

        assert!(!db.validate_consistency(store.root()).unwrap());
    }

    #[test]
    fn test_validate_consistency_missing_file() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        store
            .create_note("Test Note", None, &["tag".to_string()], None)
            .unwrap();

        let db = store.db();

        let mut stmt = db.conn.prepare("SELECT path FROM notes").unwrap();
        let mut rows = stmt.query([]).unwrap();
        let row = rows.next().unwrap().unwrap();
        let path_str: String = row.get(0).unwrap();
        let path = std::path::PathBuf::from(path_str);

        std::fs::remove_file(&path).unwrap();

        assert!(!db.validate_consistency(store.root()).unwrap());
    }

    #[test]
    fn test_validate_consistency_mtime_mismatch() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        store
            .create_note("Test Note", None, &["tag".to_string()], None)
            .unwrap();

        let db = store.db();

        let mut stmt = db.conn.prepare("SELECT id FROM notes").unwrap();
        let mut rows = stmt.query([]).unwrap();
        let row = rows.next().unwrap().unwrap();
        let note_id: String = row.get(0).unwrap();

        db.conn
            .execute(
                "UPDATE notes SET mtime = ?1 WHERE id = ?2",
                params![999, note_id],
            )
            .unwrap();

        assert!(!db.validate_consistency(store.root()).unwrap());
    }

    #[test]
    fn test_validate_consistency_empty_database() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        let db = store.db();
        assert!(db.validate_consistency(store.root()).unwrap());
    }

    #[test]
    fn test_validate_consistency_samples_multiple_notes() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        for i in 0..10 {
            store
                .create_note(
                    &format!("Test Note {}", i),
                    None,
                    &["tag".to_string()],
                    None,
                )
                .unwrap();
        }

        let db = store.db();
        assert!(db.validate_consistency(store.root()).unwrap());
    }

    #[test]
    fn test_incremental_repair_updates_changed_notes() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        let mut note = store
            .create_note("Original Title", None, &["tag1".to_string()], None)
            .unwrap();

        let db = store.db();

        db.incremental_repair(store.root()).unwrap();

        let count_before: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count_before, 1);

        note.frontmatter.title = "Updated Title".to_string();
        note.body = "Updated content".to_string();
        store.save_note(&mut note).unwrap();

        let before_sync = chrono::Utc::now().timestamp();
        std::thread::sleep(std::time::Duration::from_millis(10));

        db.incremental_repair(store.root()).unwrap();

        let title: String = db
            .conn
            .query_row(
                "SELECT title FROM notes WHERE id = ?1",
                params![note.id()],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(title, "Updated Title");

        let body: String = db
            .conn
            .query_row(
                "SELECT body FROM notes WHERE id = ?1",
                params![note.id()],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(body, "Updated content");
    }

    #[test]
    fn test_incremental_repair_removes_deleted_notes() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        let note1 = store
            .create_note("Note 1", None, &["tag1".to_string()], None)
            .unwrap();
        let note2 = store
            .create_note("Note 2", None, &["tag2".to_string()], None)
            .unwrap();

        let db = store.db();

        db.incremental_repair(store.root()).unwrap();

        let count_before: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count_before, 2);

        let note1_path = note1.path.as_ref().unwrap();
        std::fs::remove_file(note1_path).unwrap();

        db.incremental_repair(store.root()).unwrap();

        let count_after: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count_after, 1);

        let remaining_title: String = db
            .conn
            .query_row("SELECT title FROM notes", [], |row| row.get(0))
            .unwrap();
        assert_eq!(remaining_title, "Note 2");
    }

    #[test]
    fn test_incremental_repair_updates_last_sync() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        store
            .create_note("Test Note", None, &["tag".to_string()], None)
            .unwrap();

        let db = store.db();

        db.incremental_repair(store.root()).unwrap();

        let before_sync = chrono::Utc::now().timestamp();
        std::thread::sleep(std::time::Duration::from_millis(10));
        db.incremental_repair(store.root()).unwrap();

        let last_sync: String = db
            .conn
            .query_row(
                "SELECT value FROM index_meta WHERE key = 'last_sync'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let last_sync_ts: i64 = last_sync.parse().unwrap();
        assert!(last_sync_ts >= before_sync);
    }

    #[test]
    fn test_incremental_repair_handles_empty_database() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

        let db = store.db();

        db.incremental_repair(store.root()).unwrap();

        let count: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
}
