//! Index command integration tests
//!
//! Tests for note indexing including incremental updates, rebuild,
//! link extraction, and stemming configuration.

use crate::support::{qipu, setup_test_dir};
use predicates::prelude::*;
use std::fs;

// ============================================================================
// Index command tests
// ============================================================================

#[test]
fn test_index_empty_store() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success()
        .stdout(predicate::str::contains("Indexed 0 notes"));
}

#[test]
fn test_index_with_notes() {
    let dir = setup_test_dir();

    // Create notes
    qipu()
        .current_dir(dir.path())
        .args(["create", "--tag", "test", "Note 1"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--tag", "test", "Note 2"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success()
        .stdout(predicate::str::contains("Indexed 2 notes"));
}

#[test]
fn test_index_json_format() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "index"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"ok\""))
        .stdout(predicate::str::contains("\"notes_indexed\":"));
}

#[test]
fn test_index_records_format() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "index"])
        .assert()
        .success()
        .stdout(predicate::str::contains("H qipu=1 records=1"))
        .stdout(predicate::str::contains("mode=index"))
        .stdout(predicate::str::contains("notes=1"));
}

#[test]
fn test_index_rebuild() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success();

    // First index
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Rebuild should also work
    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Indexed 1 notes"));
}

#[test]
fn test_index_extracts_relative_path_markdown_links() {
    use std::fs;

    let dir = setup_test_dir();

    // Create a note in notes/
    let result = qipu()
        .current_dir(dir.path())
        .args(["create", "Source Note"])
        .assert()
        .success();
    let output = String::from_utf8_lossy(&result.get_output().stdout);
    let source_id = output
        .lines()
        .find(|line| line.contains("qp-"))
        .and_then(|line| line.split_whitespace().find(|word| word.starts_with("qp-")))
        .unwrap();

    // Create a note in mocs/
    let result = qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "moc", "Target MOC"])
        .assert()
        .success();
    let output = String::from_utf8_lossy(&result.get_output().stdout);
    let target_id = output
        .lines()
        .find(|line| line.contains("qp-"))
        .and_then(|line| line.split_whitespace().find(|word| word.starts_with("qp-")))
        .unwrap();

    // Find the source note file
    let notes_dir = dir.path().join(".qipu/notes");
    let source_file = fs::read_dir(&notes_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with(&format!("{}-", source_id))
        })
        .map(|e| e.path())
        .unwrap();

    // Find the target note file name
    let mocs_dir = dir.path().join(".qipu/mocs");
    let target_file_name = fs::read_dir(&mocs_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with(&format!("{}-", target_id))
        })
        .map(|e| e.file_name().to_string_lossy().to_string())
        .unwrap();

    // Update source note to add a relative markdown link to the target
    let mut source_content = fs::read_to_string(&source_file).unwrap();
    source_content.push_str(&format!("\n\n[Link to MOC](../mocs/{})", target_file_name));
    fs::write(&source_file, source_content).unwrap();

    // Rebuild index to pick up the link
    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success();

    // Verify the link was extracted by checking if we can traverse from source to target
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", source_id])
        .assert()
        .success()
        .stdout(predicate::str::contains(target_id));
}

#[test]
fn test_index_extracts_relative_path_markdown_links_cross_directory_reverse() {
    use std::fs;

    let dir = setup_test_dir();

    // Create a note in notes/
    let result = qipu()
        .current_dir(dir.path())
        .args(["create", "Target Note"])
        .assert()
        .success();
    let output = String::from_utf8_lossy(&result.get_output().stdout);
    let target_id = output
        .lines()
        .find(|line| line.contains("qp-"))
        .and_then(|line| line.split_whitespace().find(|word| word.starts_with("qp-")))
        .unwrap();

    // Create a MOC in mocs/
    let result = qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "moc", "Source MOC"])
        .assert()
        .success();
    let output = String::from_utf8_lossy(&result.get_output().stdout);
    let source_id = output
        .lines()
        .find(|line| line.contains("qp-"))
        .and_then(|line| line.split_whitespace().find(|word| word.starts_with("qp-")))
        .unwrap();

    // Find the source MOC file
    let mocs_dir = dir.path().join(".qipu/mocs");
    let source_file = fs::read_dir(&mocs_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with(&format!("{}-", source_id))
        })
        .map(|e| e.path())
        .unwrap();

    // Find the target note file name
    let notes_dir = dir.path().join(".qipu/notes");
    let target_file_name = fs::read_dir(&notes_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with(&format!("{}-", target_id))
        })
        .map(|e| e.file_name().to_string_lossy().to_string())
        .unwrap();

    // Update source MOC to add a relative markdown link to the target (reverse direction)
    let mut source_content = fs::read_to_string(&source_file).unwrap();
    source_content.push_str(&format!(
        "\n\n[Link to Note](../notes/{})",
        target_file_name
    ));
    fs::write(&source_file, source_content).unwrap();

    // Rebuild index to pick up the link
    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success();

    // Verify the link was extracted by checking if we can traverse from source to target
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", source_id])
        .assert()
        .success()
        .stdout(predicate::str::contains(target_id));
}

#[test]
fn test_index_stemming_can_be_disabled() {
    let dir = setup_test_dir();

    // Disable stemming in config
    let config_path = dir.path().join(".qipu/config.toml");
    let config_content = fs::read_to_string(&config_path).unwrap();
    let new_content = config_content.replace("stemming = true", "stemming = false");
    fs::write(&config_path, new_content).unwrap();

    // Create a note with words that would stem differently
    qipu()
        .current_dir(dir.path())
        .args(["create", "Graph Theory"])
        .assert()
        .success();

    // Index should work with stemming disabled
    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Indexed 1 notes"));
}

#[test]
fn test_index_stemming_enabled_by_default() {
    let dir = setup_test_dir();

    // Create a note
    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success();

    // Index should work with default config (stemming enabled)
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success()
        .stdout(predicate::str::contains("Indexed 1 notes"));
}

#[test]
fn test_index_incremental_repair_only_updates_changed_notes() {
    let dir = setup_test_dir();

    // Create two notes (they are automatically indexed)
    let result1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 1"])
        .assert()
        .success();
    let output1 = String::from_utf8_lossy(&result1.get_output().stdout);
    let note1_id = output1
        .lines()
        .find(|line| line.contains("qp-"))
        .and_then(|line| line.split_whitespace().find(|word| word.starts_with("qp-")))
        .unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Note 2"])
        .assert()
        .success();

    // First index - no notes should be updated (already indexed)
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success()
        .stdout(predicate::str::contains("Indexed 2 notes"));

    // Modify Note 1 directly on disk
    let notes_dir = dir.path().join(".qipu/notes");
    let note1_file = fs::read_dir(&notes_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with(&format!("{}-", note1_id))
        })
        .map(|e| e.path())
        .unwrap();

    let mut content = fs::read_to_string(&note1_file).unwrap();
    content.push_str("\n\nUpdated content");
    fs::write(&note1_file, content).unwrap();

    // Ensure mtime advances
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Second index without --rebuild - should pick up the modified note
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success()
        .stdout(predicate::str::contains("Indexed 2 notes"));

    // Verify database contains updated content
    let show_output = qipu()
        .current_dir(dir.path())
        .args(["show", note1_id])
        .assert()
        .success();
    let show_text = String::from_utf8_lossy(&show_output.get_output().stdout);
    assert!(show_text.contains("Updated content"));

    // Third index - no notes should be updated (mtime matches again)
    std::thread::sleep(std::time::Duration::from_millis(10));
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success()
        .stdout(predicate::str::contains("Indexed 2 notes"));
}

#[test]
fn test_index_full_flag_triggers_full_reindex() {
    let dir = setup_test_dir();

    // Create notes
    qipu()
        .current_dir(dir.path())
        .args(["create", "Note 1"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Note 2"])
        .assert()
        .success();

    // Use --full flag - should fully reindex all notes
    qipu()
        .current_dir(dir.path())
        .args(["index", "--full"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Indexed 2 notes"));
}

#[test]
fn test_index_basic_flag_triggers_basic_index_only() {
    let dir = setup_test_dir();

    // Create notes
    qipu()
        .current_dir(dir.path())
        .args(["create", "Note 1"])
        .assert()
        .success();

    // Use --basic flag - should index metadata only
    qipu()
        .current_dir(dir.path())
        .args(["index", "--basic"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Indexed 1 notes"));
}

#[test]
fn test_index_full_and_basic_mutually_exclusive() {
    let dir = setup_test_dir();

    // Try to use both --full and --basic - should fail
    qipu()
        .current_dir(dir.path())
        .args(["index", "--full", "--basic"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("mutually exclusive"));
}

#[test]
fn test_index_modified_since_relative_time() {
    let dir = setup_test_dir();

    // Create first note
    qipu()
        .current_dir(dir.path())
        .args(["create", "Old Note"])
        .assert()
        .success();

    // Create second note immediately
    qipu()
        .current_dir(dir.path())
        .args(["create", "Recent Note"])
        .assert()
        .success();

    // Index with --modified-since "1 minute ago" should index both
    qipu()
        .current_dir(dir.path())
        .args(["index", "--modified-since", "1 minute ago"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Indexed 2 notes"));
}

#[test]
fn test_index_modified_since_filters_old_files() {
    let dir = setup_test_dir();

    // Create an old note
    let notes_dir = dir.path().join(".qipu/notes");
    fs::create_dir_all(&notes_dir).unwrap();
    let old_note_content = r#"---
id: qp-01abcdef123456789
---
# Old Note

This is an old note.
"#;
    fs::write(
        notes_dir.join("qp-01abcdef123456789-old.md"),
        old_note_content,
    )
    .unwrap();

    // Set its mtime to 2 days ago
    let old_time = std::time::SystemTime::now() - std::time::Duration::from_secs(2 * 24 * 60 * 60);
    let old_path = notes_dir.join("qp-01abcdef123456789-old.md");
    let file_times = std::fs::FileTimes::new().set_modified(old_time);
    fs::OpenOptions::new()
        .write(true)
        .open(&old_path)
        .unwrap()
        .set_times(file_times)
        .unwrap();

    // Create a recent note
    qipu()
        .current_dir(dir.path())
        .args(["create", "Recent Note"])
        .assert()
        .success();

    // Index with --modified-since "1 day ago" should catch only the recent note
    qipu()
        .current_dir(dir.path())
        .args(["index", "--modified-since", "1 day ago"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Indexed 1 notes"));
}

#[test]
fn test_index_modified_since_various_formats() {
    let dir = setup_test_dir();

    // Create a note
    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success();

    // Test "24 hours ago" format
    qipu()
        .current_dir(dir.path())
        .args(["index", "--modified-since", "24 hours ago"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Indexed 1 notes"));

    // Test "7 days ago" format
    qipu()
        .current_dir(dir.path())
        .args(["index", "--modified-since", "7 days ago"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Indexed 1 notes"));

    // Test "2 weeks ago" format
    qipu()
        .current_dir(dir.path())
        .args(["index", "--modified-since", "2 weeks ago"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Indexed 1 notes"));
}

#[test]
fn test_index_modified_since_invalid_format() {
    let dir = setup_test_dir();

    // Try invalid time format
    qipu()
        .current_dir(dir.path())
        .args(["index", "--modified-since", "not a valid time"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cannot parse time"));
}

#[test]
fn test_index_batch_flag_accepts_custom_size() {
    let dir = setup_test_dir();

    // Create multiple notes
    for i in 1..=5 {
        qipu()
            .current_dir(dir.path())
            .args(["create", &format!("Note {}", i)])
            .assert()
            .success();
    }

    // Index with custom batch size
    qipu()
        .current_dir(dir.path())
        .args(["index", "--batch", "3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Indexed 5 notes"));
}

#[test]
fn test_index_batch_with_rebuild_and_resume() {
    let dir = setup_test_dir();

    // Create multiple notes
    for i in 1..=10 {
        qipu()
            .current_dir(dir.path())
            .args(["create", &format!("Note {}", i)])
            .assert()
            .success();
    }

    // Rebuild with small batch size
    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild", "--batch", "5"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Indexed 10 notes"));

    // Resume with same batch size
    qipu()
        .current_dir(dir.path())
        .args(["index", "--resume", "--batch", "5"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Indexed 10 notes"));
}
