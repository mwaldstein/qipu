use assert_cmd::{cargo::cargo_bin_cmd, Command};
use predicates::prelude::*;
use tempfile::tempdir;

fn qipu() -> Command {
    cargo_bin_cmd!("qipu")
}

#[test]
fn test_load_strategy_overwrite() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test.pack.json");

    // Use a fixed ID for the test
    let note_id = "qp-test-overwrite";

    // 1. Initialize store 1 and create a note with tag "original"
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note A")
        .arg("--tag")
        .arg("original")
        .arg("--id")
        .arg(note_id)
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Pack the note
    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 3. Initialize store 2 and create a note with same ID but different content
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note B")
        .arg("--tag")
        .arg("modified")
        .arg("--id")
        .arg(note_id)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 4. Load with overwrite strategy
    let mut cmd = qipu();
    cmd.arg("load")
        .arg(&pack_file)
        .arg("--strategy")
        .arg("overwrite")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 5. Verify the note has been overwritten with pack content
    let output = qipu()
        .arg("show")
        .arg(note_id)
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();

    assert!(predicate::str::contains("Note A").eval(&String::from_utf8_lossy(&output.stdout)));
}
