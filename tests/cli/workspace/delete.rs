use crate::support::{qipu, setup_test_dir};
use predicates::prelude::*;

#[test]
fn test_workspace_delete_removes_workspace() {
    let dir = setup_test_dir();

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
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["workspace", "delete", "nonexistent-workspace"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_workspace_delete_with_unmerged_changes() {
    let dir = setup_test_dir();

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
        .stderr(predicate::str::contains("unmerged"))
        .stderr(predicate::str::contains("--force"));
}

#[test]
fn test_workspace_delete_force_with_unmerged_changes() {
    let dir = setup_test_dir();

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
