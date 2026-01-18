//! SQLite database module for qipu

mod schema;

use crate::lib::error::{QipuError, Result};
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

        Ok(Database { conn })
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

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(query)];

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
}
