use crate::support::{extract_id, qipu, setup_test_dir};
use predicates::prelude::*;

#[test]
fn test_verify_toggle() {
    let dir = setup_test_dir();

    // Create a note (default verified=false)
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Toggle verify (should set to true)
    qipu()
        .current_dir(dir.path())
        .args(["verify", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("verified: true"));

    // Verify it's now true
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"verified\": true"));

    // Toggle again (should set to false)
    qipu()
        .current_dir(dir.path())
        .args(["verify", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("verified: false"));
}

#[test]
fn test_verify_set_true() {
    let dir = setup_test_dir();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Set verified to true explicitly
    qipu()
        .current_dir(dir.path())
        .args(["verify", &id, "--status", "true"])
        .assert()
        .success()
        .stdout(predicate::str::contains("verified: true"));

    // Verify it's true
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"verified\": true"));
}

#[test]
fn test_verify_set_false() {
    let dir = setup_test_dir();

    // Create a verified note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "--verified", "true", "Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Set verified to false explicitly
    qipu()
        .current_dir(dir.path())
        .args(["verify", &id, "--status", "false"])
        .assert()
        .success()
        .stdout(predicate::str::contains("verified: false"));

    // Verify it's false
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"verified\": false"));
}

#[test]
fn test_verify_by_id() {
    let dir = setup_test_dir();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Verify using ID
    qipu()
        .current_dir(dir.path())
        .args(["verify", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("verified: true"));
}

#[test]
fn test_verify_by_path() {
    let dir = setup_test_dir();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Verify using relative path
    // Filename format: {id}-{slug(title)}.md
    qipu()
        .current_dir(dir.path())
        .args(["verify", &format!(".qipu/notes/{}-test-note.md", id)])
        .assert()
        .success()
        .stdout(predicate::str::contains("verified: true"));
}

#[test]
fn test_verify_json_format() {
    let dir = setup_test_dir();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Verify and output in JSON format
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "verify", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\""))
        .stdout(predicate::str::contains("\"verified\":true"))
        .stdout(predicate::str::contains("\"previous\":"));
}

#[test]
fn test_verify_records_format() {
    let dir = setup_test_dir();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Verify and output in records format
    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "verify", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("H qipu="))
        .stdout(predicate::str::contains("records=1"))
        .stdout(predicate::str::contains("mode=verify"))
        .stdout(predicate::str::contains("verified=true"));
}

#[test]
fn test_verify_with_previous_status_display() {
    let dir = setup_test_dir();

    // Create a verified note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "--verified", "true", "Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Verify with status=false and check previous status
    qipu()
        .current_dir(dir.path())
        .args(["verify", &id, "--status", "false"])
        .assert()
        .success()
        .stdout(predicate::str::contains("verified: false"))
        .stdout(predicate::str::contains("was: true"));
}
