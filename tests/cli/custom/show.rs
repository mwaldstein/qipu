use crate::support::setup_test_dir;
use crate::support::{extract_id, qipu};

#[test]
fn test_custom_show_json_multiple_fields() {
    let dir = setup_test_dir();

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
    let dir = setup_test_dir();

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
fn test_custom_show_json_deterministic_ordering() {
    let dir = setup_test_dir();

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

    let custom_obj = json["custom"].as_object().unwrap();
    let keys: Vec<_> = custom_obj.keys().collect();
    assert_eq!(keys, vec!["alpha", "middle", "zebra"]);
}
