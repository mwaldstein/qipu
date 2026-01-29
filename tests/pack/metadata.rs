use assert_cmd::{cargo::cargo_bin_cmd, Command};
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn qipu() -> Command {
    cargo_bin_cmd!("qipu")
}

#[test]
fn test_pack_preserves_value_and_custom_metadata() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test.pack");

    // 1. Initialize store 1
    qipu()
        .arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Create a note
    let output = qipu()
        .arg("create")
        .arg("Note with Value and Custom")
        .env("QIPU_STORE", store1_path)
        .output()
        .unwrap();
    let output_str = String::from_utf8_lossy(&output.stdout);
    let note_id = output_str.lines().next().unwrap().trim();

    // 3. Set value and custom metadata
    qipu()
        .arg("value")
        .arg("set")
        .arg(note_id)
        .arg("75")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    qipu()
        .arg("custom")
        .arg("set")
        .arg(note_id)
        .arg("priority")
        .arg("high")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    qipu()
        .arg("custom")
        .arg("set")
        .arg(note_id)
        .arg("count")
        .arg("42")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 4. Verify they are set
    qipu()
        .arg("show")
        .arg(note_id)
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""value": 75"#));

    // 5. Dump
    qipu()
        .arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // Verify pack contains value and custom
    let pack_content = fs::read_to_string(&pack_file).unwrap();
    assert!(
        pack_content.contains("value=75"),
        "Pack should contain value=75"
    );
    assert!(
        pack_content.contains("custom="),
        "Pack should contain custom metadata"
    );

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

    // 8. Verify value was preserved
    qipu()
        .arg("show")
        .arg(note_id)
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""value": 75"#));

    // 9. Verify custom metadata was preserved
    qipu()
        .arg("custom")
        .arg("get")
        .arg(note_id)
        .arg("priority")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("high"));

    qipu()
        .arg("custom")
        .arg("get")
        .arg(note_id)
        .arg("count")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}
