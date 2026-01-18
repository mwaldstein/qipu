use crate::cli::support::qipu;
use predicates::prelude::*;
use tempfile::tempdir;

// ============================================================================
// Index command tests
// ============================================================================

#[test]
fn test_index_empty_store() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success()
        .stdout(predicate::str::contains("Indexed 0 notes"));
}

#[test]
fn test_index_with_notes() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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

    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
