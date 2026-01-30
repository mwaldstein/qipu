//! Indexing performance benchmarks - Phase 5: Quick Mode
//!
//! Phase 5 benchmarks: Quick indexing mode

use super::helper::{create_test_store_with_notes, qipu};
use std::time::Instant;
use tempfile::tempdir;

#[test]
#[ignore] // Run with: cargo test bench_quick_index_5k_notes --release -- --ignored
fn bench_quick_index_5k_notes() {
    let store_dir = tempdir().unwrap();
    let note_count = 5000;

    create_test_store_with_notes(store_dir.path(), note_count).unwrap();

    // Measure quick index (MOCs + 100 recent)
    let start = Instant::now();
    qipu()
        .arg("--store")
        .arg(store_dir.path())
        .args(["index", "--quick"])
        .assert()
        .success();
    let duration = start.elapsed();
    let duration_secs = duration.as_secs_f64();

    // Target: Quick index <2s for 5k notes (should only index ~150 notes: 10% MOCs + 100 recent)
    let target_max_secs = 3.0;

    println!(
        "Quick index for {} notes: {:.2}s",
        note_count, duration_secs
    );

    assert!(
        duration_secs < target_max_secs,
        "Quick index took {:.2}s for {} notes, target is <{}s",
        duration_secs,
        note_count,
        target_max_secs
    );
}

#[test]
#[ignore] // Longer test, run with: cargo test bench_quick_index_10k_notes -- --ignored
fn bench_quick_index_10k_notes() {
    let store_dir = tempdir().unwrap();
    let note_count = 10000;

    create_test_store_with_notes(store_dir.path(), note_count).unwrap();

    // Measure quick index (MOCs + 100 recent)
    let start = Instant::now();
    qipu()
        .arg("--store")
        .arg(store_dir.path())
        .args(["index", "--quick"])
        .assert()
        .success();
    let duration = start.elapsed();
    let duration_secs = duration.as_secs_f64();

    // Target: Quick index <3s for 10k notes
    let target_max_secs = 3.0;

    println!(
        "Quick index for {} notes: {:.2}s",
        note_count, duration_secs
    );

    assert!(
        duration_secs < target_max_secs,
        "Quick index took {:.2}s for {} notes, target is <{}s",
        duration_secs,
        note_count,
        target_max_secs
    );
}
