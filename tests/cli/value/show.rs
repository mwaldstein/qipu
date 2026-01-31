use crate::support::{extract_id, qipu, setup_test_dir};
use predicates::prelude::*;

#[test]
fn test_value_show_explicit_value() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();

    let note_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &note_id, "80"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "show", &note_id])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("{}: 80", note_id)))
        .stdout(predicate::str::contains("(default)").not());
}

#[test]
fn test_value_show_default_value() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();

    let note_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["value", "show", &note_id])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "{}: 50 (default)",
            note_id
        )));
}

#[test]
fn test_value_show_nonexistent_note() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["value", "show", "qp-nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_value_show_json_explicit() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();

    let note_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &note_id, "75"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "value", "show", &note_id])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["id"], note_id);
    assert_eq!(json["value"], 75);
    assert_eq!(json["default"], false);
    assert_eq!(json.as_object().unwrap().len(), 3);
}

#[test]
fn test_value_show_json_default() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();

    let note_id = extract_id(&output);

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "value", "show", &note_id])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["id"], note_id);
    assert_eq!(json["value"], 50);
    assert_eq!(json["default"], true);
    assert_eq!(json.as_object().unwrap().len(), 3);
}

#[test]
fn test_value_show_json_nonexistent() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "value", "show", "qp-nonexistent"])
        .assert()
        .failure();
}
