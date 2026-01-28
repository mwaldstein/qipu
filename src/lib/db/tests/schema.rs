use crate::lib::store::Store;

#[test]
fn test_unknown_note_type_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::init(dir.path(), crate::lib::store::InitOptions::default()).unwrap();

    let db = store.db();

    // Insert a note with a custom type directly into database
    db.conn
        .execute(
            "INSERT INTO notes (id, title, type, path, created, updated, body, mtime, value, compacts, author, verified, source, sources, generated_by, prompt_hash, custom_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            rusqlite::params![
                "qp-unknown",
                "Unknown Type Note",
                "invalid_type",
                "unknown-type-note.md",
                "2024-01-01T00:00:00Z",
                "2024-01-01T00:00:00Z",
                "Test content",
                0,
                None::<i64>,
                "[]",
                None::<String>,
                0,
                None::<String>,
                "[]",
                None::<String>,
                None::<String>,
                "{}",
            ],
        )
        .unwrap();

    // Attempting to read note should succeed (note types are now unvalidated strings)
    let result = db.get_note("qp-unknown");
    assert!(result.is_ok());
    let note = result.unwrap().expect("Note should exist");
    assert_eq!(note.note_type().as_str(), "invalid_type");
}
