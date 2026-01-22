use super::*;
use crate::lib::graph::types::Direction;
use crate::lib::index::types::LinkSource;
use crate::lib::note::{LinkType, NoteType};
use crate::lib::store::Store;
use rusqlite::params;
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

    let results = db.search("test", None, None, None, 10).unwrap();

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

    let results = db.search("test", None, None, None, 10).unwrap();

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
        .search("test", Some(NoteType::Fleeting), None, None, 10)
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

    let results = db.search("test", None, Some("test-tag"), None, 10).unwrap();

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

    let results = db.search("", None, None, None, 10).unwrap();

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

    let db = Database::open(store.root()).unwrap();
    db.rebuild(store.root()).unwrap();

    let results = db.search("test", None, None, None, 3).unwrap();

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
fn test_schema_version_outdated_rebuilds() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

    let db = store.db();

    crate::lib::db::schema::force_set_schema_version(&db.conn, 0).unwrap();

    let result = Database::open(store.root());
    assert!(
        result.is_ok(),
        "Database should auto-rebuild on outdated schema version"
    );

    let db = result.unwrap();
    let version: String = db
        .conn
        .query_row(
            "SELECT value FROM index_meta WHERE key = 'schema_version'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        version.parse::<i32>().unwrap(),
        crate::lib::db::schema::get_schema_version()
    );
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

    let _before_sync = chrono::Utc::now().timestamp();
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
    let _note2 = store
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

#[test]
fn test_schema_version_set_on_fresh_install() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

    let db = store.db();

    let version: String = db
        .conn
        .query_row(
            "SELECT value FROM index_meta WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(version, "6");
}

#[test]
fn test_schema_version_matches_current() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

    let db = store.db();

    let version: i32 = db
        .conn
        .query_row(
            "SELECT value FROM index_meta WHERE key = 'schema_version'",
            [],
            |row| row.get::<_, String>(0).map(|s| s.parse().unwrap_or(0)),
        )
        .unwrap();
    assert_eq!(version, crate::lib::db::schema::get_schema_version());
}
