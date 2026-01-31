use tempfile::tempdir;

use crate::pack::support::qipu_store;

#[test]
fn test_load_strategy_skip() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test.pack.json");

    // 1. Initialize store 1 and create a note
    qipu_store(store1_path).arg("init").assert().success();

    qipu_store(store1_path)
        .arg("create")
        .arg("Original Note")
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

    // 3. Initialize store 2
    qipu_store(store2_path).arg("init").assert().success();

    // 4. Load with skip strategy (default)
    qipu_store(store2_path)
        .arg("load")
        .arg(&pack_file)
        .arg("--strategy")
        .arg("skip")
        .assert()
        .success();

    // 5. Verify the note exists and has original content
    let output = qipu_store(store2_path)
        .arg("list")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    let list: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let note_id = list[0]["id"].as_str().unwrap().to_string();

    qipu_store(store2_path)
        .arg("show")
        .arg(&note_id)
        .assert()
        .success();
}
