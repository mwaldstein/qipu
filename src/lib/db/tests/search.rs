use crate::lib::db::*;
use crate::lib::note::NoteType;
use crate::lib::store::Store;
use tempfile::tempdir;

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

    let db = Database::open(store.root(), true).unwrap();
    db.rebuild(store.root(), None).unwrap();

    let results = db.search("test", None, None, None, None, 10).unwrap();

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

    let db = Database::open(store.root(), true).unwrap();
    db.rebuild(store.root(), None).unwrap();

    let results = db.search("test", None, None, None, None, 10).unwrap();

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

    let db = Database::open(store.root(), true).unwrap();
    db.rebuild(store.root(), None).unwrap();

    let results = db
        .search("test", Some(NoteType::Fleeting), None, None, None, 10)
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

    let db = Database::open(store.root(), true).unwrap();
    db.rebuild(store.root(), None).unwrap();

    let results = db
        .search("test", None, Some("test-tag"), None, None, 10)
        .unwrap();

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

    let db = Database::open(store.root(), true).unwrap();
    db.rebuild(store.root(), None).unwrap();

    let results = db.search("", None, None, None, None, 10).unwrap();

    assert_eq!(results.len(), 0);
}

#[test]
fn test_search_limit() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

    for i in 0..5 {
        store
            .create_note_with_content(&format!("Test Note {}", i), None, &[], "test content", None)
            .unwrap();
    }

    let db = Database::open(store.root(), true).unwrap();
    db.rebuild(store.root(), None).unwrap();

    let results = db.search("test", None, None, None, None, 3).unwrap();

    assert_eq!(results.len(), 3);
}
