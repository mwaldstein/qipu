use crate::support::setup_test_dir;
use crate::support::{extract_id, qipu};

#[test]
fn test_custom_set_json_string() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();

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
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["id"], note_id);
    assert_eq!(json["key"], "status");
    assert_eq!(json["value"], "in-progress");
    assert_eq!(json.as_object().unwrap().len(), 3);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("Custom metadata is for applications"));
}

#[test]
fn test_custom_set_json_number() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();

    let note_id = extract_id(&output);

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format", "json", "custom", "set", &note_id, "priority", "5",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["id"], note_id);
    assert_eq!(json["key"], "priority");
    assert_eq!(json["value"], 5);
    assert_eq!(json.as_object().unwrap().len(), 3);
}

#[test]
fn test_custom_set_json_boolean() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();

    let note_id = extract_id(&output);

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format", "json", "custom", "set", &note_id, "flag", "true",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["id"], note_id);
    assert_eq!(json["key"], "flag");
    assert_eq!(json["value"], true);
    assert_eq!(json.as_object().unwrap().len(), 3);
}

#[test]
fn test_custom_set_json_array() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();

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
        .output()
        .unwrap();

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
fn test_custom_set_json_no_disclaimer_on_stderr() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();

    let note_id = extract_id(&output);

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format", "json", "custom", "set", &note_id, "key", "value",
        ])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("Custom metadata is for applications"));

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &note_id, "key2", "value2"])
        .output()
        .unwrap();

    let stderr2 = String::from_utf8_lossy(&output2.stderr);
    assert!(stderr2.contains("Custom metadata is for applications"));
}

#[test]
fn test_custom_set_negative_number() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();

    let note_id = extract_id(&output);

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format", "json", "custom", "set", &note_id, "balance", "-100",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["id"], note_id);
    assert_eq!(json["key"], "balance");
    assert_eq!(json["value"], -100);

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "custom", "show", &note_id])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["custom"]["balance"], -100);
}

#[test]
fn test_custom_set_leading_hyphen_strings() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();

    let note_id = extract_id(&output);

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "json",
            "custom",
            "set",
            &note_id,
            "temperature",
            "-2.75",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["id"], note_id);
    assert_eq!(json["key"], "temperature");
    let temp_val: f64 = json["value"].as_f64().unwrap();
    assert!((temp_val - -2.75).abs() < 0.001);

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format", "json", "custom", "set", &note_id, "flag", "-verbose",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["id"], note_id);
    assert_eq!(json["key"], "flag");
    assert_eq!(json["value"], "-verbose");

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
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["id"], note_id);
    assert_eq!(json["key"], "option");
    assert_eq!(json["value"], "--long-option");
}
