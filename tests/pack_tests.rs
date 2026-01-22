use assert_cmd::{cargo::cargo_bin_cmd, Command};
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn qipu() -> Command {
    cargo_bin_cmd!("qipu")
}

#[test]
fn test_pack_unpack_json_roundtrip() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test.pack.json");

    // 1. Initialize store 1
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Create a note with all fields
    let mut cmd = qipu();
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
    let output = qipu()
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
        if entry.file_type().is_file() && entry.path().extension().is_some_and(|e| e == "md") {
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
    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 4. Initialize store 2
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 5. Unpack/Load into store 2
    let mut cmd = qipu();
    cmd.arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 6. Verify note in store 2
    let mut cmd = qipu();
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
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Create a note
    let mut cmd = qipu();
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
    let output = qipu()
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
        if entry.file_type().is_file() && entry.path().extension().is_some_and(|e| e == "md") {
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
    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .arg("--format")
        .arg("records")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 4. Initialize store 2
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 5. Unpack/Load into store 2
    let mut cmd = qipu();
    cmd.arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 6. Verify note in store 2
    let mut cmd = qipu();
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
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Original Note")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Pack the note
    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 3. Initialize store 2
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 4. Load with skip strategy (default)
    let mut cmd = qipu();
    cmd.arg("load")
        .arg(&pack_file)
        .arg("--strategy")
        .arg("skip")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 5. Verify the note exists and has original content
    let output = qipu()
        .arg("list")
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();

    let list: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let note_id = list[0]["id"].as_str().unwrap().to_string();

    let mut cmd = qipu();
    cmd.arg("show")
        .arg(&note_id)
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

    // Use a fixed ID for the test
    let note_id = "qp-test-overwrite";

    // 1. Initialize store 1 and create a note with tag "original"
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note A")
        .arg("--tag")
        .arg("original")
        .arg("--id")
        .arg(note_id)
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Pack the note
    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 3. Initialize store 2 and create a note with same ID but different content
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note B")
        .arg("--tag")
        .arg("modified")
        .arg("--id")
        .arg(note_id)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 4. Load with overwrite strategy
    let mut cmd = qipu();
    cmd.arg("load")
        .arg(&pack_file)
        .arg("--strategy")
        .arg("overwrite")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 5. Verify the note has been overwritten with pack content
    let output = qipu()
        .arg("show")
        .arg(note_id)
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

    // Use unique IDs with timestamp to avoid any parallel test interference
    let unique_suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let target_id = format!("qp-{}", unique_suffix);
    let linked_id = format!("qp-{}", unique_suffix + 1);

    // 1. Initialize store 1 and create a note with links
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Target Note")
        .arg("--id")
        .arg(&target_id)
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Linked Note")
        .arg("--id")
        .arg(&linked_id)
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg(&target_id)
        .arg(&linked_id)
        .arg("--type")
        .arg("related")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Pack the notes
    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 3. Initialize store 2 and create a target note with same ID but different links
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // Create target note in store2 with same ID as in store1
    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Target Note")
        .arg("--id")
        .arg(&target_id)
        .arg("--tag")
        .arg("store2")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 4. Load with merge-links strategy
    let mut cmd = qipu();
    cmd.arg("load")
        .arg(&pack_file)
        .arg("--strategy")
        .arg("merge-links")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 5. Verify the target note now has the merged link from pack
    let output = qipu()
        .arg("show")
        .arg(&target_id)
        .arg("--links")
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();

    assert!(
        predicate::str::contains(linked_id.as_str()).eval(&String::from_utf8_lossy(&output.stdout))
    );
}

#[test]
fn test_load_strategy_merge_links_preserves_content() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test.pack.json");

    // Use unique IDs with timestamp to avoid any parallel test interference
    let unique_suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let target_id = format!("qp-{}", unique_suffix);
    let linked_id = format!("qp-{}", unique_suffix + 1);

    // 1. Initialize store 1 and create notes with links
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Target Note")
        .arg("--id")
        .arg(&target_id)
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Linked Note")
        .arg("--id")
        .arg(&linked_id)
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg(&target_id)
        .arg(&linked_id)
        .arg("--type")
        .arg("related")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Pack the notes
    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 3. Initialize store 2 and create a target note with DIFFERENT content
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // Create target note in store2 with same ID but different title and tags
    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Different Title")
        .arg("--id")
        .arg(&target_id)
        .arg("--tag")
        .arg("store2-tag")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 4. Load with merge-links strategy
    let mut cmd = qipu();
    cmd.arg("load")
        .arg(&pack_file)
        .arg("--strategy")
        .arg("merge-links")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 5. Verify the target note's ORIGINAL content is preserved
    let output = qipu()
        .arg("show")
        .arg(&target_id)
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();

    let output_str = String::from_utf8_lossy(&output.stdout);

    // Title should be from store2 (original), not from pack
    assert!(predicate::str::contains("Different Title").eval(&output_str));
    assert!(!predicate::str::contains("Target Note").eval(&output_str));

    // Tag from store2 should be preserved
    assert!(predicate::str::contains("store2-tag").eval(&output_str));

    // Link from pack should be added
    assert!(predicate::str::contains(linked_id.as_str()).eval(&output_str));
}

#[test]
fn test_merge_links_only_merges_to_newly_loaded_notes() {
    // This test verifies that merge-links strategy only merges links
    // when the TARGET note was newly loaded, not when it already existed
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test.pack.json");

    // Use unique IDs to avoid parallel test interference
    let unique_suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let note_a_id = format!("qp-a-{}", unique_suffix);
    let note_b_id = format!("qp-b-{}", unique_suffix);
    let note_c_id = format!("qp-c-{}", unique_suffix);

    // 1. Initialize store 1 and create notes A, B, C with links
    qipu()
        .arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // Create note A
    qipu()
        .arg("create")
        .arg("Note A")
        .arg("--id")
        .arg(&note_a_id)
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // Create note B
    qipu()
        .arg("create")
        .arg("Note B")
        .arg("--id")
        .arg(&note_b_id)
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // Create note C
    qipu()
        .arg("create")
        .arg("Note C")
        .arg("--id")
        .arg(&note_c_id)
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // Add link from A to B (B will be pre-existing in store2)
    qipu()
        .arg("link")
        .arg("add")
        .arg(&note_a_id)
        .arg(&note_b_id)
        .arg("--type")
        .arg("supports")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // Add link from A to C (C will be newly loaded in store2)
    qipu()
        .arg("link")
        .arg("add")
        .arg(&note_a_id)
        .arg(&note_c_id)
        .arg("--type")
        .arg("related")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Dump all notes to pack
    qipu()
        .arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 3. Initialize store 2 with pre-existing note B (but not A or C)
    qipu()
        .arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // Create note B in store2 with same ID but different content
    qipu()
        .arg("create")
        .arg("Note B Pre-existing")
        .arg("--id")
        .arg(&note_b_id)
        .arg("--tag")
        .arg("pre-existing")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 4. Load pack with merge-links strategy
    qipu()
        .arg("load")
        .arg(&pack_file)
        .arg("--strategy")
        .arg("merge-links")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 5. Verify results:
    // - Note A should be created
    // - Note B should keep existing content (not overwritten)
    // - Note C should be created
    // - Note A's links:
    //   * Link to C should be PRESENT (C was newly loaded)
    //   * Link to B should NOT be present (B already existed)

    // Check note A - should have link to C but not to B
    let output_a = qipu()
        .arg("show")
        .arg(&note_a_id)
        .arg("--links")
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();
    let output_a_str = String::from_utf8_lossy(&output_a.stdout);

    // Should have link to newly loaded note C
    assert!(
        predicate::str::contains(&note_c_id).eval(&output_a_str),
        "Note A should have link to newly loaded note C\nOutput: {}",
        output_a_str
    );

    // Should NOT have link to pre-existing note B
    assert!(
        !predicate::str::contains(&note_b_id).eval(&output_a_str),
        "Note A should NOT have link to pre-existing note B\nOutput: {}",
        output_a_str
    );

    // Check note B - should preserve original content
    let output_b = qipu()
        .arg("show")
        .arg(&note_b_id)
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();
    let output_b_str = String::from_utf8_lossy(&output_b.stdout);

    assert!(
        predicate::str::contains("Note B Pre-existing").eval(&output_b_str),
        "Note B should preserve original title\nOutput: {}",
        output_b_str
    );
    assert!(
        predicate::str::contains("pre-existing").eval(&output_b_str),
        "Note B should preserve original tag\nOutput: {}",
        output_b_str
    );

    // Check note C - should be created
    let output_c = qipu()
        .arg("show")
        .arg(&note_c_id)
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();
    let output_c_str = String::from_utf8_lossy(&output_c.stdout);

    assert!(
        predicate::str::contains("Note C").eval(&output_c_str),
        "Note C should be created\nOutput: {}",
        output_c_str
    );
}

#[test]
fn test_pack_preserves_note_paths() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test.pack.json");

    // 1. Initialize store 1
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Create a note
    let mut cmd = qipu();
    let output = cmd
        .arg("create")
        .arg("Custom Path Note")
        .arg("--type")
        .arg("permanent")
        .env("QIPU_STORE", store1_path)
        .output()
        .unwrap();
    let output_str = String::from_utf8_lossy(&output.stdout);
    let note_id = output_str.lines().next().unwrap().trim();

    // 3. Find the note file path in store 1
    let mut original_note_path = None;
    for entry in walkdir::WalkDir::new(store1_path) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() && entry.path().extension().is_some_and(|e| e == "md") {
            let content = fs::read_to_string(entry.path()).unwrap();
            if content.contains(note_id) {
                original_note_path = Some(entry.path().to_path_buf());
                break;
            }
        }
    }
    let original_note_path = original_note_path.expect("Should find note file");

    // 4. Dump (which should include the path in pack)
    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // Verify pack was created
    assert!(pack_file.exists(), "Pack file should be created");

    // 5. Initialize store 2
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 6. Load into store 2
    let mut cmd = qipu();
    cmd.arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 7. Verify note was loaded at the correct path
    // The path in the pack is absolute to store1, so we need to check that the note
    // was loaded at a path with the same relative structure in store2
    let mut loaded_note_path = None;
    for entry in walkdir::WalkDir::new(store2_path) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() && entry.path().extension().is_some_and(|e| e == "md") {
            let content = fs::read_to_string(entry.path()).unwrap();
            if content.contains(note_id) {
                loaded_note_path = Some(entry.path().to_path_buf());
                break;
            }
        }
    }
    let loaded_note_path = loaded_note_path.expect("Should find loaded note file");

    // Get relative paths from their respective store roots
    let original_relative = original_note_path.strip_prefix(store1_path).unwrap();
    let loaded_relative = loaded_note_path.strip_prefix(store2_path).unwrap();

    // They should have the same relative path structure
    assert_eq!(
        original_relative, loaded_relative,
        "Loaded note should preserve the same relative path structure"
    );

    // 8. Verify the note is accessible via qipu show
    let mut cmd = qipu();
    cmd.arg("show")
        .arg(note_id)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Custom Path Note"));
}

#[test]
fn test_pack_attachments_roundtrip() {
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
    qipu()
        .arg("create")
        .arg("Note with Attachments")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // Get the ID of the created note
    let output = qipu()
        .arg("list")
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store1_path)
        .output()
        .unwrap();

    let list: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let note_id = list[0]["id"].as_str().unwrap().to_string();

    // 3. Create attachments directory and add test files
    let attachments_dir = store1_path.join("attachments");
    fs::create_dir_all(&attachments_dir).unwrap();

    let test_file1 = attachments_dir.join("test1.txt");
    fs::write(&test_file1, b"Test attachment 1 content").unwrap();

    let test_file2 = attachments_dir.join("test2.json");
    fs::write(&test_file2, b"{\"key\": \"value\"}").unwrap();

    let test_file3 = attachments_dir.join("image.png");
    // Create a minimal PNG file (1x1 pixel)
    let png_data = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, // IHDR chunk length
        0x49, 0x48, 0x44, 0x52, // "IHDR"
        0x00, 0x00, 0x00, 0x01, // Width: 1
        0x00, 0x00, 0x00, 0x01, // Height: 1
        0x08, 0x02, 0x00, 0x00, 0x00, // Bit depth, color type, etc.
        0x90, 0x77, 0x53, 0xDE, // CRC
        0x00, 0x00, 0x00, 0x0C, // IDAT chunk length
        0x49, 0x44, 0x41, 0x54, // "IDAT"
        0x08, 0x99, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D,
        0xB4, // IDAT data + CRC
        0x00, 0x00, 0x00, 0x00, // IEND chunk length
        0x49, 0x45, 0x4E, 0x44, // "IEND"
        0xAE, 0x42, 0x60, 0x82, // CRC
    ];
    fs::write(&test_file3, &png_data).unwrap();

    // 4. Reference attachments in the note
    // Find the actual note file
    let mut note_path = None;
    for entry in walkdir::WalkDir::new(store1_path.join("notes")) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            let content = fs::read_to_string(entry.path()).unwrap();
            if content.contains(&note_id) {
                note_path = Some(entry.path().to_path_buf());
                break;
            }
        }
    }
    let note_path = note_path.expect("Should find note file");
    let content = fs::read_to_string(&note_path).unwrap();
    let updated_content = content.replace(
        "## Notes\n",
        "## Notes\n\nSee attachment: ![test1](../attachments/test1.txt)\nAnd: ![test2](../attachments/test2.json)\nImage: ![image](../attachments/image.png)\n",
    );
    fs::write(&note_path, updated_content).unwrap();

    // Reindex to update database with new body content
    qipu()
        .arg("index")
        .arg("--rebuild")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 5. Dump with attachments (default behavior)
    qipu()
        .arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // Verify pack file contains attachment data
    let pack_content = fs::read_to_string(&pack_file).unwrap();
    assert!(pack_content.contains("name=test1.txt"));
    assert!(pack_content.contains("name=test2.json"));
    assert!(pack_content.contains("name=image.png"));

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

    // 8. Verify note was loaded
    qipu()
        .arg("show")
        .arg(&note_id)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Note with Attachments"));

    // 9. Verify attachments were restored
    let attachments_dir2 = store2_path.join("attachments");
    assert!(attachments_dir2.exists());

    let restored_file1 = attachments_dir2.join("test1.txt");
    assert!(restored_file1.exists());
    let content1 = fs::read(&restored_file1).unwrap();
    assert_eq!(content1, b"Test attachment 1 content");

    let restored_file2 = attachments_dir2.join("test2.json");
    assert!(restored_file2.exists());
    let content2 = fs::read(&restored_file2).unwrap();
    assert_eq!(content2, b"{\"key\": \"value\"}");

    let restored_file3 = attachments_dir2.join("image.png");
    assert!(restored_file3.exists());
    let content3 = fs::read(&restored_file3).unwrap();
    assert_eq!(content3, png_data);
}

#[test]
fn test_pack_no_attachments_flag() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test_no_attach.pack");

    // 1. Initialize store 1
    qipu()
        .arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Create a note
    qipu()
        .arg("create")
        .arg("Note without Attachments")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // Get the ID
    let output = qipu()
        .arg("list")
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store1_path)
        .output()
        .unwrap();

    let list: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let note_id = list[0]["id"].as_str().unwrap().to_string();

    // 3. Create attachments in store 1
    let attachments_dir = store1_path.join("attachments");
    fs::create_dir_all(&attachments_dir).unwrap();

    let test_file = attachments_dir.join("should_not_pack.txt");
    fs::write(&test_file, b"This should not be packed").unwrap();

    // Reference it in the note
    // Find the actual note file
    let mut note_path = None;
    for entry in walkdir::WalkDir::new(store1_path.join("notes")) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            let content = fs::read_to_string(entry.path()).unwrap();
            if content.contains(&note_id) {
                note_path = Some(entry.path().to_path_buf());
                break;
            }
        }
    }
    let note_path = note_path.expect("Should find note file");
    let content = fs::read_to_string(&note_path).unwrap();
    let updated_content = content.replace(
        "## Notes\n",
        "## Notes\n\nSee: ![file](../attachments/should_not_pack.txt)\n",
    );
    fs::write(&note_path, updated_content).unwrap();

    // Reindex to update database
    qipu()
        .arg("index")
        .arg("--rebuild")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 4. Dump with --no-attachments
    qipu()
        .arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .arg("--no-attachments")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // Verify pack file does NOT contain attachment data
    let pack_content = fs::read_to_string(&pack_file).unwrap();
    assert!(!pack_content.contains("name=should_not_pack.txt"));
    assert!(!pack_content.contains("This should not be packed"));

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

    // 7. Verify note was loaded
    qipu()
        .arg("show")
        .arg(&note_id)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Note without Attachments"));

    // 8. Verify attachments were NOT restored
    let attachments_dir2 = store2_path.join("attachments");
    let restored_file = attachments_dir2.join("should_not_pack.txt");
    assert!(!restored_file.exists());
}

#[test]
fn test_pack_attachments_multiple_notes() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test_multi.pack");

    // 1. Initialize store 1
    qipu()
        .arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Create two notes
    qipu()
        .arg("create")
        .arg("First Note")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    qipu()
        .arg("create")
        .arg("Second Note")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // Get IDs
    let output = qipu()
        .arg("list")
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store1_path)
        .output()
        .unwrap();

    let list: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let note1_id = list[0]["id"].as_str().unwrap().to_string();
    let note2_id = list[1]["id"].as_str().unwrap().to_string();

    // 3. Create attachments
    let attachments_dir = store1_path.join("attachments");
    fs::create_dir_all(&attachments_dir).unwrap();

    fs::write(attachments_dir.join("shared.txt"), b"Shared file").unwrap();
    fs::write(attachments_dir.join("note1_only.txt"), b"Note 1 only").unwrap();
    fs::write(attachments_dir.join("note2_only.txt"), b"Note 2 only").unwrap();

    // Reference attachments in notes
    // Find the actual note file for note1
    let mut note1_path = None;
    for entry in walkdir::WalkDir::new(store1_path.join("notes")) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            let content = fs::read_to_string(entry.path()).unwrap();
            if content.contains(&note1_id) {
                note1_path = Some(entry.path().to_path_buf());
                break;
            }
        }
    }
    let note1_path = note1_path.expect("Should find note1 file");
    let content1 = fs::read_to_string(&note1_path).unwrap();
    let updated1 = content1.replace(
        "## Notes\n",
        "## Notes\n\n![shared](../attachments/shared.txt)\n![note1](../attachments/note1_only.txt)\n",
    );
    fs::write(&note1_path, updated1).unwrap();

    // Find the actual note file for note2
    let mut note2_path = None;
    for entry in walkdir::WalkDir::new(store1_path.join("notes")) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            let content = fs::read_to_string(entry.path()).unwrap();
            if content.contains(&note2_id) && !content.contains(&note1_id) {
                note2_path = Some(entry.path().to_path_buf());
                break;
            }
        }
    }
    let note2_path = note2_path.expect("Should find note2 file");
    let content2 = fs::read_to_string(&note2_path).unwrap();
    let updated2 = content2.replace(
        "## Notes\n",
        "## Notes\n\n![shared](../attachments/shared.txt)\n![note2](../attachments/note2_only.txt)\n",
    );
    fs::write(&note2_path, updated2).unwrap();

    // Reindex to update database
    qipu()
        .arg("index")
        .arg("--rebuild")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 4. Dump all notes (default behavior)
    qipu()
        .arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // Verify pack contains all three attachments (deduplicated if shared)
    let pack_content = fs::read_to_string(&pack_file).unwrap();
    assert!(pack_content.contains("name=shared.txt"));
    assert!(pack_content.contains("name=note1_only.txt"));
    assert!(pack_content.contains("name=note2_only.txt"));

    // 5. Initialize store 2
    qipu()
        .arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 6. Load pack
    qipu()
        .arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 7. Verify all attachments restored
    let attachments_dir2 = store2_path.join("attachments");
    assert!(attachments_dir2.join("shared.txt").exists());
    assert!(attachments_dir2.join("note1_only.txt").exists());
    assert!(attachments_dir2.join("note2_only.txt").exists());

    let shared_content = fs::read(attachments_dir2.join("shared.txt")).unwrap();
    assert_eq!(shared_content, b"Shared file");
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
