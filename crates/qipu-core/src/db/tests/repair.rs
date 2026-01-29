use crate::store::Store;
use rusqlite::params;
use tempfile::tempdir;

#[test]
fn test_incremental_repair_updates_changed_notes() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

    let mut note = store
        .create_note("Original Title", None, &["tag1".to_string()], None)
        .unwrap();

    let db = store.db();

    db.incremental_repair(store.root(), None).unwrap();

    let count_before: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count_before, 1);

    note.frontmatter.title = "Updated Title".to_string();
    note.body = "Updated content".to_string();
    store.save_note(&mut note).unwrap();

    let _before_sync = chrono::Utc::now().timestamp();
    std::thread::sleep(std::time::Duration::from_millis(10));

    db.incremental_repair(store.root(), None).unwrap();

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
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

    let note1 = store
        .create_note("Note 1", None, &["tag1".to_string()], None)
        .unwrap();
    let _note2 = store
        .create_note("Note 2", None, &["tag2".to_string()], None)
        .unwrap();

    let db = store.db();

    db.incremental_repair(store.root(), None).unwrap();

    let count_before: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count_before, 2);

    let note1_path = note1.path.as_ref().unwrap();
    std::fs::remove_file(note1_path).unwrap();

    db.incremental_repair(store.root(), None).unwrap();

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
fn test_incremental_repair_skips_unchanged_notes() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

    let note = store
        .create_note("Test Note", None, &["tag".to_string()], None)
        .unwrap();

    let db = store.db();

    db.incremental_repair(store.root(), None).unwrap();

    let mtime_after_first: i64 = db
        .conn
        .query_row(
            "SELECT mtime FROM notes WHERE id = ?1",
            params![note.id()],
            |row| row.get(0),
        )
        .unwrap();

    std::thread::sleep(std::time::Duration::from_millis(10));

    db.incremental_repair(store.root(), None).unwrap();

    let mtime_after_second: i64 = db
        .conn
        .query_row(
            "SELECT mtime FROM notes WHERE id = ?1",
            params![note.id()],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(mtime_after_first, mtime_after_second);
}

#[test]
fn test_incremental_repair_handles_empty_database() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

    let db = store.db();

    db.incremental_repair(store.root(), None).unwrap();

    let count: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 0);
}
