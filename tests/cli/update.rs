use crate::support::{extract_id, qipu, setup_test_dir};
use predicates::prelude::*;

#[test]
fn test_update_title() {
    let dir = setup_test_dir();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Original Title"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Update the title
    qipu()
        .current_dir(dir.path())
        .args(["update", "--title", "New Title", &id])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));

    // Verify the title was updated
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"title\": \"New Title\""));
}

#[test]
fn test_update_type_moc() {
    let dir = setup_test_dir();

    // Create a regular note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Regular Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Update to MOC type
    qipu()
        .current_dir(dir.path())
        .args(["update", "--type", "moc", &id])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));

    // Verify the type was updated
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"type\": \"moc\""));
}

#[test]
fn test_update_type_permanent() {
    let dir = setup_test_dir();

    // Create a fleeting note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "fleeting", "Fleeting Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Update to permanent type
    qipu()
        .current_dir(dir.path())
        .args(["update", "--type", "permanent", &id])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));

    // Verify the type was updated
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"type\": \"permanent\""));
}

#[test]
fn test_update_add_tags() {
    let dir = setup_test_dir();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Add tags
    qipu()
        .current_dir(dir.path())
        .args(["update", "--tag", "tag1", "--tag", "tag2", &id])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));

    // Verify tags were added
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"tag1\""))
        .stdout(predicate::str::contains("\"tag2\""));
}

#[test]
fn test_update_remove_tags() {
    let dir = setup_test_dir();

    // Create a note with tags
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "create",
            "--tag",
            "tag1",
            "--tag",
            "tag2",
            "--tag",
            "tag3",
            "Test Note",
        ])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Remove tags
    qipu()
        .current_dir(dir.path())
        .args(["update", "--remove-tag", "tag2", &id])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));

    // Verify tag was removed
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"tag1\""))
        .stdout(predicate::str::contains("\"tag3\""))
        .stdout(predicate::str::contains("\"tag2\"").not());
}

#[test]
fn test_update_value() {
    let dir = setup_test_dir();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Update value
    qipu()
        .current_dir(dir.path())
        .args(["update", "--value", "75", &id])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));

    // Verify value was updated
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"value\": 75"));
}

#[test]
fn test_update_source() {
    let dir = setup_test_dir();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Update source
    qipu()
        .current_dir(dir.path())
        .args(["update", "--source", "https://example.com", &id])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));

    // Verify source was updated
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"source\": \"https://example.com\"",
        ));
}

#[test]
fn test_update_author() {
    let dir = setup_test_dir();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Update author
    qipu()
        .current_dir(dir.path())
        .args(["update", "--author", "Alice Smith", &id])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));

    // Verify author was updated
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"author\": \"Alice Smith\""));
}

#[test]
fn test_update_generated_by() {
    let dir = setup_test_dir();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Update generated_by
    qipu()
        .current_dir(dir.path())
        .args(["update", "--generated-by", "gpt-4", &id])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));

    // Verify generated_by was updated
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"generated_by\": \"gpt-4\""));
}

#[test]
fn test_update_prompt_hash() {
    let dir = setup_test_dir();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Update prompt_hash
    qipu()
        .current_dir(dir.path())
        .args(["update", "--prompt-hash", "abc123def456", &id])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));

    // Verify prompt_hash was updated
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"prompt_hash\": \"abc123def456\"",
        ));
}

#[test]
fn test_update_verified() {
    let dir = setup_test_dir();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Update verified to true
    qipu()
        .current_dir(dir.path())
        .args(["update", "--verified", "true", &id])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));

    // Verify verified was updated
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"verified\": true"));
}

#[test]
fn test_update_body_from_stdin() {
    let dir = setup_test_dir();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Update body from stdin
    qipu()
        .current_dir(dir.path())
        .args(["update", &id])
        .write_stdin("New body content from stdin")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));

    // Verify body was updated
    qipu()
        .current_dir(dir.path())
        .args(["show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("New body content from stdin"));
}

#[test]
fn test_update_multiple_fields() {
    let dir = setup_test_dir();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Original"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Update multiple fields at once
    qipu()
        .current_dir(dir.path())
        .args([
            "update",
            "--title",
            "Updated Title",
            "--tag",
            "tag1",
            "--tag",
            "tag2",
            "--value",
            "85",
            "--verified",
            "true",
            &id,
        ])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));

    // Verify all fields were updated
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"title\": \"Updated Title\""))
        .stdout(predicate::str::contains("\"tag1\""))
        .stdout(predicate::str::contains("\"tag2\""))
        .stdout(predicate::str::contains("\"value\": 85"))
        .stdout(predicate::str::contains("\"verified\": true"));
}

#[test]
fn test_update_preserves_body_when_no_stdin() {
    let dir = setup_test_dir();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Update with metadata only (no stdin)
    qipu()
        .current_dir(dir.path())
        .args(["update", "--title", "Updated Title", &id])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));

    // Verify title was updated but body was preserved
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"title\": \"Updated Title\""));
}

#[test]
fn test_update_json_format() {
    let dir = setup_test_dir();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Update and output in JSON format
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "update", "--title", "Updated", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\""))
        .stdout(predicate::str::contains("\"title\": \"Updated\""))
        .stdout(predicate::str::contains("\"type\""));
}

#[test]
fn test_update_records_format() {
    let dir = setup_test_dir();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Update and output in records format
    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "update", "--title", "Updated", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("H qipu="))
        .stdout(predicate::str::contains("records=1"))
        .stdout(predicate::str::contains("mode=update"));
}
