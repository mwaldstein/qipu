//! Indexing performance benchmarks - Phase 4: Incremental Indexing
//!
//! Phase 4 benchmarks: Incremental indexing with various change counts

use super::helper::{create_test_store_with_notes, qipu};
use std::time::Instant;
use tempfile::tempdir;

#[test]
#[ignore] // Run with: cargo test bench_incremental_indexing_10_changed_1k_notes --release -- --ignored
fn bench_incremental_indexing_10_changed_1k_notes() {
    let store_dir = tempdir().unwrap();
    let note_count = 1000;
    let changed_count = 10;

    create_test_store_with_notes(store_dir.path(), note_count).unwrap();

    // Initial full index
    run_index_benchmark(store_dir.path(), true);

    // Modify some notes by creating new content
    // We'll create new notes to simulate changes
    for i in 0..changed_count {
        let title = format!("Modified Test Note {}", i);
        qipu()
            .arg("--store")
            .arg(store_dir.path())
            .args(["create", &title])
            .assert()
            .success();
    }

    // Measure incremental indexing
    let start = Instant::now();
    qipu()
        .arg("--store")
        .arg(store_dir.path())
        .arg("index")
        .assert()
        .success();
    let duration = start.elapsed();
    let duration_secs = duration.as_secs_f64();

    // Target: Incremental index for 10 changed notes should be <2s
    let target_max_secs = 2.0;

    println!(
        "Incremental indexing for {} changed notes in 1k total: {:.2}s",
        changed_count, duration_secs
    );

    assert!(
        duration_secs < target_max_secs,
        "Incremental indexing took {:.2}s for {} changed notes, target is <{}s",
        duration_secs,
        changed_count,
        target_max_secs
    );
}

#[test]
#[ignore] // Run with: cargo test bench_incremental_indexing_100_changed_1k_notes --release -- --ignored
fn bench_incremental_indexing_100_changed_1k_notes() {
    let store_dir = tempdir().unwrap();
    let note_count = 1000;
    let changed_count = 100;

    create_test_store_with_notes(store_dir.path(), note_count).unwrap();

    // Initial full index
    run_index_benchmark(store_dir.path(), true);

    // Modify some notes
    for i in 0..changed_count {
        let title = format!("Modified Test Note {}", i);
        qipu()
            .arg("--store")
            .arg(store_dir.path())
            .args(["create", &title])
            .assert()
            .success();
    }

    // Measure incremental indexing
    let start = Instant::now();
    qipu()
        .arg("--store")
        .arg(store_dir.path())
        .arg("index")
        .assert()
        .success();
    let duration = start.elapsed();
    let duration_secs = duration.as_secs_f64();

    // Target: Incremental index for 100 changed notes should be <3s
    let target_max_secs = 3.0;

    println!(
        "Incremental indexing for {} changed notes in 1k total: {:.2}s",
        changed_count, duration_secs
    );

    assert!(
        duration_secs < target_max_secs,
        "Incremental indexing took {:.2}s for {} changed notes, target is <{}s",
        duration_secs,
        changed_count,
        target_max_secs
    );
}

#[test]
#[ignore] // Longer test, run with: cargo test bench_incremental_indexing_100_changed_10k_notes -- --ignored
fn bench_incremental_indexing_100_changed_10k_notes() {
    let store_dir = tempdir().unwrap();
    let note_count = 10000;
    let changed_count = 100;

    create_test_store_with_notes(store_dir.path(), note_count).unwrap();

    // Initial full index
    run_index_benchmark(store_dir.path(), true);

    // Modify some notes
    for i in 0..changed_count {
        let title = format!("Modified Test Note {}", i);
        qipu()
            .arg("--store")
            .arg(store_dir.path())
            .args(["create", &title])
            .assert()
            .success();
    }

    // Measure incremental indexing
    let start = Instant::now();
    qipu()
        .arg("--store")
        .arg(store_dir.path())
        .arg("index")
        .assert()
        .success();
    let duration = start.elapsed();
    let duration_secs = duration.as_secs_f64();

    // Target: Incremental index for 100 changed notes in 10k total should be <5s
    let target_max_secs = 5.0;

    println!(
        "Incremental indexing for {} changed notes in 10k total: {:.2}s",
        changed_count, duration_secs
    );

    assert!(
        duration_secs < target_max_secs,
        "Incremental indexing took {:.2}s for {} changed notes in 10k total, target is <{}s",
        duration_secs,
        changed_count,
        target_max_secs
    );
}

// Re-export helper function for use in this module
use super::helper::run_index_benchmark;
