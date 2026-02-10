use crate::config::SearchConfig;
use crate::db::tests::benchmarks::create_test_store_with_notes;
use crate::note::NoteType;
use std::time::Instant;

#[test]
#[ignore]
fn bench_search_500_notes() {
    let store = create_test_store_with_notes(500);
    let db = store.db();

    let start = Instant::now();
    let results = db
        .search(
            "test",
            None,
            None,
            None,
            None,
            100,
            &SearchConfig::default(),
        )
        .unwrap();
    let duration = start.elapsed();

    println!("Search 500 notes: {:?}", duration);

    let target_max_ms = 50.0;
    assert!(
        duration.as_millis() < target_max_ms as u128,
        "Search took {:?} for 500 notes, target is <{}ms",
        duration,
        target_max_ms
    );

    assert!(!results.is_empty(), "Search should return results");
}

#[test]
#[ignore]
fn bench_search_2000_notes() {
    let store = create_test_store_with_notes(2000);
    let db = store.db();

    let start = Instant::now();
    let results = db
        .search(
            "test",
            None,
            None,
            None,
            None,
            100,
            &SearchConfig::default(),
        )
        .unwrap();
    let duration = start.elapsed();

    println!("Search 2000 notes: {:?}", duration);

    let target_max_ms = 50.0;
    assert!(
        duration.as_millis() < target_max_ms as u128,
        "Search took {:?} for 2000 notes, target is <{}ms",
        duration,
        target_max_ms
    );

    assert!(!results.is_empty(), "Search should return results");
}

#[test]
#[ignore]
fn bench_search_with_filters_2000_notes() {
    let store = create_test_store_with_notes(2000);
    let db = store.db();

    let start = Instant::now();
    let _results = db
        .search(
            "test",
            Some(NoteType::from(NoteType::FLEETING)),
            None,
            None,
            None,
            100,
            &SearchConfig::default(),
        )
        .unwrap();
    let duration = start.elapsed();

    println!("Search with filters 2000 notes: {:?}", duration);

    let target_max_ms = 50.0;
    assert!(
        duration.as_millis() < target_max_ms as u128,
        "Search with filters took {:?} for 2000 notes, target is <{}ms",
        duration,
        target_max_ms
    );
}
