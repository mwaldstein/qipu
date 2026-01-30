use crate::cli::support::{extract_id, qipu};
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_show_json_custom_omitted_by_default() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Custom Test"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Set custom metadata
    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &id, "priority", "high"])
        .assert()
        .success();

    // Show JSON without --custom should NOT include custom field
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("\"custom\""),
        "custom should be omitted by default"
    );
}

#[test]
fn test_show_json_custom_opt_in() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Custom Test"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Set custom metadata
    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &id, "priority", "high"])
        .assert()
        .success();

    // Show JSON with --custom should include custom field
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id, "--custom"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"custom\""))
        .stdout(predicate::str::contains("\"priority\""))
        .stdout(predicate::str::contains("high"));
}

#[test]
fn test_show_records_custom_opt_in() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Custom Records Test"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Set custom metadata
    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &id, "score", "42"])
        .assert()
        .success();

    // Show records with --custom should include C line
    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "show", &id, "--custom"])
        .assert()
        .success()
        .stdout(predicate::str::contains("C "))
        .stdout(predicate::str::contains("score=42"));
}
