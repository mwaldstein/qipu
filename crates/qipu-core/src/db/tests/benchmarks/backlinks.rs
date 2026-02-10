use crate::db::tests::benchmarks::{create_test_store_with_links, find_note_by_title};
use std::time::Instant;

#[test]
#[ignore]
fn bench_backlinks_100_notes() {
    let store = create_test_store_with_links(100, 3);
    let db = store.db();

    let target_note_id =
        find_note_by_title(&store, "Test Note 50").expect("Target note should exist");

    let start = Instant::now();
    let backlinks = db.get_backlinks(&target_note_id).unwrap();
    let duration = start.elapsed();

    println!(
        "Backlinks lookup 100 notes: {:?}, found {} backlinks",
        duration,
        backlinks.len()
    );

    let target_max_ms = 10.0;
    assert!(
        duration.as_millis() < target_max_ms as u128,
        "Backlinks lookup took {:?} for 100 notes, target is <{}ms",
        duration,
        target_max_ms
    );
}

#[test]
#[ignore]
fn bench_backlinks_500_notes() {
    let store = create_test_store_with_links(500, 3);
    let db = store.db();

    let target_note_id =
        find_note_by_title(&store, "Test Note 250").expect("Target note should exist");

    let start = Instant::now();
    let backlinks = db.get_backlinks(&target_note_id).unwrap();
    let duration = start.elapsed();

    println!(
        "Backlinks lookup 500 notes: {:?}, found {} backlinks",
        duration,
        backlinks.len()
    );

    let target_max_ms = 10.0;
    assert!(
        duration.as_millis() < target_max_ms as u128,
        "Backlinks lookup took {:?} for 500 notes, target is <{}ms",
        duration,
        target_max_ms
    );
}

#[test]
#[ignore]
fn bench_backlinks_2000_notes() {
    let store = create_test_store_with_links(2000, 3);
    let db = store.db();

    let target_note_id =
        find_note_by_title(&store, "Test Note 1000").expect("Target note should exist");

    let start = Instant::now();
    let backlinks = db.get_backlinks(&target_note_id).unwrap();
    let duration = start.elapsed();

    println!(
        "Backlinks lookup 2000 notes: {:?}, found {} backlinks",
        duration,
        backlinks.len()
    );

    let target_max_ms = 10.0;
    assert!(
        duration.as_millis() < target_max_ms as u128,
        "Backlinks lookup took {:?} for 2000 notes, target is <{}ms",
        duration,
        target_max_ms
    );
}
