use crate::support::qipu;
use predicates::prelude::*;
use tempfile::tempdir;

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
