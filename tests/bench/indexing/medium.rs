//! Indexing performance benchmarks - Phase 2: Medium Note Counts
//!
//! Phase 2 benchmarks: 10k notes

use super::helper::{create_test_store_with_notes, run_index_benchmark};
use tempfile::tempdir;

#[test]
#[ignore] // Long-running test, run with: cargo test bench_full_text_indexing_10k_notes -- --ignored
fn bench_full_text_indexing_10k_notes() {
    let store_dir = tempdir().unwrap();
    let note_count = 10000;

    create_test_store_with_notes(store_dir.path(), note_count).unwrap();

    let result = run_index_benchmark(store_dir.path(), true);

    // Target: Full-text index <25s for 10k notes (from spec)
    let target_max_secs = 25.0;

    println!(
        "Full-text indexing 10k notes: {:.2}s ({:.0} notes/sec)",
        result.duration_secs, result.notes_per_sec
    );

    assert!(
        result.duration_secs < target_max_secs,
        "Full-text indexing took {:.2}s for {} notes, target is <{}s",
        result.duration_secs,
        note_count,
        target_max_secs
    );

    // Full-text should be at least 400 notes/sec (10k/25s)
    let target_min_notes_per_sec = 400.0;
    assert!(
        result.notes_per_sec >= target_min_notes_per_sec,
        "Notes/sec {:.0} is below target of {:.0}",
        result.notes_per_sec,
        target_min_notes_per_sec
    );
}
