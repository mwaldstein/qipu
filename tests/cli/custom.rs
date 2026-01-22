use crate::cli::support::{extract_id, qipu};
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_custom_set_json_string() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success()
        .get_output()
        .clone();

    let note_id = extract_id(&output);

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "json",
            "custom",
            "set",
            &note_id,
            "status",
            "in-progress",
        ])
        .assert()
        .success()
        .get_output()
        .clone();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["id"], note_id);
    assert_eq!(json["key"], "status");
    assert_eq!(json["value"], "in-progress");
    assert_eq!(json.as_object().unwrap().len(), 3);

    // Verify stderr doesn't contain disclaimer in JSON format
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("Custom metadata is for applications"));
}

#[test]
fn test_custom_set_json_number() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success()
        .get_output()
        .clone();

    let note_id = extract_id(&output);

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format", "json", "custom", "set", &note_id, "priority", "5",
        ])
        .assert()
        .success()
        .get_output()
        .clone();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["id"], note_id);
    assert_eq!(json["key"], "priority");
    assert_eq!(json["value"], 5);
    assert_eq!(json.as_object().unwrap().len(), 3);
}

#[test]
fn test_custom_set_json_boolean() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success()
        .get_output()
        .clone();

    let note_id = extract_id(&output);

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format", "json", "custom", "set", &note_id, "flag", "true",
        ])
        .assert()
        .success()
        .get_output()
        .clone();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["id"], note_id);
    assert_eq!(json["key"], "flag");
    assert_eq!(json["value"], true);
    assert_eq!(json.as_object().unwrap().len(), 3);
}

#[test]
fn test_custom_set_json_array() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success()
        .get_output()
        .clone();

    let note_id = extract_id(&output);

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "json",
            "custom",
            "set",
            &note_id,
            "tags",
            "[\"a\", \"b\", \"c\"]",
        ])
        .assert()
        .success()
        .get_output()
        .clone();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["id"], note_id);
    assert_eq!(json["key"], "tags");
    assert_eq!(json["value"].as_array().unwrap().len(), 3);
    assert_eq!(json["value"][0], "a");
    assert_eq!(json["value"][1], "b");
    assert_eq!(json["value"][2], "c");
}

#[test]
fn test_custom_get_json() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success()
        .get_output()
        .clone();

    let note_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &note_id, "workflow", "review"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "custom", "get", &note_id, "workflow"])
        .assert()
        .success()
        .get_output()
        .clone();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["id"], note_id);
    assert_eq!(json["key"], "workflow");
    assert_eq!(json["value"], "review");
    assert_eq!(json.as_object().unwrap().len(), 3);
}

#[test]
fn test_custom_get_json_nonexistent_field() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success()
        .get_output()
        .clone();

    let note_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "custom", "get", &note_id, "nonexistent"])
        .assert()
        .failure();
}

#[test]
fn test_custom_show_json_multiple_fields() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success()
        .get_output()
        .clone();

    let note_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &note_id, "status", "in-progress"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &note_id, "priority", "10"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "custom", "show", &note_id])
        .assert()
        .success()
        .get_output()
        .clone();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["id"], note_id);
    assert!(json["custom"].is_object());
    assert_eq!(json["custom"]["status"], "in-progress");
    assert_eq!(json["custom"]["priority"], 10);
    assert_eq!(json.as_object().unwrap().len(), 2);
}

#[test]
fn test_custom_show_json_empty() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success()
        .get_output()
        .clone();

    let note_id = extract_id(&output);

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "custom", "show", &note_id])
        .assert()
        .success()
        .get_output()
        .clone();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["id"], note_id);
    assert!(json["custom"].is_object());
    assert_eq!(json["custom"].as_object().unwrap().len(), 0);
    assert_eq!(json.as_object().unwrap().len(), 2);
}

#[test]
fn test_custom_unset_json() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success()
        .get_output()
        .clone();

    let note_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &note_id, "temp", "value"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "custom", "unset", &note_id, "temp"])
        .assert()
        .success()
        .get_output()
        .clone();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["id"], note_id);
    assert_eq!(json["key"], "temp");
    assert_eq!(json["removed"], true);
    assert_eq!(json.as_object().unwrap().len(), 3);
}

#[test]
fn test_custom_unset_json_nonexistent_field() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success()
        .get_output()
        .clone();

    let note_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "json",
            "custom",
            "unset",
            &note_id,
            "nonexistent",
        ])
        .assert()
        .failure();
}

#[test]
fn test_custom_show_json_deterministic_ordering() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success()
        .get_output()
        .clone();

    let note_id = extract_id(&output);

    // Set fields in non-alphabetical order
    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &note_id, "zebra", "1"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &note_id, "alpha", "2"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &note_id, "middle", "3"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "custom", "show", &note_id])
        .assert()
        .success()
        .get_output()
        .clone();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Verify keys are sorted alphabetically
    let custom_obj = json["custom"].as_object().unwrap();
    let keys: Vec<_> = custom_obj.keys().collect();
    assert_eq!(keys, vec!["alpha", "middle", "zebra"]);
}

#[test]
fn test_custom_set_json_no_disclaimer_on_stderr() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success()
        .get_output()
        .clone();

    let note_id = extract_id(&output);

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format", "json", "custom", "set", &note_id, "key", "value",
        ])
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("Custom metadata is for applications"));

    // Verify human format does show disclaimer
    let output2 = qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &note_id, "key2", "value2"])
        .assert()
        .success()
        .get_output()
        .clone();

    let stderr2 = String::from_utf8_lossy(&output2.stderr);
    assert!(stderr2.contains("Custom metadata is for applications"));
}

#[test]
fn test_custom_set_negative_number() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success()
        .get_output()
        .clone();

    let note_id = extract_id(&output);

    // Test negative integer
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format", "json", "custom", "set", &note_id, "balance", "-100",
        ])
        .assert()
        .success()
        .get_output()
        .clone();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["id"], note_id);
    assert_eq!(json["key"], "balance");
    assert_eq!(json["value"], -100);

    // Verify the value is correctly stored and retrieved
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "custom", "show", &note_id])
        .assert()
        .success()
        .get_output()
        .clone();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["custom"]["balance"], -100);
}

#[test]
fn test_custom_set_leading_hyphen_strings() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success()
        .get_output()
        .clone();

    let note_id = extract_id(&output);

    // Test negative float
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "json",
            "custom",
            "set",
            &note_id,
            "temperature",
            "-3.14",
        ])
        .assert()
        .success()
        .get_output()
        .clone();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["id"], note_id);
    assert_eq!(json["key"], "temperature");
    let temp_val: f64 = json["value"].as_f64().unwrap();
    assert!((temp_val - -3.14).abs() < 0.001);

    // Test string with leading hyphen
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format", "json", "custom", "set", &note_id, "flag", "-verbose",
        ])
        .assert()
        .success()
        .get_output()
        .clone();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["id"], note_id);
    assert_eq!(json["key"], "flag");
    assert_eq!(json["value"], "-verbose");

    // Test another string with leading hyphen
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "json",
            "custom",
            "set",
            &note_id,
            "option",
            "--long-option",
        ])
        .assert()
        .success()
        .get_output()
        .clone();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["id"], note_id);
    assert_eq!(json["key"], "option");
    assert_eq!(json["value"], "--long-option");
}
