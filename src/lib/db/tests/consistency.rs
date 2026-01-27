use crate::lib::db::*;
use crate::lib::store::Store;
use rusqlite::params;
use tempfile::tempdir;

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

    let db = Database::open(store.root(), true).unwrap();

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

    let db = Database::open(&dir.path().join(".qipu"), true).unwrap();

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

    let result = Database::open(store.root(), true);
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
fn test_schema_version_rollback_rebuilds() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

    store
        .create_note("Test Note", None, &["tag".to_string()], None)
        .unwrap();

    let db = store.db();

    crate::lib::db::schema::force_set_schema_version(&db.conn, 999).unwrap();

    let result = Database::open(store.root(), true);
    assert!(
        result.is_ok(),
        "Database should auto-rebuild on schema version mismatch (rollback)"
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
