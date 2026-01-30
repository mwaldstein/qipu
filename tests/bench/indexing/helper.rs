//! Indexing performance benchmarks - helpers
//!
//! Shared code for indexing benchmark tests.

use assert_cmd::{cargo::cargo_bin_cmd, Command};
use std::path::Path;
use std::time::Instant;

/// Get a Command for qipu
pub fn qipu() -> Command {
    cargo_bin_cmd!("qipu")
}

/// Benchmark result struct
pub struct BenchmarkResult {
    pub duration_secs: f64,
    pub notes_per_sec: f64,
}

/// Create a test store with specified number of notes using qipu create command
/// This ensures notes are created in the correct format and location
pub fn create_test_store_with_notes(
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
pub fn run_index_benchmark(store_dir: &Path, force: bool) -> BenchmarkResult {
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
