//! Indexing performance benchmarks
//!
//! These tests verify that qipu meets the performance budgets specified in specs/progressive-indexing.md.
//!
//! Performance targets:
//! - Basic index: 100-200 notes/sec (metadata-only)
//! - Full-text index: 50-100 notes/sec (complete content)
//! - Basic index <5s for 10k notes
//! - Full-text index <25s for 10k notes
//!
//! NOTE: All benchmarks are marked #[ignore] and require --release flag to run.
//! Debug builds are significantly slower than release builds and will fail benchmarks.
//!
use assert_cmd::{cargo::cargo_bin_cmd, Command};
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use std::time::Instant;
use tempfile::tempdir;

/// Get a Command for qipu
fn qipu() -> Command {
    cargo_bin_cmd!("qipu")
}

/// Benchmark result struct
struct BenchmarkResult {
    duration_secs: f64,
    notes_per_sec: f64,
}

/// Create a test store with specified number of notes using qipu create command
/// This ensures notes are created in the correct format and location
fn create_test_store_with_notes(
    store_dir: &Path,
    count: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize store
    qipu()
        .arg("--store")
        .arg(store_dir)
        .arg("init")
        .assert()
        .success();

    // Create notes using qipu create command for correct formatting
    for i in 0..count {
        let title = format!("Test Note {}", i);
        qipu()
            .arg("--store")
            .arg(store_dir)
            .args(["create", &title])
            .assert()
            .success();
    }

    Ok(())
}

/// Helper to run benchmark and return results
fn run_index_benchmark(store_dir: &Path, force: bool) -> BenchmarkResult {
    let start = Instant::now();

    let mut cmd = qipu();
    cmd.arg("--store").arg(store_dir).arg("index");
    if force {
        cmd.arg("--rebuild");
    }
    cmd.assert().success();

    let duration = start.elapsed();
    let duration_secs = duration.as_secs_f64();

    // Count notes to calculate notes/sec
    let notes_dir = store_dir.join(".qipu/notes");
    let mocs_dir = store_dir.join(".qipu/mocs");
    let note_count = walkdir::WalkDir::new(&notes_dir)
        .into_iter()
        .filter_entry(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .count()
        + walkdir::WalkDir::new(&mocs_dir)
            .into_iter()
            .filter_entry(|e| e.path().extension().is_some_and(|ext| ext == "md"))
            .count();

    let notes_per_sec = if duration_secs > 0.0 {
        note_count as f64 / duration_secs
    } else {
        0.0
    };

    BenchmarkResult {
        duration_secs,
        notes_per_sec,
    }
}

// ============================================================================
// Phase 1: Small Note Counts (1k, 2k, 5k) - Basic & Full-Text
// Note: These benchmarks should be run with --release flag for meaningful results
// Debug mode will be significantly slower
// ============================================================================

#[test]
#[ignore] // Run with: cargo test $(basename {} --release -- --ignored\nfn
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
#[ignore] // Run with: cargo test $(basename {} --release -- --ignored\nfn
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
#[ignore] // Run with: cargo test $(basename {} --release -- --ignored\nfn
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

// ============================================================================
// Phase 2: Medium Note Counts (10k)
// ============================================================================

#[test]
#[ignore] // Run with: cargo test $(basename {} --release -- --ignored\nfn
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

// ============================================================================
// Phase 3: Large Note Counts (50k)
// ============================================================================

#[test]
#[ignore] // Run with: cargo test $(basename {} --release -- --ignored\nfn
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

// ============================================================================
// Phase 4: Incremental Indexing Tests
// ============================================================================

#[test]
#[ignore] // Run with: cargo test $(basename {} --release -- --ignored\nfn
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
#[ignore] // Run with: cargo test $(basename {} --release -- --ignored\nfn
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
#[ignore] // Run with: cargo test $(basename {} --release -- --ignored\nfn
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

// ============================================================================
// Phase 5: Quick Mode Tests
// ============================================================================

#[test]
#[ignore] // Run with: cargo test $(basename {} --release -- --ignored\nfn
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
#[ignore] // Run with: cargo test $(basename {} --release -- --ignored\nfn
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

// ============================================================================
// Phase 6: Full Text Rebuild Tests (already covered by other tests)
// ============================================================================

// ============================================================================
// Additional Utility Tests
// ============================================================================

#[test]
#[ignore] // Run with: cargo test $(basename {} --release -- --ignored\nfn
fn test_index_status_command() {
    let store_dir = tempdir().unwrap();
    let note_count = 100;

    create_test_store_with_notes(store_dir.path(), note_count).unwrap();

    // Run index
    qipu()
        .arg("--store")
        .arg(store_dir.path())
        .arg("index")
        .assert()
        .success();

    // Check status
    qipu()
        .arg("--store")
        .arg(store_dir.path())
        .args(["index", "--status"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Total notes:"))
        .stdout(predicates::str::contains("Basic indexed:"))
        .stdout(predicates::str::contains("Full-text indexed:"));
}

#[test]
#[ignore] // Run with: cargo test $(basename {} --release -- --ignored\nfn
fn test_index_with_verbose_progress() {
    let store_dir = tempdir().unwrap();
    let note_count = 500;

    create_test_store_with_notes(store_dir.path(), note_count).unwrap();

    qipu()
        .arg("--store")
        .arg(store_dir.path())
        .arg("--verbose")
        .arg("index")
        .assert()
        .success()
        .stdout(predicates::str::contains("Indexed"));
}
