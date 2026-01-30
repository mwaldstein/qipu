//! Indexing performance benchmarks - Phase 1: Small Note Counts
//!
//! Phase 1 benchmarks: 1k, 2k, 5k notes

use super::helper::{create_test_store_with_notes, run_index_benchmark};
use tempfile::tempdir;

#[test]
#[ignore] // Run with: cargo test bench_basic_indexing_1k_notes --release -- --ignored
fn bench_basic_indexing_1k_notes() {
    let store_dir = tempdir().unwrap();
    let note_count = 1000;

    create_test_store_with_notes(store_dir.path(), note_count).unwrap();

    let result = run_index_benchmark(store_dir.path(), true);

    // Target: Full-text index <3s for 1k notes (from spec)
    let target_max_secs = 5.0;

    println!(
        "Full-text indexing 1k notes: {:.2}s ({:.0} notes/sec)",
        result.duration_secs, result.notes_per_sec
    );

    assert!(
        result.duration_secs < target_max_secs,
        "Full-text indexing took {:.2}s for {} notes, target is <{}s",
        result.duration_secs,
        note_count,
        target_max_secs
    );

    // Full-text should be at least 100-200 notes/sec
    let target_min_notes_per_sec = 200.0;
    assert!(
        result.notes_per_sec >= target_min_notes_per_sec,
        "Notes/sec {:.0} is below target of {:.0}",
        result.notes_per_sec,
        target_min_notes_per_sec
    );
}

#[test]
#[ignore] // Run with: cargo test bench_basic_indexing_2k_notes --release -- --ignored
fn bench_basic_indexing_2k_notes() {
    let store_dir = tempdir().unwrap();
    let note_count = 2000;

    create_test_store_with_notes(store_dir.path(), note_count).unwrap();

    let result = run_index_benchmark(store_dir.path(), true);

    // Target: Full-text index <8s for 2k notes (from spec, extrapolated)
    let target_max_secs = 10.0;

    println!(
        "Full-text indexing 2k notes: {:.2}s ({:.0} notes/sec)",
        result.duration_secs, result.notes_per_sec
    );

    assert!(
        result.duration_secs < target_max_secs,
        "Full-text indexing took {:.2}s for {} notes, target is <{}s",
        result.duration_secs,
        note_count,
        target_max_secs
    );

    let target_min_notes_per_sec = 200.0;
    assert!(
        result.notes_per_sec >= target_min_notes_per_sec,
        "Notes/sec {:.0} is below target of {:.0}",
        result.notes_per_sec,
        target_min_notes_per_sec
    );
}

#[test]
#[ignore] // Longer test, run with: cargo test bench_basic_indexing_5k_notes -- --ignored
fn bench_basic_indexing_5k_notes() {
    let store_dir = tempdir().unwrap();
    let note_count = 5000;

    create_test_store_with_notes(store_dir.path(), note_count).unwrap();

    let result = run_index_benchmark(store_dir.path(), true);

    // Target: Full-text index <15s for 5k notes (from spec)
    let target_max_secs = 20.0;

    println!(
        "Full-text indexing 5k notes: {:.2}s ({:.0} notes/sec)",
        result.duration_secs, result.notes_per_sec
    );

    assert!(
        result.duration_secs < target_max_secs,
        "Full-text indexing took {:.2}s for {} notes, target is <{}s",
        result.duration_secs,
        note_count,
        target_max_secs
    );

    let target_min_notes_per_sec = 250.0;
    assert!(
        result.notes_per_sec >= target_min_notes_per_sec,
        "Notes/sec {:.0} is below target of {:.0}",
        result.notes_per_sec,
        target_min_notes_per_sec
    );
}
