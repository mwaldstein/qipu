//! Performance benchmarks for qipu CLI
//!
//! These tests verify that qipu meets the performance budgets specified in specs/cli-tool.md:
//! - <100ms for --help and --version
//! - <200ms for list with ~1k notes
//! - <1s for search over ~10k notes

mod support;

use predicates::prelude::*;
use std::time::Instant;
use tempfile::tempdir;

use crate::support::{create_test_store_with_notes, qipu};

// ============================================================================
// Help and Version Performance Tests (<100ms)
// ============================================================================

#[test]
fn test_help_performance() {
    let iterations = 10;
    let mut total_duration = std::time::Duration::new(0, 0);

    for _ in 0..iterations {
        let start = Instant::now();
        qipu()
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("Usage: qipu"));
        total_duration += start.elapsed();
    }

    let avg_duration = total_duration / iterations;

    // Performance budget: <100ms
    assert!(
        avg_duration.as_millis() < 100,
        "Help command took {}ms, budget is <100ms",
        avg_duration.as_millis()
    );

    println!(
        "Help command average: {}ms (budget: <100ms)",
        avg_duration.as_millis()
    );
}

#[test]
fn test_version_performance() {
    let iterations = 10;
    let mut total_duration = std::time::Duration::new(0, 0);

    for _ in 0..iterations {
        let start = Instant::now();
        qipu()
            .arg("--version")
            .assert()
            .success()
            .stdout(predicate::str::contains("qipu"));
        total_duration += start.elapsed();
    }

    let avg_duration = total_duration / iterations;

    // Performance budget: <100ms
    assert!(
        avg_duration.as_millis() < 100,
        "Version command took {}ms, budget is <100ms",
        avg_duration.as_millis()
    );

    println!(
        "Version command average: {}ms (budget: <100ms)",
        avg_duration.as_millis()
    );
}

// ============================================================================
// List Performance Tests (<200ms for ~1k notes)
// ============================================================================

#[test]
fn test_list_performance_1k_notes() {
    let store_dir = tempdir().unwrap();
    let note_count = 1000;

    // Create test store with notes
    create_test_store_with_notes(store_dir.path(), note_count).unwrap();

    // Warm up
    for _ in 0..3 {
        qipu()
            .arg("--store")
            .arg(store_dir.path())
            .arg("list")
            .assert()
            .success();
    }

    // Measure performance
    let iterations = 5;
    let mut total_duration = std::time::Duration::new(0, 0);

    for _ in 0..iterations {
        let start = Instant::now();
        qipu()
            .arg("--store")
            .arg(store_dir.path())
            .arg("list")
            .assert()
            .success();
        total_duration += start.elapsed();
    }

    let avg_duration = total_duration / iterations;

    // Performance budget: <200ms
    assert!(
        avg_duration.as_millis() < 200,
        "List command took {}ms for {} notes, budget is <200ms",
        avg_duration.as_millis(),
        note_count
    );

    println!(
        "List command average: {}ms for {} notes (budget: <200ms)",
        avg_duration.as_millis(),
        note_count
    );
}

// ============================================================================
// Search Performance Tests (<1s for ~10k notes)
// ============================================================================

#[test]
#[ignore] // Long-running test: indexing 10k notes takes minutes
fn test_search_performance_10k_notes() {
    let store_dir = tempdir().unwrap();
    let note_count = 10000;

    // Create test store with notes
    create_test_store_with_notes(store_dir.path(), note_count).unwrap();

    // Warm up
    for _ in 0..2 {
        qipu()
            .arg("--store")
            .arg(store_dir.path())
            .args(["search", "programming"])
            .assert()
            .success();
    }

    // Measure performance
    let iterations = 2;
    let mut total_duration = std::time::Duration::new(0, 0);

    for _ in 0..iterations {
        let start = Instant::now();
        qipu()
            .arg("--store")
            .arg(store_dir.path())
            .args(["search", "programming"])
            .assert()
            .success();
        total_duration += start.elapsed();
    }

    let avg_duration = total_duration / iterations;

    // Per spec: search must complete in <1s for 10k notes
    let spec_budget_ms = (note_count as f64 / 10000.0 * 1000.0) as u128;

    println!(
        "Search command average: {}ms for {} notes (spec target: <{}ms)",
        avg_duration.as_millis(),
        note_count,
        spec_budget_ms
    );

    // Verify spec compliance (<1s for 10k notes)
    assert!(
        avg_duration.as_millis() < spec_budget_ms,
        "Search performance exceeds spec: took {}ms, expected <{}ms",
        avg_duration.as_millis(),
        spec_budget_ms
    );
}

// ============================================================================
// Additional Performance Validation
// ============================================================================

#[test]
fn test_verbose_timing_output() {
    let store_dir = tempdir().unwrap();
    let note_count = 100;

    // Create test store with notes
    create_test_store_with_notes(store_dir.path(), note_count).unwrap();

    // Test that verbose mode produces timing output (logs go to stderr)
    qipu()
        .arg("--store")
        .arg(store_dir.path())
        .arg("--verbose")
        .arg("list")
        .assert()
        .success()
        .stderr(predicate::str::contains("discover_store"));
}

// ============================================================================
// Scaling Performance Validation
// ============================================================================

#[test]
fn test_list_performance_scaling() {
    let note_counts = vec![100, 500, 1000];

    for count in note_counts {
        let store_dir = tempdir().unwrap();
        create_test_store_with_notes(store_dir.path(), count).unwrap();

        let start = Instant::now();
        qipu()
            .arg("--store")
            .arg(store_dir.path())
            .arg("list")
            .assert()
            .success();
        let duration = start.elapsed();

        // Performance should scale reasonably (not necessarily linear)
        let expected_max = 200 + (count as f64 / 1000.0 * 100.0) as u128; // Base + some scaling
        assert!(
            duration.as_millis() < expected_max,
            "List took {}ms for {} notes, expected <{}ms",
            duration.as_millis(),
            count,
            expected_max
        );

        println!(
            "List scaling: {}ms for {} notes",
            duration.as_millis(),
            count
        );
    }
}
