use assert_cmd::{cargo::cargo_bin_cmd, Command};
use tempfile::tempdir;

fn qipu() -> Command {
    cargo_bin_cmd!("qipu")
}

#[test]
fn test_dump_selector_tag_roundtrip() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test_tag.pack");

    // 1. Initialize store 1
    qipu()
        .arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Create notes with different tags
    qipu()
        .arg("create")
        .arg("Project note A")
        .arg("--tag")
        .arg("project")
        .arg("--id")
        .arg("note-a")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    qipu()
        .arg("create")
        .arg("Project note B")
        .arg("--tag")
        .arg("project")
        .arg("--id")
        .arg("note-b")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    qipu()
        .arg("create")
        .arg("Personal note")
        .arg("--tag")
        .arg("personal")
        .arg("--id")
        .arg("note-c")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 3. Dump with --tag selector
    qipu()
        .arg("dump")
        .arg("--tag")
        .arg("project")
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

    // 6. Verify only notes with "project" tag are loaded
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
fn test_dump_selector_moc_roundtrip() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test_moc.pack");

    // 1. Initialize store 1
    qipu()
        .arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Create MOC note
    qipu()
        .arg("create")
        .arg("My MOC")
        .arg("--type")
        .arg("moc")
        .arg("--id")
        .arg("my-moc")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 3. Create content notes
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

    // 4. Link notes from MOC (A and B are linked, C is not)
    qipu()
        .arg("link")
        .arg("add")
        .arg("my-moc")
        .arg("note-a")
        .arg("--type")
        .arg("has-part")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    qipu()
        .arg("link")
        .arg("add")
        .arg("my-moc")
        .arg("note-b")
        .arg("--type")
        .arg("has-part")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 5. Dump with --moc selector
    qipu()
        .arg("dump")
        .arg("--moc")
        .arg("my-moc")
        .arg("--output")
        .arg(&pack_file)
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 6. Initialize store 2
    qipu()
        .arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 7. Load pack into store 2
    qipu()
        .arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 8. Verify notes linked from MOC are loaded (A and B)
    let output = qipu()
        .arg("list")
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();

    let list: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let mut ids: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap())
        .collect();
    ids.sort();

    assert!(ids.contains(&"note-a"), "Should contain note-a");
    assert!(ids.contains(&"note-b"), "Should contain note-b");
    assert!(
        !ids.contains(&"note-c"),
        "Should not contain note-c (not linked from MOC)"
    );
}

#[test]
fn test_dump_selector_query_roundtrip() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test_query.pack");

    // 1. Initialize store 1
    qipu()
        .arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Create notes with searchable content
    qipu()
        .arg("create")
        .arg("Rust programming tutorial")
        .arg("--id")
        .arg("note-a")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    qipu()
        .arg("create")
        .arg("Advanced Rust techniques")
        .arg("--id")
        .arg("note-b")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    qipu()
        .arg("create")
        .arg("Python basics")
        .arg("--id")
        .arg("note-c")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 3. Rebuild index to ensure FTS is updated
    qipu()
        .arg("index")
        .arg("--rebuild")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 4. Dump with --query selector
    qipu()
        .arg("dump")
        .arg("--query")
        .arg("Rust")
        .arg("--output")
        .arg(&pack_file)
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 5. Initialize store 2
    qipu()
        .arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 6. Load pack into store 2
    qipu()
        .arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 7. Verify only notes matching query are loaded
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

#[test]
fn test_dump_selector_no_selector_full_store_roundtrip() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test_full.pack");

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
        .arg("--tag")
        .arg("tag1")
        .arg("--id")
        .arg("note-a")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    qipu()
        .arg("create")
        .arg("Note B")
        .arg("--tag")
        .arg("tag2")
        .arg("--id")
        .arg("note-b")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    qipu()
        .arg("create")
        .arg("Note C")
        .arg("--type")
        .arg("moc")
        .arg("--id")
        .arg("note-c")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 3. Create links
    qipu()
        .arg("link")
        .arg("add")
        .arg("note-a")
        .arg("note-b")
        .arg("--type")
        .arg("related")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 4. Dump without any selector (should dump entire store)
    qipu()
        .arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 5. Initialize store 2
    qipu()
        .arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 6. Load pack into store 2
    qipu()
        .arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 7. Verify all notes are loaded
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

    assert_eq!(ids.len(), 3);
    assert!(ids.contains(&"note-a"));
    assert!(ids.contains(&"note-b"));
    assert!(ids.contains(&"note-c"));

    // 8. Verify links are preserved
    let output = qipu()
        .arg("link")
        .arg("list")
        .arg("note-a")
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();

    let links: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let link_targets: Vec<&str> = links
        .as_array()
        .unwrap()
        .iter()
        .filter(|l| l["direction"].as_str() == Some("out"))
        .map(|l| l["id"].as_str().unwrap())
        .collect();

    assert!(link_targets.contains(&"note-b"));
}
