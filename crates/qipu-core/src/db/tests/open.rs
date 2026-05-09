use crate::db::*;
use crate::store::Store;
use rusqlite::Connection;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_database_open_creates_tables() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

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
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

    store
        .create_note("Test Note 1", None, &["tag1".to_string()], None)
        .unwrap();
    store
        .create_note("Test Note 2", None, &["tag2".to_string()], None)
        .unwrap();

    let store_root = store.root().to_path_buf();
    let db_path = store_root.join("qipu.db");
    drop(store);

    fs::write(&db_path, b"corrupted database file that is malformed").unwrap();

    let db = Database::open(&store_root, true).unwrap();

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
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

    store
        .create_note("Test Note", None, &["tag1".to_string()], None)
        .unwrap();

    let store_root = store.root().to_path_buf();
    let db_path = store_root.join("qipu.db");
    drop(store);

    fs::write(&db_path, b"database disk image is malformed").unwrap();

    let db = Database::open(&store_root, true).unwrap();

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
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

    let store_root = store.root().to_path_buf();
    let db_path = store_root.join("qipu.db");
    drop(store);

    fs::write(&db_path, b"").unwrap();

    let db = Database::open(&store_root, true).unwrap();

    let note_count: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM notes", [], |row: &rusqlite::Row| {
            row.get(0)
        })
        .unwrap();

    assert_eq!(note_count, 0);
}

#[test]
fn test_database_locked_does_not_trigger_rebuild() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

    store
        .create_note("Locked DB Note", None, &["tag1".to_string()], None)
        .unwrap();

    let store_root = store.root().to_path_buf();
    let db_path = store_root.join("qipu.db");
    let original_size = fs::metadata(&db_path).unwrap().len();
    drop(store);

    let lock_conn = Connection::open(&db_path).unwrap();
    lock_conn
        .pragma_update(None, "locking_mode", "EXCLUSIVE")
        .unwrap();
    lock_conn.execute_batch("BEGIN EXCLUSIVE;").unwrap();

    let result = Database::open(&store_root, true);
    assert!(result.is_err());
    assert!(db_path.exists(), "locked database must not be deleted");
    assert_eq!(
        fs::metadata(&db_path).unwrap().len(),
        original_size,
        "locked database must not be replaced"
    );

    lock_conn.execute_batch("ROLLBACK;").unwrap();
    drop(lock_conn);

    let db = Database::open(&store_root, true).unwrap();
    let note_count: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
        .unwrap();

    assert_eq!(note_count, 1);
}

#[test]
fn test_open_with_auto_repair_triggers_incremental_repair() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

    let note = store
        .create_note("Original Title", None, &["tag1".to_string()], None)
        .unwrap();

    let note_path = note.path.as_ref().unwrap();

    let note_content = fs::read_to_string(note_path).unwrap();
    let updated_content = note_content.replace("Original Title", "Updated Title");

    fs::write(note_path, updated_content).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(10));

    let db = Database::open(store.root(), true).unwrap();

    let title: String = db
        .conn
        .query_row(
            "SELECT title FROM notes WHERE id = ?1",
            [note.id()],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(title, "Updated Title");
}

#[test]
fn test_open_without_auto_repair_does_not_trigger_repair() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

    let note = store
        .create_note("Original Title", None, &["tag1".to_string()], None)
        .unwrap();

    let note_path = note.path.as_ref().unwrap();

    let note_content = fs::read_to_string(note_path).unwrap();
    let updated_content = note_content.replace("Original Title", "Updated Title");

    fs::write(note_path, updated_content).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(10));

    let db = Database::open(store.root(), false).unwrap();

    let title: String = db
        .conn
        .query_row(
            "SELECT title FROM notes WHERE id = ?1",
            [note.id()],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(title, "Original Title");
}
