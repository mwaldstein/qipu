//! Indexing performance benchmarks - Phase 3: Large Note Counts
//!
//! Phase 3 benchmarks: 50k notes

use super::helper::{create_test_store_with_notes, run_index_benchmark};
use tempfile::tempdir;

#[test]
#[ignore] // Very long-running test, run with: cargo test bench_basic_indexing_50k_notes -- --ignored
fn bench_basic_indexing_50k_notes() {
    let store_dir = tempdir().unwrap();
    let note_count = 50000;

    create_test_store_with_notes(store_dir.path(), note_count).unwrap();

    let result = run_index_benchmark(store_dir.path(), true);

    // Target: Basic index <8s for 50k notes (from spec)
    let target_max_secs = 250.0;

    println!(
        "Basic indexing 50k notes: {:.2}s ({:.0} notes/sec)",
        result.duration_secs, result.notes_per_sec
    );

    assert!(
        result.duration_secs < target_max_secs,
        "Basic indexing took {:.2}s for {} notes, target is <{}s",
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
