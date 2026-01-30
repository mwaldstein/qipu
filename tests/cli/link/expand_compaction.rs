use crate::support::{extract_id, qipu};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_link_tree_expand_compaction_human_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create notes: note1 -> note2, and digest that compacts note2
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 1"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 2"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    let output3 = qipu()
        .current_dir(dir.path())
        .args(["create", "Digest Note"])
        .output()
        .unwrap();
    let digest_id = extract_id(&output3);

    // Add link from note1 to note2
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    // Modify digest note to compact note2
    let notes_dir = dir.path().join(".qipu/notes");
    for entry in fs::read_dir(&notes_dir).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        if name.to_string_lossy().starts_with(&digest_id) {
            let note_content = fs::read_to_string(entry.path()).unwrap();
            let new_content = note_content.replace(
                &format!("id: {}", digest_id),
                &format!("id: {}\ncompacts:\n  - {}", digest_id, id2),
            );
            fs::write(entry.path(), new_content).unwrap();
            break;
        }
    }

    // Rebuild index
    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success();

    // Test expand_compaction in human format
    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id1, "--expand-compaction"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show digest
    assert!(stdout.contains(&digest_id));
    // Should show compacted note content
    assert!(stdout.contains("Compacted Notes:"));
    assert!(stdout.contains(&id2));
}

#[test]
fn test_link_tree_expand_compaction_json_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 1"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 2"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    let output3 = qipu()
        .current_dir(dir.path())
        .args(["create", "Digest Note"])
        .output()
        .unwrap();
    let digest_id = extract_id(&output3);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    // Modify digest to compact note2
    let notes_dir = dir.path().join(".qipu/notes");
    for entry in fs::read_dir(&notes_dir).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        if name.to_string_lossy().starts_with(&digest_id) {
            let note_content = fs::read_to_string(entry.path()).unwrap();
            let new_content = note_content.replace(
                &format!("id: {}", digest_id),
                &format!("id: {}\ncompacts:\n  - {}", digest_id, id2),
            );
            fs::write(entry.path(), new_content).unwrap();
            break;
        }
    }

    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success();

    // Test expand_compaction in JSON format
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "tree",
            &id1,
            "--expand-compaction",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // JSON should contain compacted_notes array
    assert!(stdout.contains(r#""compacted_notes""#));
}

#[test]
fn test_link_tree_expand_compaction_records_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 1"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 2"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    let output3 = qipu()
        .current_dir(dir.path())
        .args(["create", "Digest Note"])
        .output()
        .unwrap();
    let digest_id = extract_id(&output3);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    // Modify digest to compact note2
    let notes_dir = dir.path().join(".qipu/notes");
    for entry in fs::read_dir(&notes_dir).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        if name.to_string_lossy().starts_with(&digest_id) {
            let note_content = fs::read_to_string(entry.path()).unwrap();
            let new_content = note_content.replace(
                &format!("id: {}", digest_id),
                &format!("id: {}\ncompacts:\n  - {}", digest_id, id2),
            );
            fs::write(entry.path(), new_content).unwrap();
            break;
        }
    }

    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success();

    // Test expand_compaction in records format
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "tree",
            &id1,
            "--expand-compaction",
            "--format",
            "records",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show compacted note with body
    assert!(stdout.contains(&format!("N {} {} \"Note 2\"", id2, "fleeting")));
    assert!(stdout.contains(&format!("B {}", id2)));
    assert!(stdout.contains(&format!("B-END {}", id2)));
}

#[test]
fn test_link_path_expand_compaction_human_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 1"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 2"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    let output3 = qipu()
        .current_dir(dir.path())
        .args(["create", "Digest Note"])
        .output()
        .unwrap();
    let digest_id = extract_id(&output3);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    // Modify digest to compact note2
    let notes_dir = dir.path().join(".qipu/notes");
    for entry in fs::read_dir(&notes_dir).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        if name.to_string_lossy().starts_with(&digest_id) {
            let note_content = fs::read_to_string(entry.path()).unwrap();
            let new_content = note_content.replace(
                &format!("id: {}", digest_id),
                &format!("id: {}\ncompacts:\n  - {}", digest_id, id2),
            );
            fs::write(entry.path(), new_content).unwrap();
            break;
        }
    }

    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success();

    // Test expand_compaction in human format for path
    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "path", &id1, &id2, "--expand-compaction"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show digest with expanded compaction
    assert!(stdout.contains(&digest_id));
    assert!(stdout.contains("Compacted Notes:"));
}

#[test]
fn test_link_path_expand_compaction_json_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 1"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 2"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    let output3 = qipu()
        .current_dir(dir.path())
        .args(["create", "Digest Note"])
        .output()
        .unwrap();
    let digest_id = extract_id(&output3);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    // Modify digest to compact note2
    let notes_dir = dir.path().join(".qipu/notes");
    for entry in fs::read_dir(&notes_dir).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        if name.to_string_lossy().starts_with(&digest_id) {
            let note_content = fs::read_to_string(entry.path()).unwrap();
            let new_content = note_content.replace(
                &format!("id: {}", digest_id),
                &format!("id: {}\ncompacts:\n  - {}", digest_id, id2),
            );
            fs::write(entry.path(), new_content).unwrap();
            break;
        }
    }

    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success();

    // Test expand_compaction in JSON format for path
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "path",
            &id1,
            &id2,
            "--expand-compaction",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // JSON should contain compacted_notes array
    assert!(stdout.contains(r#""compacted_notes""#));
}

#[test]
fn test_link_path_expand_compaction_records_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 1"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 2"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    let output3 = qipu()
        .current_dir(dir.path())
        .args(["create", "Digest Note"])
        .output()
        .unwrap();
    let digest_id = extract_id(&output3);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    // Modify digest to compact note2
    let notes_dir = dir.path().join(".qipu/notes");
    for entry in fs::read_dir(&notes_dir).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        if name.to_string_lossy().starts_with(&digest_id) {
            let note_content = fs::read_to_string(entry.path()).unwrap();
            let new_content = note_content.replace(
                &format!("id: {}", digest_id),
                &format!("id: {}\ncompacts:\n  - {}", digest_id, id2),
            );
            fs::write(entry.path(), new_content).unwrap();
            break;
        }
    }

    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success();

    // Test expand_compaction in records format for path
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "path",
            &id1,
            &id2,
            "--expand-compaction",
            "--format",
            "records",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show compacted note with body
    assert!(stdout.contains(&format!("N {} {} \"Note 2\"", id2, "fleeting")));
    assert!(stdout.contains(&format!("B {}", id2)));
    assert!(stdout.contains(&format!("B-END {}", id2)));
}
