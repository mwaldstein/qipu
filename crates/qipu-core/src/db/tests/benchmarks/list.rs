use crate::db::tests::benchmarks::create_test_store_with_notes;
use crate::note::NoteType;
use std::time::Instant;

#[test]
#[ignore]
fn bench_list_notes_500() {
    let store = create_test_store_with_notes(500);
    let db = store.db();

    let start = Instant::now();
    let notes = db.list_notes(None, None, None).unwrap();
    let duration = start.elapsed();

    println!(
        "List notes 500: {:?}, found {} notes",
        duration,
        notes.len()
    );

    let target_max_ms = 20.0;
    assert!(
        duration.as_millis() < target_max_ms as u128,
        "List notes took {:?} for 500 notes, target is <{}ms",
        duration,
        target_max_ms
    );

    assert_eq!(notes.len(), 500, "Should list all notes");
}

#[test]
#[ignore]
fn bench_list_notes_2000() {
    let store = create_test_store_with_notes(2000);
    let db = store.db();

    let start = Instant::now();
    let notes = db.list_notes(None, None, None).unwrap();
    let duration = start.elapsed();

    println!(
        "List notes 2000: {:?}, found {} notes",
        duration,
        notes.len()
    );

    let target_max_ms = 20.0;
    assert!(
        duration.as_millis() < target_max_ms as u128,
        "List notes took {:?} for 2000 notes, target is <{}ms",
        duration,
        target_max_ms
    );

    assert_eq!(notes.len(), 2000, "Should list all notes");
}

#[test]
#[ignore]
fn bench_list_notes_with_type_filter_2000() {
    let store = create_test_store_with_notes(2000);
    let db = store.db();

    let start = Instant::now();
    let notes = db
        .list_notes(Some(NoteType::from(NoteType::FLEETING)), None, None)
        .unwrap();
    let duration = start.elapsed();

    println!(
        "List notes with type filter 2000: {:?}, found {} notes",
        duration,
        notes.len()
    );

    let target_max_ms = 20.0;
    assert!(
        duration.as_millis() < target_max_ms as u128,
        "List notes with type filter took {:?} for 2000 notes, target is <{}ms",
        duration,
        target_max_ms
    );
}
