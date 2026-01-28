use crate::lib::db::*;
use crate::lib::store::Store;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_database_open_creates_tables() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

    let db = Database::open(store.root(), true).unwrap();

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
fn test_database_corrupt_auto_rebuild() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

    store
        .create_note("Test Note 1", None, &["tag1".to_string()], None)
        .unwrap();
    store
        .create_note("Test Note 2", None, &["tag2".to_string()], None)
        .unwrap();

    let db_path = store.root().join("qipu.db");

    fs::write(&db_path, b"corrupted database file that is malformed").unwrap();

    let db = Database::open(store.root(), true).unwrap();

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
fn test_database_corrupt_rebuild_preserves_note_count() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

    store
        .create_note("Test Note", None, &["tag1".to_string()], None)
        .unwrap();

    let db_path = store.root().join("qipu.db");

    fs::write(&db_path, b"database disk image is malformed").unwrap();

    let db = Database::open(store.root(), true).unwrap();

    let note_count: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM notes", [], |row: &rusqlite::Row| {
            row.get(0)
        })
        .unwrap();

    assert_eq!(note_count, 1);
}

#[test]
fn test_database_corrupt_empty_db() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

    let db_path = store.root().join("qipu.db");

    fs::write(&db_path, b"").unwrap();

    let db = Database::open(store.root(), true).unwrap();

    let note_count: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM notes", [], |row: &rusqlite::Row| {
            row.get(0)
        })
        .unwrap();

    assert_eq!(note_count, 0);
}
