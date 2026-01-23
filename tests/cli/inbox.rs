use crate::cli::support::{extract_id, qipu};
use predicates::prelude::*;
use tempfile::tempdir;

// ============================================================================
// Inbox command tests
// ============================================================================

#[test]
fn test_inbox_empty() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("inbox")
        .assert()
        .success()
        .stdout(predicate::str::contains("Inbox is empty"));
}

#[test]
fn test_inbox_shows_fleeting() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "fleeting", "Inbox Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("inbox")
        .assert()
        .success()
        .stdout(predicate::str::contains("Inbox Note"));
}

#[test]
fn test_inbox_excludes_permanent() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "permanent", "Not In Inbox"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("inbox")
        .assert()
        .success()
        .stdout(predicate::str::contains("Inbox is empty"));
}

#[test]
fn test_inbox_exclude_linked() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a MOC
    let moc_output = qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "moc", "Project MOC"])
        .output()
        .unwrap();
    let moc_id = extract_id(&moc_output);

    // Create two fleeting notes
    let fleeting1_output = qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "fleeting", "Linked Note"])
        .output()
        .unwrap();
    let fleeting1_id = extract_id(&fleeting1_output);

    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "fleeting", "Unlinked Note"])
        .assert()
        .success();

    // Link the first fleeting note from the MOC
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &moc_id, &fleeting1_id, "--type", "related"])
        .assert()
        .success();

    // Build index to make sure links are tracked
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Without --exclude-linked, should show both fleeting notes
    qipu()
        .current_dir(dir.path())
        .arg("inbox")
        .assert()
        .success()
        .stdout(predicate::str::contains("Linked Note"))
        .stdout(predicate::str::contains("Unlinked Note"));

    // With --exclude-linked, should only show the unlinked note
    qipu()
        .current_dir(dir.path())
        .args(["inbox", "--exclude-linked"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Unlinked Note")
                .and(predicate::str::contains("Linked Note").not()),
        );
}

#[test]
fn test_inbox_json_format_includes_path() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "fleeting", "Test Note"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["inbox", "--format", "json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json_output = String::from_utf8(output.stdout).unwrap();

    // Verify JSON is valid
    let value: serde_json::Value = serde_json::from_str(&json_output).unwrap();

    // Verify the path field is present
    let notes = value.as_array().unwrap();
    assert_eq!(notes.len(), 1);
    let note = &notes[0];

    // Check that required fields are present per spec: id, title, type, tags, path, created, updated
    assert!(note.get("id").is_some());
    assert!(note.get("title").is_some());
    assert!(note.get("type").is_some());
    assert!(note.get("tags").is_some());
    assert!(
        note.get("path").is_some(),
        "path field should be present in JSON output"
    );
    assert!(note.get("created").is_some());
    assert!(note.get("updated").is_some());
}
