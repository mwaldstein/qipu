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
        .arg("permanent")
        .arg("--tag")
        .arg("tag1")
        .arg("--tag")
        .arg("tag2")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // Get the ID of the created note
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

    // Add extra metadata manually to the file since 'create' doesn't support all fields yet
    let note_path = store1_path
        .join("notes")
        .join(format!("{}-test-note.md", note_id));
    let content = fs::read_to_string(&note_path).unwrap();
    let updated_content = content.replace(
        "tags: [tag1, tag2]",
        "tags: [tag1, tag2]\nsummary: \"Test summary\"\ncompacts: [comp1, comp2]\nsource: \"Test source\"\nauthor: \"Test author\"\ngenerated_by: \"Test generator\"\nprompt_hash: \"Test hash\"\nverified: true"
    );
    fs::write(&note_path, updated_content).unwrap();

    // 3. Pack to JSON
    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("dump")
        .arg(&note_id)
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
        .stdout(predicate::str::contains("Test Note"))
        .stdout(predicate::str::contains("tag1"))
        .stdout(predicate::str::contains("tag2"));

    // Check file content for high-fidelity fields
    let note2_path = store2_path
        .join("notes")
        .join(format!("{}-test-note.md", note_id));
    let content2 = fs::read_to_string(&note2_path).unwrap();
    assert!(content2.contains("summary: Test summary"));
    assert!(content2.contains("compacts: [comp1, comp2]"));
    assert!(content2.contains("source: Test source"));
    assert!(content2.contains("author: Test author"));
    assert!(content2.contains("generated_by: Test generator"));
    assert!(content2.contains("prompt_hash: Test hash"));
    assert!(content2.contains("verified: true"));
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

    // 2. Create a note with all fields
    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("create")
        .arg("Test Note Records")
        .arg("--type")
        .arg("permanent")
        .arg("--tag")
        .arg("tag1")
        .arg("--tag")
        .arg("tag2")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // Get the ID of the created note
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

    // Add extra metadata manually
    let note_path = store1_path
        .join("notes")
        .join(format!("{}-test-note-records.md", note_id));
    let content = fs::read_to_string(&note_path).unwrap();
    let updated_content = content.replace(
        "tags: [tag1, tag2]",
        "tags: [tag1, tag2]\nsummary: \"Test summary records\"\ncompacts: [comp1, comp2]\nsource: \"Test source records\"\nauthor: \"Test author records\"\ngenerated_by: \"Test generator records\"\nprompt_hash: \"Test hash records\"\nverified: false"
    );
    fs::write(&note_path, updated_content).unwrap();

    // 3. Pack to Records
    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("dump")
        .arg(&note_id)
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

    // Check file content for high-fidelity fields
    let note2_path = store2_path
        .join("notes")
        .join(format!("{}-test-note-records.md", note_id));
    let content2 = fs::read_to_string(&note2_path).unwrap();
    assert!(content2.contains("summary: Test summary records"));
    assert!(content2.contains("compacts: [comp1, comp2]"));
    assert!(content2.contains("source: Test source records"));
    assert!(content2.contains("author: Test author records"));
    assert!(content2.contains("generated_by: Test generator records"));
    assert!(content2.contains("prompt_hash: Test hash records"));
    assert!(content2.contains("verified: false"));
}
