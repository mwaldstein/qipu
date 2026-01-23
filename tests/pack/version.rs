use assert_cmd::{cargo::cargo_bin_cmd, Command};
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn qipu() -> Command {
    cargo_bin_cmd!("qipu")
}

#[test]
fn test_pack_unsupported_version_error() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test_bad_version.pack");

    // 1. Initialize store 1 and create a note
    qipu()
        .arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    qipu()
        .arg("create")
        .arg("Test Note")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Dump to create a valid pack (records format)
    qipu()
        .arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 3. Manually modify the pack version to an unsupported value
    let pack_content = fs::read_to_string(&pack_file).unwrap();
    let modified_pack = pack_content.replace("version=1.0", "version=2.0");
    fs::write(&pack_file, modified_pack).unwrap();

    // 4. Initialize store 2
    qipu()
        .arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 5. Try to load the pack - should fail with version error
    qipu()
        .arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("unsupported pack version: 2.0"));
}

#[test]
fn test_pack_store_version_too_high() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test_future_version.pack");

    // 1. Initialize store 1 and create a note
    qipu()
        .arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    qipu()
        .arg("create")
        .arg("Future Note")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Dump to create a valid pack (records format)
    qipu()
        .arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 3. Manually modify the store_version to be higher than STORE_FORMAT_VERSION (1)
    let pack_content = fs::read_to_string(&pack_file).unwrap();
    let modified_pack = pack_content.replace("store_version=1", "store_version=999");
    fs::write(&pack_file, modified_pack).unwrap();

    // 4. Initialize store 2
    qipu()
        .arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 5. Try to load the pack - should fail with version error
    qipu()
        .arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "pack store version 999 is higher than store version 1",
        ))
        .stderr(predicate::str::contains("please upgrade qipu"));
}

#[test]
fn test_pack_store_version_backward_compatible() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test_old_version.pack");

    // 1. Initialize store 1 and create a note
    qipu()
        .arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    qipu()
        .arg("create")
        .arg("Old Format Note")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // Get the note ID
    let output = qipu()
        .arg("list")
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store1_path)
        .output()
        .unwrap();

    let list: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let note_id = list[0]["id"].as_str().unwrap().to_string();

    // 2. Dump to create a valid pack (records format)
    qipu()
        .arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 3. Manually modify the store_version to be lower than STORE_FORMAT_VERSION
    // This simulates loading a pack from an older version of qipu
    let pack_content = fs::read_to_string(&pack_file).unwrap();
    // Set store_version to 0 (simulating a pack from a very old version)
    let modified_pack = pack_content.replace("store_version=1", "store_version=0");
    fs::write(&pack_file, modified_pack).unwrap();

    // 4. Initialize store 2
    qipu()
        .arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 5. Load the pack - should succeed (backward compatible)
    qipu()
        .arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 6. Verify the note was loaded successfully
    qipu()
        .arg("show")
        .arg(&note_id)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Old Format Note"));
}
