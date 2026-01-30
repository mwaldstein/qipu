use crate::support::{qipu, setup_test_dir};
use predicates::prelude::*;
use std::fs;

#[test]
fn test_capture_with_id() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["capture", "--id", "qp-custom123", "--title", "Custom ID"])
        .write_stdin("Content with custom ID")
        .assert()
        .success()
        .stdout(predicate::str::contains("qp-custom123"));
}

#[test]
fn test_capture_verbose_output() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["--verbose", "capture", "--title", "Verbose Test"])
        .write_stdin("Verbose output test")
        .assert()
        .success()
        .stdout(predicate::str::contains("qp-"));
}

#[test]
fn test_capture_multiple_tags_json() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format", "json", "capture", "--tag", "alpha", "--tag", "beta", "--tag", "gamma",
        ])
        .write_stdin("Multi-tag test")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("\"tags\":"));
    assert!(output_str.contains("alpha"));
    assert!(output_str.contains("beta"));
    assert!(output_str.contains("gamma"));
}

#[test]
fn test_capture_records_with_tags() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args([
            "--format", "records", "capture", "--tag", "foo", "--tag", "bar",
        ])
        .write_stdin("Records tags test")
        .assert()
        .success()
        .stdout(predicate::str::contains("tags=foo,bar"));
}

#[test]
fn test_capture_records_no_tags() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "capture"])
        .write_stdin("No tags test")
        .assert()
        .success()
        .stdout(predicate::str::contains("tags=-"));
}

#[test]
fn test_capture_json_with_provenance() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "json",
            "capture",
            "--title",
            "Provenance JSON Test",
            "--source",
            "https://example.com/article",
            "--author",
            "Jane Doe",
            "--generated-by",
            "gpt-4",
            "--prompt-hash",
            "abc123",
            "--verified",
            "true",
        ])
        .write_stdin("Content with full provenance")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).unwrap();

    assert!(output_str.contains("\"source\": \"https://example.com/article\""));
    assert!(output_str.contains("\"author\": \"Jane Doe\""));
    assert!(output_str.contains("\"generated_by\": \"gpt-4\""));
    assert!(output_str.contains("\"prompt_hash\": \"abc123\""));
    assert!(output_str.contains("\"verified\": true"));
}

#[test]
fn test_capture_invalid_type() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["capture", "--type", "invalid-type"])
        .write_stdin("Test content")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Invalid note type: 'invalid-type'",
        ))
        .stderr(predicate::str::contains("Valid types:"));
}

#[test]
fn test_capture_with_custom_ontology() {
    let dir = setup_test_dir();

    let config_path = dir.path().join(".qipu/config.toml");
    let config_content = r#"
[ontology]
mode = "extended"

[ontology.note_types.my-custom-type]
description = "A custom note type"
"#;
    fs::write(&config_path, config_content).unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["capture", "--type", "my-custom-type"])
        .write_stdin("Test content with custom type")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));
}
