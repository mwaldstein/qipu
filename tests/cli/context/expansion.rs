use crate::support::setup_test_dir;
use crate::support::{extract_id, qipu};

#[test]
fn test_context_related_expansion() {
    let dir = setup_test_dir();

    // Create notes with similar content
    // Note 1: "machine learning algorithms"
    qipu()
        .current_dir(dir.path())
        .args(["create", "Machine Learning Algorithms"])
        .assert()
        .success();

    // Note 2: "machine learning techniques" - very similar to Note 1
    qipu()
        .current_dir(dir.path())
        .args(["create", "Machine Learning Techniques"])
        .assert()
        .success();

    // Note 3: "cooking recipes" - completely different
    qipu()
        .current_dir(dir.path())
        .args(["create", "Cooking Recipes"])
        .assert()
        .success();

    // Rebuild index for similarity calculation
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Get the ID of the first note
    let list_output = qipu().current_dir(dir.path()).arg("list").output().unwrap();
    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    let lines: Vec<&str> = list_stdout.lines().collect();
    let first_id = lines[1].split_whitespace().next().unwrap();

    // Test context with --related: should add similar note
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--note",
            first_id,
            "--related",
            "0.1",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let notes = json["notes"].as_array().unwrap();

    // Should include the selected note and at least one similar note
    assert!(
        notes.len() >= 2,
        "Should include selected note and similar notes, got {}",
        notes.len()
    );

    // All notes should have IDs
    for note in notes {
        assert!(note["id"].is_string());
    }
}

#[test]
fn test_context_backlinks() {
    let dir = setup_test_dir();

    // Create two notes
    let note1_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Source Note"])
        .output()
        .unwrap();
    let note1_id = extract_id(&note1_output);

    let note2_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Target Note"])
        .output()
        .unwrap();
    let note2_id = extract_id(&note2_output);

    // Create a link from note1 to note2
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &note1_id, &note2_id, "--type", "related"])
        .assert()
        .success();

    // Rebuild index to update database
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Test context with --backlinks: selecting note2 should include note1 (backlink)
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--note",
            &note2_id,
            "--backlinks",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let notes = json["notes"].as_array().unwrap();

    // Should include both notes
    assert_eq!(
        notes.len(),
        2,
        "Should include selected note and backlink source"
    );

    let note_ids: Vec<&str> = notes.iter().map(|n| n["id"].as_str().unwrap()).collect();

    assert!(note_ids.contains(&note1_id.as_str()));
    assert!(note_ids.contains(&note2_id.as_str()));

    // Test without --backlinks: should only include selected note
    let output = qipu()
        .current_dir(dir.path())
        .args(["context", "--note", &note2_id, "--format", "json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let notes = json["notes"].as_array().unwrap();

    assert_eq!(
        notes.len(),
        1,
        "Without --backlinks should only include selected note"
    );

    assert_eq!(notes[0]["id"].as_str().unwrap(), note2_id);
}
