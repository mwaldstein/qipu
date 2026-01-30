use assert_cmd::{cargo::cargo_bin_cmd, Command};
use tempfile::tempdir;

fn qipu() -> Command {
    cargo_bin_cmd!("qipu")
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
