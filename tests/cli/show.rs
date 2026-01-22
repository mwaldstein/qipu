use crate::cli::support::{extract_id, qipu};
use predicates::prelude::*;
use tempfile::tempdir;

// ============================================================================
// Show command tests
// ============================================================================

#[test]
fn test_show_note() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create and capture ID
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Show Test"])
        .output()
        .unwrap();

    let id = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap()
        .to_string();

    // Show should display the note
    qipu()
        .current_dir(dir.path())
        .args(["show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Show Test"))
        .stdout(predicate::str::contains(&id));
}

#[test]
fn test_show_nonexistent() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["show", "qp-nonexistent"])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_show_links_no_links() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Note with Links"])
        .output()
        .unwrap();

    let id = extract_id(&output);

    // Show --links with JSON format
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id, "--links"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"links\""))
        .stdout(predicate::str::contains(&id));
}

#[test]
fn test_show_links_records_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes and link them
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Source"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Target"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success();

    // Show --links with records format should include header and edge lines
    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "show", &id1, "--links"])
        .assert()
        .success()
        .stdout(predicate::str::contains("H qipu=1"))
        .stdout(predicate::str::contains("mode=show.links"))
        .stdout(predicate::str::contains("E "));
}

#[test]
fn test_show_json_includes_value() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note with value
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Valued Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Set value
    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id, "75"])
        .assert()
        .success();

    // Show with JSON format should include value
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"value\": 75"));
}

#[test]
fn test_show_records_includes_value() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note with value
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Valued Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Set value
    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id, "50"])
        .assert()
        .success();

    // Show with records format should include value
    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("value=50"));
}

#[test]
fn test_show_json_custom_omitted_by_default() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Custom Test"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Set custom metadata
    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &id, "priority", "high"])
        .assert()
        .success();

    // Show JSON without --custom should NOT include custom field
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("\"custom\""),
        "custom should be omitted by default"
    );
}

#[test]
fn test_show_json_custom_opt_in() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Custom Test"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Set custom metadata
    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &id, "priority", "high"])
        .assert()
        .success();

    // Show JSON with --custom should include custom field
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id, "--custom"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"custom\""))
        .stdout(predicate::str::contains("\"priority\""))
        .stdout(predicate::str::contains("high"));
}

#[test]
fn test_show_records_custom_opt_in() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Custom Records Test"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Set custom metadata
    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &id, "score", "42"])
        .assert()
        .success();

    // Show records with --custom should include C line
    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "show", &id, "--custom"])
        .assert()
        .success()
        .stdout(predicate::str::contains("C "))
        .stdout(predicate::str::contains("score=42"));
}
