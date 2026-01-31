use crate::support::{extract_id, qipu, setup_test_dir};
use predicates::prelude::*;

// ============================================================================
// Value command tests
// ============================================================================

#[test]
fn test_value_set_basic() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();

    let note_id = extract_id(&output);

    // Set value to 90
    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &note_id, "90"])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("{}: 90", note_id)));
}

#[test]
fn test_value_set_min_value() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();

    let note_id = extract_id(&output);

    // Set value to 0 (minimum)
    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &note_id, "0"])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("{}: 0", note_id)));
}

#[test]
fn test_value_set_max_value() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();

    let note_id = extract_id(&output);

    // Set value to 100 (maximum)
    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &note_id, "100"])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("{}: 100", note_id)));
}

#[test]
fn test_value_set_validation_over_100() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();

    let note_id = extract_id(&output);

    // Try to set value over 100 - should fail
    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &note_id, "101"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Value score must be between 0 and 100",
        ));
}

#[test]
fn test_value_set_updates_frontmatter() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();

    let note_id = extract_id(&output);

    // Set value
    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &note_id, "75"])
        .assert()
        .success();

    // Verify value is set via qipu show
    qipu()
        .current_dir(dir.path())
        .args(["value", "show", &note_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("75"));
}

#[test]
fn test_value_show_explicit_value() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();

    let note_id = extract_id(&output);

    // Set value
    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &note_id, "80"])
        .assert()
        .success();

    // Show value - should display explicit value without "(default)"
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

    // Show value without setting - should display default value with "(default)"
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

    // Try to show value for nonexistent note
    qipu()
        .current_dir(dir.path())
        .args(["value", "show", "qp-nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_value_set_nonexistent_note() {
    let dir = setup_test_dir();

    // Try to set value for nonexistent note
    qipu()
        .current_dir(dir.path())
        .args(["value", "set", "qp-nonexistent", "80"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_value_set_multiple_times() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();

    let note_id = extract_id(&output);

    // Set value to 70
    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &note_id, "70"])
        .assert()
        .success();

    // Update to 90
    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &note_id, "90"])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("{}: 90", note_id)));

    // Verify show displays the updated value
    qipu()
        .current_dir(dir.path())
        .args(["value", "show", &note_id])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("{}: 90", note_id)))
        .stdout(predicate::str::contains("(default)").not());
}

#[test]
fn test_value_set_json() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();

    let note_id = extract_id(&output);

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "value", "set", &note_id, "85"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["id"], note_id);
    assert_eq!(json["value"], 85);
    assert_eq!(json.as_object().unwrap().len(), 2);
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
fn test_value_set_json_error_invalid() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();

    let note_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "value", "set", &note_id, "101"])
        .assert()
        .failure();
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
