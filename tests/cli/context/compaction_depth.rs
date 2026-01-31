//! Tests for context command compaction depth
//! Tests for context command compaction depth control
use crate::support::setup_test_dir;
use crate::support::{extract_id, qipu};

// ============================================================================
// Compaction depth tests
// ============================================================================

#[test]
fn test_context_expand_compaction_with_depth() {
    use std::fs;

    let dir = setup_test_dir();

    // Create source notes
    qipu()
        .current_dir(dir.path())
        .args(["create", "Leaf Note 1"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Leaf Note 2"])
        .assert()
        .success();

    // Create intermediate digest
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Intermediate Digest"])
        .output()
        .unwrap();
    let intermediate_id = extract_id(&output1);

    // Create top-level digest
    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Top Digest"])
        .output()
        .unwrap();
    let top_id = extract_id(&output2);

    // Get note IDs
    let list_output = qipu()
        .current_dir(dir.path())
        .args(["list", "--format", "json"])
        .output()
        .unwrap();
    let list_json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&list_output.stdout)).unwrap();

    let notes = list_json.as_array().unwrap();
    let leaf_ids: Vec<String> = notes
        .iter()
        .filter(|n| n["title"].as_str().unwrap().starts_with("Leaf"))
        .map(|n| n["id"].as_str().unwrap().to_string())
        .collect();

    // Add compacts to intermediate digest
    let notes_dir = dir.path().join(".qipu").join("notes");
    for entry in fs::read_dir(&notes_dir).unwrap() {
        let entry = entry.unwrap();
        let note_path = entry.path();
        let note_content = fs::read_to_string(&note_path).unwrap();
        if note_content.contains(&format!("id: {}", intermediate_id)) {
            let new_content = note_content.replace(
                &format!("id: {}", intermediate_id),
                &format!(
                    "id: {}\ncompacts:\n  - {}\n  - {}",
                    intermediate_id, leaf_ids[0], leaf_ids[1]
                ),
            );
            fs::write(note_path, new_content).unwrap();
            break;
        }
    }

    // Add compacts to top digest
    for entry in fs::read_dir(&notes_dir).unwrap() {
        let entry = entry.unwrap();
        let note_path = entry.path();
        let note_content = fs::read_to_string(&note_path).unwrap();
        if note_content.contains(&format!("id: {}", top_id)) {
            let new_content = note_content.replace(
                &format!("id: {}", top_id),
                &format!("id: {}\ncompacts:\n  - {}", top_id, intermediate_id),
            );
            fs::write(note_path, new_content).unwrap();
            break;
        }
    }

    // Rebuild index
    // Use --rebuild to force re-indexing since file modification may be within same second as creation
    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success();

    // Test depth=1: should only show intermediate digest, not leaf notes
    let output1 = qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--note",
            &top_id,
            "--expand-compaction",
            "--compaction-depth",
            "1",
        ])
        .assert()
        .success();

    let stdout1 = String::from_utf8_lossy(&output1.get_output().stdout);
    assert!(stdout1.contains("Intermediate Digest"));
    assert!(!stdout1.contains("Leaf Note 1"));
    assert!(!stdout1.contains("Leaf Note 2"));

    // Test depth=2: should show both intermediate and leaf notes
    let output2 = qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--note",
            &top_id,
            "--expand-compaction",
            "--compaction-depth",
            "2",
        ])
        .assert()
        .success();

    let stdout2 = String::from_utf8_lossy(&output2.get_output().stdout);
    assert!(stdout2.contains("Intermediate Digest"));
    assert!(stdout2.contains("Leaf Note 1"));
    assert!(stdout2.contains("Leaf Note 2"));
}
