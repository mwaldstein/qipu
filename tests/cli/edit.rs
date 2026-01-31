use crate::support::{extract_id, qipu, setup_test_dir};
use predicates::prelude::*;

#[test]
fn test_edit_note_by_id() {
    let dir = setup_test_dir();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Edit the note (using echo as a fake editor that succeeds)
    qipu()
        .current_dir(dir.path())
        .args(["edit", "--editor", "true", &id])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));
}

#[test]
fn test_edit_note_by_path() {
    let dir = setup_test_dir();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Edit the note using relative path (using echo as a fake editor)
    // Filename format: {id}-{slug(title)}.md
    qipu()
        .current_dir(dir.path())
        .args([
            "edit",
            "--editor",
            "true",
            &format!(".qipu/notes/{}-test-note.md", id),
        ])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));
}

#[test]
fn test_edit_fails_without_editor() {
    let dir = setup_test_dir();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Unset EDITOR and VISUAL environment variables
    qipu()
        .current_dir(dir.path())
        .env("EDITOR", "")
        .env("VISUAL", "")
        .args(["edit", &id])
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to open editor"));
}

#[test]
fn test_edit_with_editor_override() {
    let dir = setup_test_dir();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Edit with explicit editor override (ignoring env vars)
    qipu()
        .current_dir(dir.path())
        .env("EDITOR", "nonexistent-editor")
        .args(["edit", "--editor", "true", &id])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));
}

#[test]
fn test_edit_json_format() {
    let dir = setup_test_dir();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Edit and output in JSON format
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "edit", "--editor", "true", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\""))
        .stdout(predicate::str::contains("\"title\""))
        .stdout(predicate::str::contains("\"path\""));
}

#[test]
fn test_edit_records_format() {
    let dir = setup_test_dir();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Edit and output in records format
    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "edit", "--editor", "true", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("N id="))
        .stdout(predicate::str::contains("path="));
}
