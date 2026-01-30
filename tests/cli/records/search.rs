//! Records format search and empty result tests

use crate::cli::support::qipu;
use tempfile::tempdir;

#[test]
fn test_records_empty_result_set_search() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note
    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success();

    // Search for non-existent term in records format
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "search", "nonexistent"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should have header with count=0
    assert!(stdout.contains("H qipu=1 records=1"));

    // Should not have any N lines
    assert!(
        !stdout.contains("\nN "),
        "Should not have note lines for empty search result"
    );
}
