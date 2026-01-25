use crate::cli::support::{extract_id, qipu};
use tempfile::tempdir;

// ============================================================================
// Compaction expansion tests
// ============================================================================

#[test]
fn test_context_expand_compaction_human_format() {
    use std::fs;

    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create source notes
    qipu()
        .current_dir(dir.path())
        .args(["create", "Source Note One"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Source Note Two"])
        .assert()
        .success();

    // Create a digest note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Digest Note"])
        .output()
        .unwrap();
    let digest_id = extract_id(&output);

    // Get note IDs for source notes
    let list_output = qipu()
        .current_dir(dir.path())
        .args(["list", "--format", "json"])
        .output()
        .unwrap();
    let list_json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&list_output.stdout)).unwrap();

    let notes = list_json.as_array().unwrap();
    let source_ids: Vec<String> = notes
        .iter()
        .filter(|n| n["title"].as_str().unwrap().starts_with("Source"))
        .map(|n| n["id"].as_str().unwrap().to_string())
        .collect();

    assert_eq!(source_ids.len(), 2);

    // Manually add compacts field to digest note
    let notes_dir = dir.path().join(".qipu").join("notes");
    for entry in fs::read_dir(notes_dir).unwrap() {
        let entry = entry.unwrap();
        let note_path = entry.path();
        let note_content = fs::read_to_string(&note_path).unwrap();
        if note_content.contains(&format!("id: {}", digest_id)) {
            let new_content = note_content.replace(
                &format!("id: {}", digest_id),
                &format!(
                    "id: {}\ncompacts:\n  - {}\n  - {}",
                    digest_id, source_ids[0], source_ids[1]
                ),
            );
            fs::write(note_path, new_content).unwrap();
            break;
        }
    }

    // Rebuild index to pick up compaction
    // Use --rebuild to force re-indexing since file modification may be within same second as creation
    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success();

    // Test expand_compaction in human format
    let output = qipu()
        .current_dir(dir.path())
        .args(["context", "--note", &digest_id, "--expand-compaction"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("# Qipu Context Bundle"));
    assert!(stdout.contains("Digest Note"));
    assert!(stdout.contains("Compaction:"));
    assert!(stdout.contains("### Compacted Notes:"));
    assert!(stdout.contains("Source Note One"));
    assert!(stdout.contains("Source Note Two"));
}

#[test]
fn test_context_expand_compaction_json_format() {
    use std::fs;

    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create source notes
    qipu()
        .current_dir(dir.path())
        .args(["create", "Source Note A"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Source Note B"])
        .assert()
        .success();

    // Create a digest note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Digest Note"])
        .output()
        .unwrap();
    let digest_id = extract_id(&output);

    // Get note IDs for source notes
    let list_output = qipu()
        .current_dir(dir.path())
        .args(["list", "--format", "json"])
        .output()
        .unwrap();
    let list_json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&list_output.stdout)).unwrap();

    let notes = list_json.as_array().unwrap();
    let source_ids: Vec<String> = notes
        .iter()
        .filter(|n| n["title"].as_str().unwrap().starts_with("Source"))
        .map(|n| n["id"].as_str().unwrap().to_string())
        .collect();

    // Manually add compacts field to digest note
    let notes_dir = dir.path().join(".qipu").join("notes");
    for entry in fs::read_dir(notes_dir).unwrap() {
        let entry = entry.unwrap();
        let note_path = entry.path();
        let note_content = fs::read_to_string(&note_path).unwrap();
        if note_content.contains(&format!("id: {}", digest_id)) {
            let new_content = note_content.replace(
                &format!("id: {}", digest_id),
                &format!(
                    "id: {}\ncompacts:\n  - {}\n  - {}",
                    digest_id, source_ids[0], source_ids[1]
                ),
            );
            fs::write(note_path, new_content).unwrap();
            break;
        }
    }

    // Rebuild index to pick up compaction
    // Use --rebuild to force re-indexing since file modification may be within same second as creation
    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success();

    // Test expand_compaction in JSON format
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--note",
            &digest_id,
            "--expand-compaction",
            "--format",
            "json",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert!(json["notes"].is_array());
    let notes = json["notes"].as_array().unwrap();
    assert_eq!(notes.len(), 1);

    let digest_note = &notes[0];
    assert_eq!(digest_note["id"], digest_id);
    assert_eq!(digest_note["title"], "Digest Note");

    // Check that compacted_notes is present
    assert!(digest_note["compacted_notes"].is_array());
    let compacted_notes = digest_note["compacted_notes"].as_array().unwrap();
    assert_eq!(compacted_notes.len(), 2);

    // Check that compacted notes have full content
    for note in compacted_notes {
        assert!(note["id"].is_string());
        assert!(note["title"].is_string());
        assert!(note["content"].is_string());
        assert!(note["type"].is_string());
        assert!(note["tags"].is_array());
    }
}

#[test]
fn test_context_expand_compaction_records_format() {
    use std::fs;

    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create source notes
    qipu()
        .current_dir(dir.path())
        .args(["create", "Source Note X"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Source Note Y"])
        .assert()
        .success();

    // Create a digest note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Digest Note"])
        .output()
        .unwrap();
    let digest_id = extract_id(&output);

    // Get note IDs for source notes
    let list_output = qipu()
        .current_dir(dir.path())
        .args(["list", "--format", "json"])
        .output()
        .unwrap();
    let list_json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&list_output.stdout)).unwrap();

    let notes = list_json.as_array().unwrap();
    let source_ids: Vec<String> = notes
        .iter()
        .filter(|n| n["title"].as_str().unwrap().starts_with("Source"))
        .map(|n| n["id"].as_str().unwrap().to_string())
        .collect();

    // Manually add compacts field to digest note
    let notes_dir = dir.path().join(".qipu").join("notes");
    for entry in fs::read_dir(notes_dir).unwrap() {
        let entry = entry.unwrap();
        let note_path = entry.path();
        let note_content = fs::read_to_string(&note_path).unwrap();
        if note_content.contains(&format!("id: {}", digest_id)) {
            let new_content = note_content.replace(
                &format!("id: {}", digest_id),
                &format!(
                    "id: {}\ncompacts:\n  - {}\n  - {}",
                    digest_id, source_ids[0], source_ids[1]
                ),
            );
            fs::write(note_path, new_content).unwrap();
            break;
        }
    }

    // Rebuild index to pick up compaction
    // Use --rebuild to force re-indexing since file modification may be within same second as creation
    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success();

    // Test expand_compaction in records format
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--note",
            &digest_id,
            "--expand-compaction",
            "--format",
            "records",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    assert!(stdout.contains("H qipu=1 records=1 store="));
    assert!(stdout.contains(&format!("N {} fleeting \"Digest Note\"", digest_id)));
    assert!(stdout.contains("compacts=2"));

    // Check that compacted notes are included with full N, S, B lines
    for source_id in &source_ids {
        assert!(stdout.contains(&format!("N {}", source_id)));
        assert!(stdout.contains(&format!("compacted_from={}", digest_id)));
    }
}

#[test]
fn test_context_expand_compaction_with_depth() {
    use std::fs;

    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
