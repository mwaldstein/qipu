use crate::support::{extract_id, qipu};
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_show_json_includes_value() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note with value
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Valued Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Set value
    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id, "75"])
        .assert()
        .success();

    // Show with JSON format should include value
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"value\": 75"));
}

#[test]
fn test_show_records_includes_value() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note with value
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Valued Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Set value
    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id, "50"])
        .assert()
        .success();

    // Show with records format should include value
    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("value=50"));
}
