use crate::cli::support::qipu;
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

    let note_id = String::from_utf8(note_id).unwrap().trim().to_string();

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

#[test]
fn test_workspace_list_shows_primary() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["workspace", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("(primary)"))
        .stdout(predicate::str::contains("Name"));
}

#[test]
fn test_workspace_list_shows_workspaces() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["workspace", "new", "workspace-1"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["workspace", "new", "workspace-2"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["workspace", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("(primary)"))
        .stdout(predicate::str::contains("workspace-1"))
        .stdout(predicate::str::contains("workspace-2"));
}

#[test]
fn test_workspace_list_json_format() {
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
        .args(["--format", "json", "workspace", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""name""#))
        .stdout(predicate::str::contains(r#""path""#))
        .stdout(predicate::str::contains(r#""temporary""#))
        .stdout(predicate::str::contains(r#""note_count""#));
}

#[test]
fn test_workspace_list_records_format() {
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
        .args(["--format", "records", "workspace", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("WS (primary)"))
        .stdout(predicate::str::contains("WS test-workspace"));
}

#[test]
fn test_workspace_delete_removes_workspace() {
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
        .args(["workspace", "delete", "test-workspace"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted workspace"));

    qipu()
        .current_dir(dir.path())
        .args(["workspace", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("(primary)"))
        .stdout(predicate::str::contains("test-workspace").not());
}

#[test]
fn test_workspace_delete_nonexistent() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["workspace", "delete", "nonexistent-workspace"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_workspace_delete_with_unmerged_changes() {
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
        .args(["--workspace", "test-workspace", "create", "New Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["workspace", "delete", "test-workspace"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("unmerged"))
        .stderr(predicate::str::contains("--force"));
}

#[test]
fn test_workspace_delete_force_with_unmerged_changes() {
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
        .args(["--workspace", "test-workspace", "create", "New Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["workspace", "delete", "--force", "test-workspace"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted workspace"));

    qipu()
        .current_dir(dir.path())
        .args(["workspace", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test-workspace").not());
}
