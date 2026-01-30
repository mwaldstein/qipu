use crate::support::qipu;
use predicates::prelude::*;
use tempfile::tempdir;

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

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Test Note"))
        .stdout(predicate::str::contains("qp-"));
}
