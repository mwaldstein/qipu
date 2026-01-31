use predicates::prelude::*;
use tempfile::tempdir;

use crate::pack::support::qipu_store;

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
    qipu_store(store1_path).arg("init").assert().success();

    qipu_store(store1_path)
        .arg("create")
        .arg("Note A")
        .arg("--tag")
        .arg("original")
        .arg("--id")
        .arg(note_id)
        .assert()
        .success();

    // 2. Pack the note
    qipu_store(store1_path)
        .arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .arg("--format")
        .arg("json")
        .assert()
        .success();

    // 3. Initialize store 2 and create a note with same ID but different content
    qipu_store(store2_path).arg("init").assert().success();

    qipu_store(store2_path)
        .arg("create")
        .arg("Note B")
        .arg("--tag")
        .arg("modified")
        .arg("--id")
        .arg(note_id)
        .assert()
        .success();

    // 4. Load with overwrite strategy
    qipu_store(store2_path)
        .arg("load")
        .arg(&pack_file)
        .arg("--strategy")
        .arg("overwrite")
        .assert()
        .success();

    // 5. Verify the note has been overwritten with pack content
    let output = qipu_store(store2_path)
        .arg("show")
        .arg(note_id)
        .output()
        .unwrap();

    assert!(predicate::str::contains("Note A").eval(&String::from_utf8_lossy(&output.stdout)));
}
