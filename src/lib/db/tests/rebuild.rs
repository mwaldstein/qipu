use crate::lib::db::*;
use crate::lib::store::Store;
use tempfile::tempdir;

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

    let db = Database::open(store.root(), true).unwrap();
    db.rebuild(store.root(), None).unwrap();

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

    let db = Database::open(store.root(), true).unwrap();
    db.rebuild(store.root(), None).unwrap();

    let initial_count: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM tags", [], |row: &rusqlite::Row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(initial_count, 1);

    note.frontmatter.tags = vec!["tag2".to_string()];
    store.save_note(&mut note).unwrap();

    db.rebuild(store.root(), None).unwrap();

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

    let db = Database::open(store.root(), true).unwrap();
    db.rebuild(store.root(), None).unwrap();

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

    let db = Database::open(store.root(), true).unwrap();
    db.rebuild(store.root(), None).unwrap();

    let note_count: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM notes", [], |row: &rusqlite::Row| {
            row.get(0)
        })
        .unwrap();

    assert_eq!(note_count, 0);
}
