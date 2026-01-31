use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

use crate::pack::support::qipu_store;

#[test]
fn test_pack_preserves_note_paths() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test.pack.json");

    // 1. Initialize store 1
    qipu_store(store1_path).arg("init").assert().success();

    // 2. Create a note
    let output = qipu_store(store1_path)
        .arg("create")
        .arg("Custom Path Note")
        .arg("--type")
        .arg("permanent")
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
    qipu_store(store1_path)
        .arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .assert()
        .success();

    // Verify pack was created
    assert!(pack_file.exists(), "Pack file should be created");

    // 5. Initialize store 2
    qipu_store(store2_path).arg("init").assert().success();

    // 6. Load into store 2
    qipu_store(store2_path)
        .arg("load")
        .arg(&pack_file)
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
    qipu_store(store2_path)
        .arg("show")
        .arg(note_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Custom Path Note"));
}
