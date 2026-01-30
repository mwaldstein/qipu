use crate::support::{extract_id, qipu};
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_link_tree_with_compaction() {
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
        .args(["create", "Note 3"])
        .output()
        .unwrap();
    let id3 = extract_id(&output3);

    let output_digest = qipu()
        .current_dir(dir.path())
        .args(["create", "Digest Note"])
        .output()
        .unwrap();
    let digest_id = extract_id(&output_digest);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id2, &id3, "--type", "related"])
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
                &format!("id: {}\ncompacts:\n  - {}\n  - {}", digest_id, id1, id2),
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
        .args(["link", "tree", &digest_id])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains(&digest_id));
    assert!(stdout.contains(&id3));
    assert!(!stdout.contains(&id1));
    assert!(!stdout.contains(&id2));

    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id3, "--direction", "in"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains(&digest_id));
    assert!(!stdout.contains(&id1));
    assert!(!stdout.contains(&id2));
}

#[test]
fn test_link_tree_single_node() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Root Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id))
        .stdout(predicate::str::contains("Root Note"));
}

#[test]
fn test_link_tree_with_links() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Root"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Child 1"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    let output3 = qipu()
        .current_dir(dir.path())
        .args(["create", "Child 2"])
        .output()
        .unwrap();
    let id3 = extract_id(&output3);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id2, &id3, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id1])
        .assert()
        .success()
        .stdout(predicate::str::contains("Root"))
        .stdout(predicate::str::contains("Child 1"))
        .stdout(predicate::str::contains("Child 2"));
}
