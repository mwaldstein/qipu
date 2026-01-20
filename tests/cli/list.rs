use crate::cli::support::qipu;
use predicates::prelude::*;
use tempfile::tempdir;

// ============================================================================
// List command tests
// ============================================================================

#[test]
fn test_list_empty_store() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No notes found"));
}

#[test]
fn test_list_with_notes() {
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

    // List should show it
    qipu()
        .current_dir(dir.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Test Note"))
        .stdout(predicate::str::contains("qp-"));
}

#[test]
fn test_list_json_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "JSON List Test"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"title\": \"JSON List Test\""));
}

#[test]
fn test_list_filter_by_type() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create notes of different types
    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "fleeting", "Fleeting Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "permanent", "Permanent Note"])
        .assert()
        .success();

    // Filter by type
    qipu()
        .current_dir(dir.path())
        .args(["list", "--type", "fleeting"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Fleeting Note"))
        .stdout(predicate::str::contains("Permanent Note").not());
}

#[test]
fn test_list_filter_by_min_value_all_match() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "High Value Note"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Medium Value Note"])
        .output()
        .unwrap();
    let medium_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Low Value Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &medium_id, "75"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // All notes should match min-value 50 (default is 50, others are >= 50)
    qipu()
        .current_dir(dir.path())
        .args(["list", "--min-value", "50"])
        .assert()
        .success()
        .stdout(predicate::str::contains("High Value Note"))
        .stdout(predicate::str::contains("Medium Value Note"))
        .stdout(predicate::str::contains("Low Value Note"));
}

#[test]
fn test_list_filter_by_min_value_some_match() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "High Value Note"])
        .output()
        .unwrap();
    let high_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Medium Value Note"])
        .output()
        .unwrap();
    let medium_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Low Value Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &high_id, "90"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &medium_id, "75"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Only high and medium value notes should match min-value 70
    qipu()
        .current_dir(dir.path())
        .args(["list", "--min-value", "70"])
        .assert()
        .success()
        .stdout(predicate::str::contains("High Value Note"))
        .stdout(predicate::str::contains("Medium Value Note"))
        .stdout(predicate::str::contains("Low Value Note").not());
}

#[test]
fn test_list_filter_by_min_value_none_match() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Note 1"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 2"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output.stdout).trim().to_string();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id2, "30"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // No notes should match min-value 95 (default is 50, other is 30)
    qipu()
        .current_dir(dir.path())
        .args(["list", "--min-value", "95"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No notes found"))
        .stdout(predicate::str::contains("Note 1").not())
        .stdout(predicate::str::contains("Note 2").not());
}

#[test]
fn test_list_filter_by_min_value_with_defaults() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Explicit High Value"])
        .output()
        .unwrap();
    let high_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Default Value Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &high_id, "80"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Both should match min-value 50 (explicit 80 and default 50)
    qipu()
        .current_dir(dir.path())
        .args(["list", "--min-value", "50"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Explicit High Value"))
        .stdout(predicate::str::contains("Default Value Note"));
}
