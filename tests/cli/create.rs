use crate::support::{qipu, setup_test_dir};
use predicates::prelude::*;
use std::fs;

// ============================================================================
// Create command tests
// ============================================================================

#[test]
fn test_create_note() {
    let dir = setup_test_dir();

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
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "permanent", "Permanent Note"])
        .assert()
        .success();
}

#[test]
fn test_create_with_tags() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--tag", "test", "--tag", "demo", "Tagged Note"])
        .assert()
        .success();
}

#[test]
fn test_create_with_short_body_content() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Body Note", "-c", "Inline body content"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["search", "Inline body content"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Body Note"));
}

#[test]
fn test_create_with_body_flag() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Body Flag Note", "--body", "Body flag content"])
        .assert()
        .success();

    let notes_dir = dir.path().join(".qipu/notes");
    let note_files: Vec<_> = fs::read_dir(&notes_dir)
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(note_files.len(), 1);
    let content = fs::read_to_string(note_files[0].path()).unwrap();
    assert!(content.contains("Body flag content"));
}

#[test]
fn test_create_with_content_alias() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Content Alias Note", "--content", "Alias body"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["search", "Alias body"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Content Alias Note"));
}

#[test]
fn test_create_hidden_title_alias_warns() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args([
            "create",
            "--title",
            "Alias Title Note",
            "--body",
            "Alias title body",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "warning: prefer intended create syntax",
        ))
        .stderr(predicate::str::contains(
            "qipu create \"Title\" --body \"Body text\"",
        ))
        .stderr(predicate::str::contains(
            "Other basic flags: --type, --tag.",
        ))
        .stderr(predicate::str::contains(
            "Run `qipu create --help` for full and advanced details.",
        ));

    qipu()
        .current_dir(dir.path())
        .args(["search", "Alias title body"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Alias Title Note"));
}

#[test]
fn test_create_rejects_positional_and_title_alias() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Positional", "--title", "Alias"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("create received two title values"))
        .stderr(predicate::str::contains(
            "qipu create \"Title\" --body \"Body text\"",
        ))
        .stderr(predicate::str::contains(
            "Other basic flags: --type, --tag.",
        ))
        .stderr(predicate::str::contains(
            "Run `qipu create --help` for full and advanced details.",
        ));
}

#[test]
fn test_create_missing_title_shows_short_usage_guidance() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--body", "Body without title"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("create requires a title"))
        .stderr(predicate::str::contains(
            "qipu create \"Title\" --body \"Body text\"",
        ))
        .stderr(predicate::str::contains(
            "Other basic flags: --type, --tag.",
        ))
        .stderr(predicate::str::contains(
            "Run `qipu create --help` for full and advanced details.",
        ))
        .stderr(predicate::str::contains("qipu capture").not())
        .stderr(predicate::str::contains("qipu create --title").not());
}

#[test]
fn test_create_json_format() {
    let dir = setup_test_dir();

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
    let dir = setup_test_dir();

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
    let dir = setup_test_dir();

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
        .output()
        .unwrap()
        .stdout;

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
fn test_create_rejects_path_traversal_id() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Traversal", "--id", "../../outside"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid note ID"));

    assert!(
        !dir.path().join("outside-traversal.md").exists(),
        "create must not write outside the store note directory"
    );
}

#[test]
fn test_new_alias() {
    let dir = setup_test_dir();

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
    let dir = setup_test_dir();

    // Create note with custom ID
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "--id", "qp-custom123", "Custom ID Note"])
        .output()
        .unwrap()
        .stdout;

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
    let dir = setup_test_dir();

    // Test that --open flag is accepted by setting EDITOR to a no-op command
    // This verifies the flag works without actually opening an editor
    let output = qipu()
        .current_dir(dir.path())
        .env("EDITOR", "true") // 'true' is a command that always succeeds immediately
        .args(["create", "--open", "Open Flag Test"])
        .output()
        .unwrap()
        .stdout;

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
    let dir = setup_test_dir();

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
