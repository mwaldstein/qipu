use crate::cli::support::qipu;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_link_list_with_compaction() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create three notes: note1 -> note2, and digest that compacts note2
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 1"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 2"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    let output3 = qipu()
        .current_dir(dir.path())
        .args(["create", "Digest Note"])
        .output()
        .unwrap();
    let digest_id = String::from_utf8_lossy(&output3.stdout).trim().to_string();

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
            // Insert compacts field in frontmatter
            let new_content = note_content.replace(
                &format!("id: {}", digest_id),
                &format!("id: {}\ncompacts:\n  - {}", digest_id, id2),
            );
            fs::write(entry.path(), new_content).unwrap();
            break;
        }
    }

    // Rebuild index to pick up compaction
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // link list on note1 should show link to digest_id (canonical), not note2
    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id1])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show canonical ID (digest)
    assert!(stdout.contains(&digest_id));
    // Should NOT show compacted note
    assert!(!stdout.contains(&id2));

    // link list on digest should show inbound link from note1
    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "list", &digest_id])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains(&id1));
}

#[test]
fn test_link_tree_with_compaction() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a scenario that would create a self-loop without compaction:
    // note1 -> note2, note2 -> note3, then compact all into digest
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 1"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 2"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    let output3 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 3"])
        .output()
        .unwrap();
    let id3 = String::from_utf8_lossy(&output3.stdout).trim().to_string();

    let output_digest = qipu()
        .current_dir(dir.path())
        .args(["create", "Digest Note"])
        .output()
        .unwrap();
    let digest_id = String::from_utf8_lossy(&output_digest.stdout)
        .trim()
        .to_string();

    // Add links: note1 -> note2 -> note3
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

    // Modify digest to compact note1 and note2
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

    // Rebuild index
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Tree from digest should show contracted graph (no self-loop)
    // It should show: digest -> note3
    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &digest_id])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show digest and note3
    assert!(stdout.contains(&digest_id));
    assert!(stdout.contains(&id3));
    // Should NOT show compacted notes
    assert!(!stdout.contains(&id1));
    assert!(!stdout.contains(&id2));

    // Tree from note3 going inbound should also use canonical IDs
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
fn test_link_path_with_compaction() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a path: start -> middle -> end
    // Then compact middle into digest
    let output_start = qipu()
        .current_dir(dir.path())
        .args(["create", "Start Note"])
        .output()
        .unwrap();
    let start_id = String::from_utf8_lossy(&output_start.stdout)
        .trim()
        .to_string();

    let output_middle = qipu()
        .current_dir(dir.path())
        .args(["create", "Middle Note"])
        .output()
        .unwrap();
    let middle_id = String::from_utf8_lossy(&output_middle.stdout)
        .trim()
        .to_string();

    let output_end = qipu()
        .current_dir(dir.path())
        .args(["create", "End Note"])
        .output()
        .unwrap();
    let end_id = String::from_utf8_lossy(&output_end.stdout)
        .trim()
        .to_string();

    let output_digest = qipu()
        .current_dir(dir.path())
        .args(["create", "Digest Note"])
        .output()
        .unwrap();
    let digest_id = String::from_utf8_lossy(&output_digest.stdout)
        .trim()
        .to_string();

    // Add links: start -> middle -> end
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &start_id, &middle_id, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &middle_id, &end_id, "--type", "related"])
        .assert()
        .success();

    // Modify digest to compact middle
    let notes_dir = dir.path().join(".qipu/notes");
    for entry in fs::read_dir(&notes_dir).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        if name.to_string_lossy().starts_with(&digest_id) {
            let note_content = fs::read_to_string(entry.path()).unwrap();
            let new_content = note_content.replace(
                &format!("id: {}", digest_id),
                &format!("id: {}\ncompacts:\n  - {}", digest_id, middle_id),
            );
            fs::write(entry.path(), new_content).unwrap();
            break;
        }
    }

    // Rebuild index
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Path from start to end should go through digest (canonical), not middle
    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "path", &start_id, &end_id])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show start -> digest -> end
    assert!(stdout.contains(&start_id));
    assert!(stdout.contains(&digest_id));
    assert!(stdout.contains(&end_id));
    // Should NOT show compacted middle note
    assert!(!stdout.contains(&middle_id));
    assert!(stdout.contains("Path length: 2 hop"));
}

#[test]
fn test_link_no_resolve_compaction_flag() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create note1 -> note2, and digest that compacts note2
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 1"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 2"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    let output3 = qipu()
        .current_dir(dir.path())
        .args(["create", "Digest Note"])
        .output()
        .unwrap();
    let digest_id = String::from_utf8_lossy(&output3.stdout).trim().to_string();

    // Add link
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

    // Rebuild index
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Test link list WITH --no-resolve-compaction flag
    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id1, "--no-resolve-compaction"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show the raw compacted note (note2), NOT the digest
    assert!(stdout.contains(&id2));
    assert!(!stdout.contains(&digest_id));

    // Test link tree WITH --no-resolve-compaction flag
    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id1, "--no-resolve-compaction"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show raw structure (note1 -> note2)
    assert!(stdout.contains(&id1));
    assert!(stdout.contains(&id2));
    // Digest shouldn't appear since we're showing raw links
    assert!(!stdout.contains(&digest_id));

    // Test link path WITH --no-resolve-compaction flag
    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "path", &id1, &id2, "--no-resolve-compaction"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show raw path (note1 -> note2)
    assert!(stdout.contains(&id1));
    assert!(stdout.contains(&id2));
    assert!(stdout.contains("Path length: 1 hop"));
}
