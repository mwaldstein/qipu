use crate::db::tests::benchmarks::{create_test_store_with_links, find_note_by_title};
use crate::graph::bfs_traverse;
use crate::graph::types::{Direction, HopCost, TreeOptions};
use crate::index::IndexBuilder;
use std::time::Instant;

#[test]
#[ignore]
fn bench_traverse_1_hop_200_notes() {
    let store = create_test_store_with_links(200, 3);

    let start_note_id = find_note_by_title(&store, "Test Note 0").expect("Start note should exist");

    let index = IndexBuilder::new(&store).build().unwrap();

    let opts = TreeOptions {
        direction: Direction::Out,
        max_hops: HopCost::from(1),
        max_nodes: Some(100),
        ignore_value: true,
        ..Default::default()
    };

    let start = Instant::now();
    bfs_traverse(&index, &store, &start_note_id, &opts, None, None).unwrap();
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

    let start_note_id = find_note_by_title(&store, "Test Note 0").expect("Start note should exist");

    let index = IndexBuilder::new(&store).build().unwrap();

    let opts = TreeOptions {
        direction: Direction::Out,
        max_hops: HopCost::from(3),
        max_nodes: Some(100),
        ignore_value: true,
        ..Default::default()
    };

    let start = Instant::now();
    bfs_traverse(&index, &store, &start_note_id, &opts, None, None).unwrap();
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

    let start_note_id = find_note_by_title(&store, "Test Note 0").expect("Start note should exist");

    let index = IndexBuilder::new(&store).build().unwrap();

    let opts = TreeOptions {
        direction: Direction::Out,
        max_hops: HopCost::from(3),
        max_nodes: Some(100),
        ignore_value: true,
        ..Default::default()
    };

    let start = Instant::now();
    bfs_traverse(&index, &store, &start_note_id, &opts, None, None).unwrap();
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

    let start_note_id = find_note_by_title(&store, "Test Note 0").expect("Start note should exist");

    let index = IndexBuilder::new(&store).build().unwrap();

    let opts = TreeOptions {
        direction: Direction::Out,
        max_hops: HopCost::from(3),
        max_nodes: Some(100),
        ignore_value: true,
        ..Default::default()
    };

    let start = Instant::now();
    bfs_traverse(&index, &store, &start_note_id, &opts, None, None).unwrap();
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

    let start_note_id = find_note_by_title(&store, "Test Note 0").expect("Start note should exist");

    let index = IndexBuilder::new(&store).build().unwrap();

    let opts = TreeOptions {
        direction: Direction::Both,
        max_hops: HopCost::from(3),
        max_nodes: Some(100),
        ignore_value: true,
        ..Default::default()
    };

    let start = Instant::now();
    bfs_traverse(&index, &store, &start_note_id, &opts, None, None).unwrap();
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
