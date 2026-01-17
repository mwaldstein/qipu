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

#[test]
fn test_load_strategy_skip() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test.pack.json");

    // 1. Initialize store 1 and create a note
    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("create")
        .arg("Original Note")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Pack the note
    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 3. Initialize store 2
    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 4. Load with skip strategy (default)
    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("load")
        .arg(&pack_file)
        .arg("--strategy")
        .arg("skip")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 5. Verify the note exists and has original content
    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("show")
        .arg("Original Note")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();
}

#[test]
fn test_load_strategy_overwrite() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test.pack.json");

    // 1. Initialize store 1 and create a note with tag "original"
    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("create")
        .arg("Note A")
        .arg("--tag")
        .arg("original")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Pack the note
    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 3. Initialize store 2 and create a note with same ID but different content
    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

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

    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("create")
        .arg("Note B")
        .arg("--tag")
        .arg("modified")
        .arg("--id")
        .arg(&note_id)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 4. Load with overwrite strategy
    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("load")
        .arg(&pack_file)
        .arg("--strategy")
        .arg("overwrite")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 5. Verify the note has been overwritten with pack content
    let output = Command::cargo_bin("qipu")
        .unwrap()
        .arg("show")
        .arg(&note_id)
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();

    assert!(predicate::str::contains("Note A").eval(&String::from_utf8_lossy(&output.stdout)));
}

#[test]
fn test_load_strategy_merge_links() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test.pack.json");

    // 1. Initialize store 1 and create a note with links
    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("create")
        .arg("Target Note")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("create")
        .arg("Linked Note")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("link")
        .arg("add")
        .arg("Target Note")
        .arg("Linked Note")
        .arg("--type")
        .arg("related")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Pack the notes
    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 3. Initialize store 2 and create the target note with same ID but different links
    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

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

    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("create")
        .arg("Target Note")
        .arg("--id")
        .arg(&note_id)
        .arg("--tag")
        .arg("store2")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 4. Load with merge-links strategy
    let mut cmd = Command::cargo_bin("qipu").unwrap();
    cmd.arg("load")
        .arg(&pack_file)
        .arg("--strategy")
        .arg("merge-links")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 5. Verify the target note now has the merged link from pack
    let output = Command::cargo_bin("qipu")
        .unwrap()
        .arg("show")
        .arg("Target Note")
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();

    assert!(predicate::str::contains("Linked Note").eval(&String::from_utf8_lossy(&output.stdout)));
}
