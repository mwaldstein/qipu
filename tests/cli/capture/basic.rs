use crate::support::{extract_id_from_bytes, qipu, setup_test_dir};
use predicates::prelude::*;

#[test]
fn test_capture_basic() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .arg("capture")
        .write_stdin("This is my captured note\nWith multiple lines")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));
}

#[test]
fn test_capture_with_title() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["capture", "--title", "My Custom Title"])
        .write_stdin("Content goes here")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));
}

#[test]
fn test_capture_with_type() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["capture", "--type", "literature"])
        .write_stdin("Literature note content")
        .assert()
        .success();
}

#[test]
fn test_capture_with_tags() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["capture", "--tag", "docs", "--tag", "test"])
        .write_stdin("Tagged capture content")
        .assert()
        .success();
}

#[test]
fn test_capture_json_format() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "capture", "--title", "JSON Capture"])
        .write_stdin("JSON test content")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\":"))
        .stdout(predicate::str::contains("\"title\": \"JSON Capture\""))
        .stdout(predicate::str::contains("\"type\":"));
}

#[test]
fn test_capture_records_format() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "capture", "--title", "Records Test"])
        .write_stdin("Records content")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("H qipu=1"))
        .stdout(predicate::str::contains("N qp-"));
}

#[test]
fn test_capture_default_type_fleeting() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "capture"])
        .write_stdin("Default type test")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("\"type\": \"fleeting\""));
}

#[test]
fn test_capture_content_preservation() {
    let dir = setup_test_dir();

    let content = "Line 1\nLine 2\n\nLine 4 with spacing";
    let output = qipu()
        .current_dir(dir.path())
        .args(["capture", "--title", "Content Test"])
        .write_stdin(content)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let note_id = extract_id_from_bytes(&output);

    let show_output = qipu()
        .current_dir(dir.path())
        .args(["show", &note_id])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let show_str = String::from_utf8(show_output).unwrap();
    assert!(show_str.contains("Line 1"));
    assert!(show_str.contains("Line 2"));
    assert!(show_str.contains("Line 4 with spacing"));
}

#[test]
fn test_capture_empty_content() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["capture", "--title", "Empty Note"])
        .write_stdin("")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));
}
