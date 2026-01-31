//! Tests for pack/dump/load functionality
use assert_cmd::{cargo::cargo_bin_cmd, Command};
use tempfile::tempdir;

fn qipu() -> Command {
    cargo_bin_cmd!("qipu")
}

#[test]
fn test_dump_selector_multiple_notes_roundtrip() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test_notes.pack");

    // 1. Initialize store 1
    qipu()
        .arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Create multiple notes
    qipu()
        .arg("create")
        .arg("Note A")
        .arg("--id")
        .arg("note-a")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    qipu()
        .arg("create")
        .arg("Note B")
        .arg("--id")
        .arg("note-b")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    qipu()
        .arg("create")
        .arg("Note C")
        .arg("--id")
        .arg("note-c")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    qipu()
        .arg("create")
        .arg("Note D")
        .arg("--id")
        .arg("note-d")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 3. Dump with multiple --note selectors
    qipu()
        .arg("dump")
        .arg("--note")
        .arg("note-a")
        .arg("--note")
        .arg("note-c")
        .arg("--output")
        .arg(&pack_file)
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 4. Initialize store 2
    qipu()
        .arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 5. Load pack into store 2
    qipu()
        .arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 6. Verify only selected notes are loaded
    let output = qipu()
        .arg("list")
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();

    let list: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let ids: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap())
        .collect();

    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&"note-a"));
    assert!(ids.contains(&"note-c"));
    assert!(!ids.contains(&"note-b"));
    assert!(!ids.contains(&"note-d"));
}
