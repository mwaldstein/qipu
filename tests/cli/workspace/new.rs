use crate::support::{extract_id_from_bytes, qipu};
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_workspace_new_creates_workspace() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["workspace", "new", "test-workspace"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created workspace"))
        .stdout(predicate::str::contains("test-workspace"));
}

#[test]
fn test_workspace_new_empty() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["workspace", "new", "--empty", "empty-workspace"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["workspace", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("empty-workspace"));
}

#[test]
fn test_workspace_new_temp() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["workspace", "new", "--temp", "temp-workspace"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["workspace", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("temp-workspace"))
        .stdout(predicate::str::contains("Temp"));
}

#[test]
fn test_workspace_new_temp_adds_to_gitignore() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let gitignore_path = dir.path().join(".gitignore");

    qipu()
        .current_dir(dir.path())
        .args(["workspace", "new", "--temp", "temp-workspace"])
        .assert()
        .success();

    let gitignore_content = std::fs::read_to_string(&gitignore_path).unwrap();
    assert!(
        gitignore_content.contains(".qipu/workspaces/temp-workspace/"),
        "Gitignore should contain .qipu/workspaces/temp-workspace/"
    );
}

#[test]
fn test_workspace_new_temp_creates_gitignore_if_not_exists() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["workspace", "new", "--temp", "temp-workspace"])
        .assert()
        .success();

    let gitignore_path = dir.path().join(".gitignore");
    assert!(
        gitignore_path.exists(),
        "Gitignore should be created if it doesn't exist"
    );

    let gitignore_content = std::fs::read_to_string(&gitignore_path).unwrap();
    assert!(
        gitignore_content.contains(".qipu/workspaces/temp-workspace/"),
        "Gitignore should contain .qipu/workspaces/temp-workspace/"
    );
}

#[test]
fn test_workspace_new_temp_preserves_existing_gitignore_content() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let gitignore_path = dir.path().join(".gitignore");
    std::fs::write(&gitignore_path, "*.log\n*.tmp\n").unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["workspace", "new", "--temp", "temp-workspace"])
        .assert()
        .success();

    let gitignore_content = std::fs::read_to_string(&gitignore_path).unwrap();
    assert!(
        gitignore_content.contains("*.log"),
        "Gitignore should preserve existing *.log entry"
    );
    assert!(
        gitignore_content.contains("*.tmp"),
        "Gitignore should preserve existing *.tmp entry"
    );
    assert!(
        gitignore_content.contains(".qipu/workspaces/temp-workspace/"),
        "Gitignore should contain .qipu/workspaces/temp-workspace/"
    );
}

#[test]
fn test_workspace_new_temp_no_duplicate_gitignore_entry() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let gitignore_path = dir.path().join(".gitignore");
    std::fs::write(&gitignore_path, ".qipu/workspaces/temp-workspace/\n").unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["workspace", "new", "--temp", "temp-workspace"])
        .assert()
        .success();

    let gitignore_content = std::fs::read_to_string(&gitignore_path).unwrap();
    let entry_count = gitignore_content
        .lines()
        .filter(|l| l.trim() == ".qipu/workspaces/temp-workspace/")
        .count();
    assert_eq!(
        entry_count, 1,
        "Gitignore should not contain duplicate entries"
    );
}

#[test]
fn test_workspace_new_non_temp_no_gitignore_modification() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let gitignore_path = dir.path().join(".gitignore");
    std::fs::write(&gitignore_path, "*.log\n").unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["workspace", "new", "regular-workspace"])
        .assert()
        .success();

    let gitignore_content = std::fs::read_to_string(&gitignore_path).unwrap();
    assert!(
        !gitignore_content.contains("regular-workspace"),
        "Non-temp workspace should not modify .gitignore"
    );
    assert_eq!(gitignore_content, "*.log\n");
}

#[test]
fn test_workspace_new_non_temp_no_gitignore_created() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["workspace", "new", "regular-workspace"])
        .assert()
        .success();

    let gitignore_path = dir.path().join(".gitignore");
    assert!(
        !gitignore_path.exists(),
        "Non-temp workspace should not create .gitignore"
    );
}

#[test]
fn test_workspace_new_copy_primary() {
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
        .args(["workspace", "new", "--copy-primary", "copied-workspace"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["workspace", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("copied-workspace"))
        .stdout(predicate::str::contains("1"));
}

#[test]
fn test_workspace_new_from_tag() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--tag", "important", "Important Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Other Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args([
            "workspace",
            "new",
            "--from-tag",
            "important",
            "tagged-workspace",
        ])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["workspace", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tagged-workspace"))
        .stdout(predicate::str::contains("1"));
}

#[test]
fn test_workspace_new_from_note() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let note_id = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let note_id = extract_id_from_bytes(&note_id);

    qipu()
        .current_dir(dir.path())
        .args([
            "workspace",
            "new",
            "--from-note",
            &note_id,
            "note-workspace",
        ])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["workspace", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("note-workspace"));
}

#[test]
fn test_workspace_new_from_query() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Rust Programming"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Python Programming"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args([
            "workspace",
            "new",
            "--from-query",
            "Rust",
            "query-workspace",
        ])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["workspace", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("query-workspace"))
        .stdout(predicate::str::contains("1"));
}

#[test]
fn test_workspace_new_already_exists() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["workspace", "new", "test-workspace"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["workspace", "new", "test-workspace"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}
