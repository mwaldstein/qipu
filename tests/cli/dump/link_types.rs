use crate::support::qipu;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_dump_type_filter_affects_reachability() {
    let dir = tempdir().unwrap();
    let store_path = dir.path();
    let pack_file = dir.path().join("test.pack");

    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note A")
        .arg("--id")
        .arg("note-a")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note B")
        .arg("--id")
        .arg("note-b")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note C")
        .arg("--id")
        .arg("note-c")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg("note-a")
        .arg("note-b")
        .arg("--type")
        .arg("supports")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg("note-a")
        .arg("note-c")
        .arg("--type")
        .arg("related")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--note")
        .arg("note-a")
        .arg("--output")
        .arg(&pack_file)
        .arg("--max-hops")
        .arg("1")
        .arg("--direction")
        .arg("out")
        .arg("--type")
        .arg("supports")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();

    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

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
    assert!(ids.contains(&"note-b"));
    assert!(!ids.contains(&"note-c"));
}

#[test]
fn test_dump_typed_only_excludes_inline_links() {
    let dir = tempdir().unwrap();
    let store_path = dir.path();
    let pack_file = dir.path().join("test.pack");

    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note A")
        .arg("--id")
        .arg("note-a")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note B")
        .arg("--id")
        .arg("note-b")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note C")
        .arg("--id")
        .arg("note-c")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg("note-a")
        .arg("note-b")
        .arg("--type")
        .arg("related")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let note_a_path = store_path.join("notes").join("note-a-note-a.md");
    let content = fs::read_to_string(&note_a_path).unwrap();
    let updated_content = format!("{}\n\nSee [[note-c]] for more info.", content);
    fs::write(&note_a_path, updated_content).unwrap();

    let mut cmd = qipu();
    cmd.arg("index")
        .arg("--rebuild")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--note")
        .arg("note-a")
        .arg("--output")
        .arg(&pack_file)
        .arg("--max-hops")
        .arg("1")
        .arg("--direction")
        .arg("out")
        .arg("--typed-only")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();

    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

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
    assert!(ids.contains(&"note-b"));
    assert!(!ids.contains(&"note-c"));
}

#[test]
fn test_dump_inline_only_excludes_typed_links() {
    let dir = tempdir().unwrap();
    let store_path = dir.path();
    let pack_file = dir.path().join("test.pack");

    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note A")
        .arg("--id")
        .arg("note-a")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note B")
        .arg("--id")
        .arg("note-b")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note C")
        .arg("--id")
        .arg("note-c")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg("note-a")
        .arg("note-b")
        .arg("--type")
        .arg("related")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let note_a_path = store_path.join("notes").join("note-a-note-a.md");
    let content = fs::read_to_string(&note_a_path).unwrap();
    let updated_content = format!("{}\n\nSee [[note-c]] for more info.", content);
    fs::write(&note_a_path, updated_content).unwrap();

    let mut cmd = qipu();
    cmd.arg("index")
        .arg("--rebuild")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--note")
        .arg("note-a")
        .arg("--output")
        .arg(&pack_file)
        .arg("--max-hops")
        .arg("1")
        .arg("--direction")
        .arg("out")
        .arg("--inline-only")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();

    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

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

    eprintln!("Dumped notes: {:?}", ids);
    assert_eq!(ids.len(), 2, "Expected 2 notes, got: {:?}", ids);
    assert!(ids.contains(&"note-a"), "Should contain note-a");
    assert!(ids.contains(&"note-c"), "Should contain note-c");
    assert!(
        !ids.contains(&"note-b"),
        "Should not contain note-b (filtered by --inline-only)"
    );
}
