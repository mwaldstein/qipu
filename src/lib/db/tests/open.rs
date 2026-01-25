use crate::lib::db::*;
use crate::lib::store::Store;
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
