//! Tests for context command metadata display
use crate::support::setup_test_dir;
use crate::support::{extract_id, qipu};
use std::fs;

#[test]
fn test_context_custom_metadata_omitted_by_default() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Note with Custom Fields"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &id, "alignment", "disagree"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &id, "priority", "5"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["context", "--note", &id])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        !stdout.contains("Custom:"),
        "Should not contain Custom section by default, got: {}",
        stdout
    );
    assert!(
        !stdout.contains("alignment"),
        "Should not contain custom field 'alignment' by default"
    );
    assert!(
        !stdout.contains("priority"),
        "Should not contain custom field 'priority' by default"
    );
}

#[test]
fn test_context_custom_metadata_with_custom_flag() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Note with Custom Fields"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &id, "alignment", "disagree"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &id, "priority", "5"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["context", "--note", &id, "--custom"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("Custom:"),
        "Should contain Custom section with --custom flag, got: {}",
        stdout
    );
    assert!(
        stdout.contains("alignment: disagree"),
        "Should contain custom field 'alignment: disagree', got: {}",
        stdout
    );
    assert!(
        stdout.contains("priority: 5"),
        "Should contain custom field 'priority: 5', got: {}",
        stdout
    );
}

#[test]
fn test_context_json_custom_metadata() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Note with Custom"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &id, "string_field", "hello"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &id, "number_field", "42"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &id, "bool_field", "true"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "context", "--note", &id, "--custom"])
        .output()
        .unwrap();

    let json_str = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    let notes = json["notes"].as_array().unwrap();
    assert_eq!(notes.len(), 1);

    let note = &notes[0];

    assert!(note["custom"].is_object(), "Should have custom object");
    assert_eq!(note["custom"]["string_field"], "hello");
    assert_eq!(note["custom"]["number_field"], 42);
    assert_eq!(note["custom"]["bool_field"], true);
}

#[test]
fn test_context_json_custom_metadata_omitted_by_default() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Note with Custom"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &id, "test_field", "value"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "context", "--note", &id])
        .output()
        .unwrap();

    let json_str = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    let notes = json["notes"].as_array().unwrap();
    assert_eq!(notes.len(), 1);

    let note = &notes[0];

    assert!(
        note.get("custom").is_none() || note["custom"].is_null(),
        "Should not have custom field without --custom flag"
    );
}

#[test]
fn test_context_records_custom_metadata() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Note with Custom"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &id, "workflow_state", "review"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &id, "priority", "3"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "context", "--note", &id, "--custom"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains(&format!("D custom.workflow_state review from={}", id)),
        "Should contain custom.workflow_state in D record, got: {}",
        stdout
    );
    assert!(
        stdout.contains(&format!("D custom.priority 3 from={}", id)),
        "Should contain custom.priority in D record, got: {}",
        stdout
    );
}

#[test]
fn test_context_records_custom_metadata_omitted_by_default() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Note with Custom"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &id, "test_field", "value"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "context", "--note", &id])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        !stdout.contains("D custom."),
        "Should not contain custom metadata without --custom flag, got: {}",
        stdout
    );
}

#[test]
fn test_context_custom_metadata_empty_custom_block() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Note without Custom"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    let output = qipu()
        .current_dir(dir.path())
        .args(["context", "--note", &id, "--custom"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        !stdout.contains("Custom:"),
        "Should not contain Custom section when note has no custom metadata, got: {}",
        stdout
    );
}

#[test]
fn test_context_custom_metadata_complex_types() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Note with Complex Custom"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    let store_path = dir.path().join(".qipu");
    let note_files: Vec<_> = fs::read_dir(store_path.join("notes"))
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();

    assert_eq!(note_files.len(), 1, "Should have exactly one note file");
    let note_path = note_files[0].path();

    let content = fs::read_to_string(&note_path).unwrap();
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    assert_eq!(
        parts.len(),
        3,
        "Note should have frontmatter with --- delimiters"
    );

    let new_content = format!(
        "---{}custom:\n  tags: [\"a\", \"b\", \"c\"]\n  nested: {{key: value}}\n---{}",
        parts[1], parts[2]
    );
    fs::write(&note_path, new_content).unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "context", "--note", &id, "--custom"])
        .output()
        .unwrap();

    let json_str = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    let notes = json["notes"].as_array().unwrap();
    assert_eq!(notes.len(), 1);

    let note = &notes[0];

    assert!(note["custom"].is_object(), "Should have custom object");
    assert!(note["custom"]["tags"].is_array(), "tags should be an array");
    assert_eq!(note["custom"]["tags"][0], "a");
    assert_eq!(note["custom"]["tags"][1], "b");
    assert_eq!(note["custom"]["tags"][2], "c");
    assert!(
        note["custom"]["nested"].is_object(),
        "nested should be an object"
    );
    assert_eq!(note["custom"]["nested"]["key"], "value");
}
