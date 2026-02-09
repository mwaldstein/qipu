use crate::config::SearchConfig;
use crate::db::*;
use crate::note::NoteType;
use crate::store::Store;
use tempfile::tempdir;

#[test]
fn test_search_fts_basic() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

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
    db.rebuild(store.root(), None, None, None).unwrap();

    let results = db
        .search("test", None, None, None, None, 10, &SearchConfig::default())
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Test Note One");
    assert!(results[0].id.starts_with("qp-"));
}

#[test]
fn test_search_bm25_weight_configuration() {
    use crate::index::weights::{BODY_WEIGHT, TAGS_WEIGHT, TITLE_WEIGHT};

    assert_eq!(TITLE_WEIGHT, 2.0, "Title weight should match spec (2.0)");

    assert_eq!(TAGS_WEIGHT, 1.5, "Tags weight should match spec (1.5)");

    assert_eq!(
        BODY_WEIGHT, 1.0,
        "Body weight should match spec (1.0, baseline)"
    );

    const {
        assert!(TITLE_WEIGHT > TAGS_WEIGHT && TAGS_WEIGHT > BODY_WEIGHT);
    }
}

#[test]
fn test_search_with_type_filter() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

    store
        .create_note_with_content(
            "Test Note",
            Some(NoteType::from(NoteType::FLEETING)),
            &[],
            "test",
            None,
        )
        .unwrap();

    store
        .create_note_with_content(
            "Test MOC",
            Some(NoteType::from(NoteType::MOC)),
            &[],
            "test",
            None,
        )
        .unwrap();

    let db = Database::open(store.root(), true).unwrap();
    db.rebuild(store.root(), None, None, None).unwrap();

    let results = db
        .search(
            "test",
            Some(NoteType::from(NoteType::FLEETING)),
            None,
            None,
            None,
            10,
            &SearchConfig::default(),
        )
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Test Note");
}

#[test]
fn test_search_with_tag_filter() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

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
    db.rebuild(store.root(), None, None, None).unwrap();

    let results = db
        .search(
            "test",
            None,
            Some("test-tag"),
            None,
            None,
            10,
            &SearchConfig::default(),
        )
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Test Note One");
}

#[test]
fn test_search_empty_query() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

    store
        .create_note("Test Note", None, &["test-tag".to_string()], None)
        .unwrap();

    let db = Database::open(store.root(), true).unwrap();
    db.rebuild(store.root(), None, None, None).unwrap();

    let results = db
        .search("", None, None, None, None, 10, &SearchConfig::default())
        .unwrap();

    assert_eq!(results.len(), 0);
}

#[test]
fn test_search_limit() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

    for i in 0..5 {
        store
            .create_note_with_content(&format!("Test Note {}", i), None, &[], "test content", None)
            .unwrap();
    }

    let db = Database::open(store.root(), true).unwrap();
    db.rebuild(store.root(), None, None, None).unwrap();

    let results = db
        .search("test", None, None, None, None, 3, &SearchConfig::default())
        .unwrap();

    assert_eq!(results.len(), 3);
}
