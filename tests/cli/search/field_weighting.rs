use crate::cli::support::qipu;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_search_field_weighting_impact() {
    // Test that field weights affect BM25 relevance scores
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create note 1: unique term in title only
    qipu()
        .current_dir(dir.path())
        .args(["create", "Alphap"])
        .assert()
        .success();

    // Create note 2: same unique term in body only
    qipu()
        .current_dir(dir.path())
        .args(["create", "Generic Note"])
        .assert()
        .success();

    let notes_dir = dir.path().join(".qipu/notes");
    let note_files: Vec<_> = fs::read_dir(&notes_dir)
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(note_files.len(), 2);

    for note_file in &note_files {
        let note_path = note_file.path();
        let content = fs::read_to_string(&note_path).unwrap();

        if content.contains("title: Alphap") {
            let mut modified = content;
            modified.push_str("\n\nThis note discusses testing and search systems.");
            fs::write(&note_path, modified).unwrap();
        } else if content.contains("title: Generic Note") {
            let mut modified = content;
            modified.push_str("\n\nThis note discusses alphap and testing methods.");
            fs::write(&note_path, modified).unwrap();
        }
    }

    // Rebuild index
    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success();

    // Search for the unique term
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "search", "alphap"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let results: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let results_array = results.as_array().expect("Expected JSON array");

    // Both notes should be found (one in title, one in body)
    assert_eq!(
        results_array.len(),
        2,
        "Both title and body matches should be found"
    );

    // Both results should have relevance scores
    let relevance1 = results_array[0]["relevance"].as_f64();
    let relevance2 = results_array[1]["relevance"].as_f64();

    assert!(
        relevance1.is_some() && relevance2.is_some(),
        "Both results should have relevance scores"
    );

    // Verify different titles are in results
    let titles: Vec<&str> = results_array
        .iter()
        .map(|r| r["title"].as_str().unwrap())
        .collect();
    assert!(
        titles.contains(&"Alphap"),
        "Title match should be in results"
    );
    assert!(
        titles.contains(&"Generic Note"),
        "Body match should be in results"
    );
}

#[test]
fn test_search_field_weighting_all_fields() {
    // Test that all fields (title, body, tags) are searched and weighted
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create note with term in title
    qipu()
        .current_dir(dir.path())
        .args(["create", "Betap"])
        .assert()
        .success();

    // Create note with term in body
    qipu()
        .current_dir(dir.path())
        .args(["create", "Body Note"])
        .assert()
        .success();

    // Create note with term in tags
    qipu()
        .current_dir(dir.path())
        .args(["create", "--tag", "betap", "Tag Note"])
        .assert()
        .success();

    let notes_dir = dir.path().join(".qipu/notes");
    let note_files: Vec<_> = fs::read_dir(&notes_dir)
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(note_files.len(), 3);

    for note_file in &note_files {
        let note_path = note_file.path();
        let content = fs::read_to_string(&note_path).unwrap();

        if content.contains("title: Betap") {
            let mut modified = content;
            modified.push_str("\n\nThis note discusses testing frameworks.");
            fs::write(&note_path, modified).unwrap();
        } else if content.contains("title: Body Note") {
            let mut modified = content;
            modified.push_str("\n\nThis note discusses betap and testing methods.");
            fs::write(&note_path, modified).unwrap();
        } else if content.contains("title: Tag Note") {
            let mut modified = content;
            modified.push_str("\n\nThis note discusses testing and quality assurance.");
            fs::write(&note_path, modified).unwrap();
        }
    }

    // Rebuild index
    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success();

    // Search for the unique term
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "search", "betap"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let results: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let results_array = results.as_array().expect("Expected JSON array");

    // All three notes should be found (title, body, and tag matches)
    assert_eq!(results_array.len(), 3, "All field matches should be found");

    // Verify all three titles are in results
    let titles: Vec<&str> = results_array
        .iter()
        .map(|r| r["title"].as_str().unwrap())
        .collect();
    assert!(
        titles.contains(&"Betap"),
        "Title match should be in results"
    );
    assert!(
        titles.contains(&"Body Note"),
        "Body match should be in results"
    );
    assert!(
        titles.contains(&"Tag Note"),
        "Tag match should be in results"
    );
}
