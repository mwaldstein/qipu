//! Tests for link command
use crate::support::setup_test_dir;
use crate::support::{extract_id, qipu};
use std::fs;

#[test]
fn test_link_tree_expand_compaction_human_format() {
    let dir = setup_test_dir();

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

    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id1, "--expand-compaction"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains(&digest_id));
    assert!(stdout.contains("Compacted Notes:"));
    assert!(stdout.contains(&id2));
}

#[test]
fn test_link_tree_expand_compaction_json_format() {
    let dir = setup_test_dir();

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

    assert!(stdout.contains(r#""compacted_notes""#));
}

#[test]
fn test_link_tree_expand_compaction_records_format() {
    let dir = setup_test_dir();

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

    assert!(stdout.contains(&format!("N {} {} \"Note 2\"", id2, "fleeting")));
    assert!(stdout.contains(&format!("B {}", id2)));
    assert!(stdout.contains(&format!("B-END {}", id2)));
}
