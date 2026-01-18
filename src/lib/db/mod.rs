//! SQLite database module for qipu

mod schema;

use crate::lib::error::{QipuError, Result};
use crate::lib::note::Note;
use crate::lib::store::Store;
use rusqlite::{params, Connection};
use std::path::Path;

pub use schema::create_schema;

/// SQLite database for qipu
#[derive(Debug)]
pub struct Database {
    conn: Connection,
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
    pub fn rebuild(&self, store: &Store) -> Result<()> {
        let notes = store.list_notes()?;

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
}

#[cfg(test)]
mod tests {
    use super::*;
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
        db.rebuild(&store).unwrap();

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
        db.rebuild(&store).unwrap();

        let initial_count: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM tags", [], |row: &rusqlite::Row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(initial_count, 1);

        note.frontmatter.tags = vec!["tag2".to_string()];
        store.save_note(&mut note).unwrap();

        db.rebuild(&store).unwrap();

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

        let note = store
            .create_note_with_content(
                "Test Note",
                None,
                &["test-tag".to_string()],
                "This is a test body with some content",
                None,
            )
            .unwrap();

        let db = Database::open(store.root()).unwrap();
        db.rebuild(&store).unwrap();

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
        db.rebuild(&store).unwrap();

        let note_count: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM notes", [], |row: &rusqlite::Row| {
                row.get(0)
            })
            .unwrap();

        assert_eq!(note_count, 0);
    }
}
