use crate::support::setup_test_dir;
use crate::support::{extract_id, qipu};
use predicates::prelude::*;

#[test]
fn test_custom_get_json() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();

    let note_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &note_id, "workflow", "review"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "custom", "get", &note_id, "workflow"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["id"], note_id);
    assert_eq!(json["key"], "workflow");
    assert_eq!(json["value"], "review");
    assert_eq!(json.as_object().unwrap().len(), 3);
}

#[test]
fn test_custom_get_json_nonexistent_field() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();

    let note_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "custom", "get", &note_id, "nonexistent"])
        .assert()
        .failure();
}

#[test]
fn test_custom_get_missing_field_shows_guidance() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();
    let note_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["custom", "get", &note_id, "missing"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "custom field \"missing\" not found",
        ))
        .stderr(predicate::str::contains("qipu custom show"))
        .stderr(predicate::str::contains("qipu custom set"));
}
