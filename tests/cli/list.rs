use crate::cli::support::{extract_id, qipu};
use predicates::prelude::*;
use tempfile::tempdir;

// ============================================================================
// List command tests
// ============================================================================

#[test]
fn test_list_empty_store() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No notes found"));
}

#[test]
fn test_list_with_notes() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note
    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success();

    // List should show it
    qipu()
        .current_dir(dir.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Test Note"))
        .stdout(predicate::str::contains("qp-"));
}

#[test]
fn test_list_json_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "JSON List Test"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"title\": \"JSON List Test\""));
}

#[test]
fn test_list_filter_by_type() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create notes of different types
    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "fleeting", "Fleeting Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "permanent", "Permanent Note"])
        .assert()
        .success();

    // Filter by type
    qipu()
        .current_dir(dir.path())
        .args(["list", "--type", "fleeting"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Fleeting Note"))
        .stdout(predicate::str::contains("Permanent Note").not());
}

#[test]
fn test_list_filter_by_min_value_all_match() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "High Value Note"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Medium Value Note"])
        .output()
        .unwrap();
    let medium_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["create", "Low Value Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &medium_id, "75"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // All notes should match min-value 50 (default is 50, others are >= 50)
    qipu()
        .current_dir(dir.path())
        .args(["list", "--min-value", "50"])
        .assert()
        .success()
        .stdout(predicate::str::contains("High Value Note"))
        .stdout(predicate::str::contains("Medium Value Note"))
        .stdout(predicate::str::contains("Low Value Note"));
}

#[test]
fn test_list_filter_by_min_value_some_match() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "High Value Note"])
        .output()
        .unwrap();
    let high_id = extract_id(&output);

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Medium Value Note"])
        .output()
        .unwrap();
    let medium_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["create", "Low Value Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &high_id, "90"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &medium_id, "75"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Only high and medium value notes should match min-value 70
    qipu()
        .current_dir(dir.path())
        .args(["list", "--min-value", "70"])
        .assert()
        .success()
        .stdout(predicate::str::contains("High Value Note"))
        .stdout(predicate::str::contains("Medium Value Note"))
        .stdout(predicate::str::contains("Low Value Note").not());
}

#[test]
fn test_list_filter_by_min_value_none_match() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Note 1"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 2"])
        .output()
        .unwrap();
    let id2 = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id2, "30"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // No notes should match min-value 95 (default is 50, other is 30)
    qipu()
        .current_dir(dir.path())
        .args(["list", "--min-value", "95"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No notes found"))
        .stdout(predicate::str::contains("Note 1").not())
        .stdout(predicate::str::contains("Note 2").not());
}

#[test]
fn test_list_filter_by_min_value_with_defaults() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Explicit High Value"])
        .output()
        .unwrap();
    let high_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["create", "Default Value Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &high_id, "80"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Both should match min-value 50 (explicit 80 and default 50)
    qipu()
        .current_dir(dir.path())
        .args(["list", "--min-value", "50"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Explicit High Value"))
        .stdout(predicate::str::contains("Default Value Note"));
}

#[test]
fn test_list_filter_by_tag() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create notes with different tags
    qipu()
        .current_dir(dir.path())
        .args([
            "create",
            "--tag",
            "rust",
            "--tag",
            "programming",
            "Rust Note",
        ])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args([
            "create",
            "--tag",
            "python",
            "--tag",
            "programming",
            "Python Note",
        ])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--tag", "meeting", "Meeting Notes"])
        .assert()
        .success();

    // Filter by tag - should show only notes with 'rust' tag
    qipu()
        .current_dir(dir.path())
        .args(["list", "--tag", "rust"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust Note"))
        .stdout(predicate::str::contains("Python Note").not())
        .stdout(predicate::str::contains("Meeting Notes").not());

    // Filter by tag - should show notes with 'programming' tag
    qipu()
        .current_dir(dir.path())
        .args(["list", "--tag", "programming"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust Note"))
        .stdout(predicate::str::contains("Python Note"))
        .stdout(predicate::str::contains("Meeting Notes").not());
}

#[test]
fn test_list_filter_by_tag_no_matches() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--tag", "work", "Work Note"])
        .assert()
        .success();

    // Filter by non-existent tag
    qipu()
        .current_dir(dir.path())
        .args(["list", "--tag", "nonexistent"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No notes found"));
}

#[test]
fn test_list_filter_by_since() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create first note
    qipu()
        .current_dir(dir.path())
        .args(["create", "Old Note"])
        .assert()
        .success();

    // Wait a moment to ensure different timestamps
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Capture the timestamp for filtering
    let since_time = chrono::Utc::now().to_rfc3339();

    // Wait again to ensure the new note is after the timestamp
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Create second note
    qipu()
        .current_dir(dir.path())
        .args(["create", "Recent Note"])
        .assert()
        .success();

    // Filter by since - should show only the recent note
    qipu()
        .current_dir(dir.path())
        .args(["list", "--since", &since_time])
        .assert()
        .success()
        .stdout(predicate::str::contains("Recent Note"))
        .stdout(predicate::str::contains("Old Note").not());
}

#[test]
fn test_list_filter_by_since_no_matches() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Past Note"])
        .assert()
        .success();

    // Use a future timestamp - no notes should match
    let future_time = (chrono::Utc::now() + chrono::Duration::days(1)).to_rfc3339();

    qipu()
        .current_dir(dir.path())
        .args(["list", "--since", &future_time])
        .assert()
        .success()
        .stdout(predicate::str::contains("No notes found"));
}

#[test]
fn test_list_records_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--tag", "test", "--tag", "example", "Test Note"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "list"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify header line
    assert!(stdout.contains("H qipu=1 records=1"));
    assert!(stdout.contains("mode=list"));
    assert!(stdout.contains("notes=1"));

    // Verify note record
    assert!(stdout.contains("N qp-"));
    assert!(stdout.contains("\"Test Note\""));
    // Tags are alphabetically sorted
    assert!(stdout.contains("tags=example,test"));
}

#[test]
fn test_list_records_format_empty() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "list"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify header line shows zero notes
    assert!(stdout.contains("H qipu=1 records=1"));
    assert!(stdout.contains("mode=list"));
    assert!(stdout.contains("notes=0"));
}

#[test]
fn test_list_records_format_multiple_notes() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args([
            "create",
            "--type",
            "fleeting",
            "--tag",
            "urgent",
            "Fleeting Note",
        ])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "permanent", "Permanent Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "moc", "MOC Note"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "list"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify header shows correct count
    assert!(stdout.contains("notes=3"));

    // Verify all note types are present
    assert!(stdout.contains("fleeting"));
    assert!(stdout.contains("permanent"));
    assert!(stdout.contains("moc"));

    // Verify note titles are quoted
    assert!(stdout.contains("\"Fleeting Note\""));
    assert!(stdout.contains("\"Permanent Note\""));
    assert!(stdout.contains("\"MOC Note\""));

    // Verify tag handling
    assert!(stdout.contains("tags=urgent"));
    assert!(stdout.matches("tags=-").count() >= 2); // Notes without tags show "-"
}

#[test]
fn test_list_filter_by_custom_string() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Note with Custom"])
        .assert()
        .success()
        .get_output()
        .clone();

    let note_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &note_id, "status", "in-progress"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["list", "--custom", "status=in-progress"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Note with Custom"));

    qipu()
        .current_dir(dir.path())
        .args(["list", "--custom", "status=completed"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No notes found"));
}

#[test]
fn test_list_filter_by_custom_number() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "High Priority Note"])
        .assert()
        .success()
        .get_output()
        .clone();

    let note_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &note_id, "priority", "10"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["list", "--custom", "priority=10"])
        .assert()
        .success()
        .stdout(predicate::str::contains("High Priority Note"));

    qipu()
        .current_dir(dir.path())
        .args(["list", "--custom", "priority=5"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No notes found"));
}

#[test]
fn test_list_filter_by_custom_boolean() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Active Note"])
        .assert()
        .success()
        .get_output()
        .clone();
    let note1_id = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Inactive Note"])
        .assert()
        .success()
        .get_output()
        .clone();
    let note2_id = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &note1_id, "active", "true"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &note2_id, "active", "false"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["list", "--custom", "active=true"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Active Note"))
        .stdout(predicate::str::contains("Inactive Note").not());

    qipu()
        .current_dir(dir.path())
        .args(["list", "--custom", "active=false"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Inactive Note"))
        .stdout(predicate::str::contains("Active Note").not());
}

#[test]
fn test_list_filter_by_custom_with_tag() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "--tag", "work", "Work Note with Custom"])
        .assert()
        .success()
        .get_output()
        .clone();
    let work_id = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "--tag", "personal", "Personal Note with Custom"])
        .assert()
        .success()
        .get_output()
        .clone();
    let personal_id = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &work_id, "status", "review"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &personal_id, "status", "review"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["list", "--tag", "work", "--custom", "status=review"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Work Note with Custom"))
        .stdout(predicate::str::contains("Personal Note with Custom").not());
}

#[test]
fn test_list_records_format_truncated_field() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "list"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("H qipu=1 records=1"),
        "list records output should have valid header"
    );
    assert!(
        stdout.contains("mode=list"),
        "list records output should contain mode=list"
    );
}
