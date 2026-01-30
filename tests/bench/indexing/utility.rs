//! Indexing performance benchmarks - Phase 6: Utility Tests
//!
//! Additional utility tests for index command

use super::helper::{create_test_store_with_notes, qipu};
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
#[ignore] // Run with: cargo test test_index_status_command --release -- --ignored
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
#[ignore] // Run with: cargo test test_index_with_verbose_progress --release -- --ignored
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
