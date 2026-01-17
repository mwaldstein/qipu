use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_pack_unpack_json_roundtrip() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test.pack.json");

    // 1. Initialize store 1
    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Create a note with all fields
    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("create")
        .arg("Test Note")
        .arg("--type")
        .arg("moc")
        .arg("--tag")
        .arg("tag1")
        .arg("--tag")
        .arg("tag2")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // Find the note ID from the output
    let output = Command::cargo_bin("qipu")
        .unwrap()
        .arg("list")
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store1_path)
        .output()
        .unwrap();

    let list: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let note_id = list[0]["id"].as_str().unwrap().to_string();

    // Find the note file and inject high-fidelity fields
    for entry in walkdir::WalkDir::new(store1_path) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() && entry.path().extension().map_or(false, |e| e == "md") {
            let content = fs::read_to_string(entry.path()).unwrap();
            if content.contains(&note_id) {
                let updated_content = content.replace(
                    "tags: [tag1, tag2]",
                    "tags: [tag1, tag2]\nsummary: \"Test summary\"\ncompacts: [comp1, comp2]\nsource: \"Test source\"\nauthor: \"Test author\"\ngenerated_by: \"Test generator\"\nprompt_hash: \"Test hash\"\nverified: true"
                );
                fs::write(entry.path(), updated_content).unwrap();
                break;
            }
        }
    }

    // 3. Pack to JSON
    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 4. Initialize store 2
    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 5. Unpack/Load into store 2
    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 6. Verify note in store 2
    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("show")
        .arg(&note_id)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Test Note"));
}

#[test]
fn test_pack_unpack_records_roundtrip() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test.pack.records");

    // 1. Initialize store 1
    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Create a note
    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("create")
        .arg("Test Note Records")
        .arg("--type")
        .arg("moc")
        .arg("--tag")
        .arg("tag1")
        .arg("--tag")
        .arg("tag2")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // Find the note ID
    let output = Command::cargo_bin("qipu")
        .unwrap()
        .arg("list")
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store1_path)
        .output()
        .unwrap();

    let list: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let note_id = list[0]["id"].as_str().unwrap().to_string();

    // Injected fields
    for entry in walkdir::WalkDir::new(store1_path) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() && entry.path().extension().map_or(false, |e| e == "md") {
            let content = fs::read_to_string(entry.path()).unwrap();
            if content.contains(&note_id) {
                let updated_content = content.replace(
                    "tags: [tag1, tag2]",
                    "tags: [tag1, tag2]\nsummary: \"Test summary records\"\ncompacts: [comp1, comp2]\nsource: \"Test source records\"\nauthor: \"Test author records\"\ngenerated_by: \"Test generator records\"\nprompt_hash: \"Test hash records\"\nverified: false"
                );
                fs::write(entry.path(), updated_content).unwrap();
                break;
            }
        }
    }

    // 3. Pack to Records
    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .arg("--format")
        .arg("records")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 4. Initialize store 2
    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 5. Unpack/Load into store 2
    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 6. Verify note in store 2
    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("show")
        .arg(&note_id)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();
}
