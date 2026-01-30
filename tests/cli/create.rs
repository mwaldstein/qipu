use crate::support::qipu;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

// ============================================================================
// Create command tests
// ============================================================================

#[test]
fn test_create_note() {
    let dir = tempdir().unwrap();

    // Init first
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create note
    qipu()
        .current_dir(dir.path())
        .args(["create", "My Test Note"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));
}

#[test]
fn test_create_with_type() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "permanent", "Permanent Note"])
        .assert()
        .success();
}

#[test]
fn test_create_with_tags() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--tag", "test", "--tag", "demo", "Tagged Note"])
        .assert()
        .success();
}

#[test]
fn test_create_json_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "create", "JSON Note"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\":"))
        .stdout(predicate::str::contains("\"title\": \"JSON Note\""));
}

#[test]
fn test_create_prompt_hash_in_frontmatter() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create note with --prompt-hash
    qipu()
        .current_dir(dir.path())
        .args([
            "create",
            "--prompt-hash",
            "test-hash-123",
            "Frontmatter Test",
        ])
        .assert()
        .success();

    // Find and read the markdown file
    let notes_dir = dir.path().join(".qipu/notes");
    let note_files: Vec<_> = fs::read_dir(&notes_dir)
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(note_files.len(), 1);
    let note_path = note_files[0].path();
    let content = fs::read_to_string(&note_path).unwrap();

    // Verify prompt_hash appears in frontmatter
    assert!(
        content.contains("prompt_hash: test-hash-123"),
        "frontmatter should contain prompt_hash"
    );
}

#[test]
fn test_create_json_with_provenance() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "json",
            "create",
            "--source",
            "https://example.com",
            "--author",
            "TestBot",
            "--generated-by",
            "gpt-4o",
            "--prompt-hash",
            "abc123",
            "--verified",
            "false",
            "Provenance Test",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Verify provenance fields are present
    assert_eq!(json["source"], "https://example.com");
    assert_eq!(json["author"], "TestBot");
    assert_eq!(json["generated_by"], "gpt-4o");
    assert_eq!(json["prompt_hash"], "abc123");
    assert_eq!(json["verified"], false);

    // Verify standard fields are also present
    assert!(json["id"].as_str().unwrap().starts_with("qp-"));
    assert_eq!(json["title"], "Provenance Test");
}

#[test]
fn test_new_alias() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Test that 'new' works as an alias for 'create'
    qipu()
        .current_dir(dir.path())
        .args(["new", "Note via New Command"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));
}

#[test]
fn test_create_with_custom_id() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create note with custom ID
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "--id", "qp-custom123", "Custom ID Note"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).unwrap();
    assert!(
        output_str.contains("qp-custom123"),
        "Output should contain custom ID"
    );

    // Verify the note was created with the specified ID by reading directory
    let notes_dir = dir.path().join(".qipu/notes");
    let entries: Vec<_> = fs::read_dir(&notes_dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    assert!(
        entries.iter().any(|name| name.contains("qp-custom123")),
        "Note file with custom ID should exist. Found files: {:?}",
        entries
    );
}

#[test]
fn test_create_with_open_flag() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Test that --open flag is accepted by setting EDITOR to a no-op command
    // This verifies the flag works without actually opening an editor
    let output = qipu()
        .current_dir(dir.path())
        .env("EDITOR", "true") // 'true' is a command that always succeeds immediately
        .args(["create", "--open", "Open Flag Test"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).unwrap();
    assert!(
        output_str.starts_with("qp-"),
        "Output should contain note ID"
    );

    // Verify the note was created
    let notes_dir = dir.path().join(".qipu/notes");
    let entries = fs::read_dir(&notes_dir)
        .unwrap()
        .filter_map(Result::ok)
        .count();
    assert_eq!(entries, 1, "Exactly one note should be created");
}

#[test]
fn test_create_invalid_type() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create with invalid note type should fail
    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "invalid-type", "Test Note"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Invalid note type: 'invalid-type'",
        ))
        .stderr(predicate::str::contains("Valid types:"));
}
