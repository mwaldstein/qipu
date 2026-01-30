use crate::support::{extract_id, qipu};
use tempfile::tempdir;

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
