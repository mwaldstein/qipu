//! Database operation performance benchmarks
//!
//! These tests verify that qipu meets the performance budgets specified in specs/operational-database.md.
//!
//! Performance targets (2000 notes):
//! - Search: <50ms
//! - Backlink lookup: <10ms
//! - Graph traversal (3 hops): <100ms
//!
//! NOTE: All benchmarks are marked #[ignore] and require --release flag to run.
//! Debug builds are significantly slower than release builds and will fail benchmarks.

use crate::lib::config::SearchConfig;
use crate::lib::note::{NoteType, TypedLink};
use crate::lib::store::InitOptions;
use crate::lib::store::Store;
use std::time::Instant;
use tempfile::tempdir;

use crate::lib::graph::types::Direction;

/// Helper to create a test store with specified number of notes
fn create_test_store_with_notes(count: usize) -> Store {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), InitOptions::default()).unwrap();

    for i in 0..count {
        let title = format!("Test Note {}", i);
        store.create_note(&title, None, &[], None).unwrap();
    }

    store
}

/// Helper to create a test store with linked notes for traversal tests
fn create_test_store_with_links(note_count: usize, links_per_note: usize) -> Store {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), InitOptions::default()).unwrap();

    let mut note_ids = Vec::new();

    for i in 0..note_count {
        let title = format!("Test Note {}", i);
        let note = store.create_note(&title, None, &[], None).unwrap();
        note_ids.push(note.id().to_string());
    }

    for (i, note_id) in note_ids.iter().enumerate() {
        let mut note = store.get_note(note_id).unwrap();

        for j in 0..links_per_note {
            let target_index = (i + j + 1) % note_ids.len();
            note.frontmatter.links.push(TypedLink {
                link_type: crate::lib::note::LinkType::from("related"),
                id: note_ids[target_index].clone(),
            });
        }

        store.save_note(&mut note).unwrap();
    }

    store
}

/// Helper to extract note ID from title
fn find_note_by_title(store: &Store, title_pattern: &str) -> Option<String> {
    store
        .db()
        .list_notes(None, None, None)
        .unwrap()
        .iter()
        .find(|n| n.title.contains(title_pattern))
        .map(|n| n.id.to_string())
}

// ============================================================================
// Search Performance Benchmarks
// ============================================================================

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
            Some(NoteType::Fleeting),
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

// ============================================================================
// Backlink Lookup Performance Benchmarks
// ============================================================================

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

// ============================================================================
// Graph Traversal Performance Benchmarks
// ============================================================================

#[test]
#[ignore]
fn bench_traverse_1_hop_200_notes() {
    let store = create_test_store_with_links(200, 3);
    let db = store.db();

    let start_note_id = find_note_by_title(&store, "Test Note 0").expect("Start note should exist");

    let start = Instant::now();
    let _results = db
        .traverse(&start_note_id, Direction::Out, 1, Some(100))
        .unwrap();
    let duration = start.elapsed();

    println!("Traversal 1 hop 200 notes: {:?}", duration);

    let target_max_ms = 100.0;
    assert!(
        duration.as_millis() < target_max_ms as u128,
        "Traversal 1 hop took {:?} for 200 notes, target is <{}ms",
        duration,
        target_max_ms
    );
}

#[test]
#[ignore]
fn bench_traverse_3_hops_200_notes() {
    let store = create_test_store_with_links(200, 3);
    let db = store.db();

    let start_note_id = find_note_by_title(&store, "Test Note 0").expect("Start note should exist");

    let start = Instant::now();
    let _results = db
        .traverse(&start_note_id, Direction::Out, 3, Some(100))
        .unwrap();
    let duration = start.elapsed();

    println!("Traversal 3 hops 200 notes: {:?}", duration);

    let target_max_ms = 100.0;
    assert!(
        duration.as_millis() < target_max_ms as u128,
        "Traversal 3 hops took {:?} for 200 notes, target is <{}ms",
        duration,
        target_max_ms
    );
}

#[test]
#[ignore]
fn bench_traverse_3_hops_500_notes() {
    let store = create_test_store_with_links(500, 3);
    let db = store.db();

    let start_note_id = find_note_by_title(&store, "Test Note 0").expect("Start note should exist");

    let start = Instant::now();
    let _results = db
        .traverse(&start_note_id, Direction::Out, 3, Some(100))
        .unwrap();
    let duration = start.elapsed();

    println!("Traversal 3 hops 500 notes: {:?}", duration);

    let target_max_ms = 100.0;
    assert!(
        duration.as_millis() < target_max_ms as u128,
        "Traversal 3 hops took {:?} for 500 notes, target is <{}ms",
        duration,
        target_max_ms
    );
}

#[test]
#[ignore]
fn bench_traverse_3_hops_2000_notes() {
    let store = create_test_store_with_links(2000, 3);
    let db = store.db();

    let start_note_id = find_note_by_title(&store, "Test Note 0").expect("Start note should exist");

    let start = Instant::now();
    let _results = db
        .traverse(&start_note_id, Direction::Out, 3, Some(100))
        .unwrap();
    let duration = start.elapsed();

    println!("Traversal 3 hops 2000 notes: {:?}", duration);

    let target_max_ms = 100.0;
    assert!(
        duration.as_millis() < target_max_ms as u128,
        "Traversal 3 hops took {:?} for 2000 notes, target is <{}ms",
        duration,
        target_max_ms
    );
}

#[test]
#[ignore]
fn bench_traverse_both_directions_3_hops_500_notes() {
    let store = create_test_store_with_links(500, 3);
    let db = store.db();

    let start_note_id = find_note_by_title(&store, "Test Note 0").expect("Start note should exist");

    let start = Instant::now();
    let _results = db
        .traverse(&start_note_id, Direction::Both, 3, Some(100))
        .unwrap();
    let duration = start.elapsed();

    println!("Traversal both directions 3 hops 500 notes: {:?}", duration);

    let target_max_ms = 100.0;
    assert!(
        duration.as_millis() < target_max_ms as u128,
        "Traversal both directions took {:?} for 500 notes, target is <{}ms",
        duration,
        target_max_ms
    );
}
