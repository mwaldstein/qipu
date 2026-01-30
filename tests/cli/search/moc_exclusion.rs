use crate::support::{extract_id, qipu};
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_search_exclude_mocs() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a MOC note
    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "moc", "Programming MOC"])
        .assert()
        .success();

    // Create a permanent note
    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "permanent", "Programming Concepts"])
        .assert()
        .success();

    // Create a fleeting note
    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "fleeting", "Programming Ideas"])
        .assert()
        .success();

    // Search without exclude_mocs should return all notes
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "search", "programming"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let results: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let results_array = results.as_array().expect("Expected JSON array");

    assert_eq!(
        results_array.len(),
        3,
        "Should find all three notes without --exclude-mocs"
    );

    // Search with --exclude-mocs should exclude MOC notes
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "json",
            "search",
            "--exclude-mocs",
            "programming",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let results: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let results_array = results.as_array().expect("Expected JSON array");

    assert_eq!(
        results_array.len(),
        2,
        "Should find only non-MOC notes with --exclude-mocs"
    );

    // Verify no MOC notes in results
    for result in results_array.iter() {
        let note_type = result["type"].as_str().unwrap();
        assert_ne!(note_type, "moc", "MOC notes should be excluded");
    }

    // Verify we got the expected notes
    let titles: Vec<&str> = results_array
        .iter()
        .map(|r| r["title"].as_str().unwrap())
        .collect();
    assert!(titles.contains(&"Programming Concepts"));
    assert!(titles.contains(&"Programming Ideas"));
}

#[test]
fn test_search_exclude_mocs_no_results() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create only MOC notes
    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "moc", "Programming MOC"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "moc", "Programming Index"])
        .assert()
        .success();

    // Search with --exclude-mocs should return no results
    qipu()
        .current_dir(dir.path())
        .args(["search", "--exclude-mocs", "programming"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No results found"));
}

#[test]
fn test_search_exclude_mocs_with_filters() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create MOC with tag
    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "moc", "--tag", "rust", "Rust MOC"])
        .assert()
        .success();

    // Create permanent note with tag
    qipu()
        .current_dir(dir.path())
        .args([
            "create",
            "--type",
            "permanent",
            "--tag",
            "rust",
            "Rust Concepts",
        ])
        .assert()
        .success();

    // Create permanent note without tag
    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "permanent", "General Programming"])
        .assert()
        .success();

    // Search with --tag and --exclude-mocs
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "json",
            "search",
            "--tag",
            "rust",
            "--exclude-mocs",
            "rust",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let results: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let results_array = results.as_array().expect("Expected JSON array");

    assert_eq!(
        results_array.len(),
        1,
        "Should find only permanent note with rust tag"
    );

    assert_eq!(results_array[0]["title"].as_str().unwrap(), "Rust Concepts");
    assert_eq!(results_array[0]["type"].as_str().unwrap(), "permanent");
}

#[test]
fn test_search_exclude_mocs_with_min_value() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create MOC with high value
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "moc", "High Value MOC"])
        .output()
        .unwrap();
    let moc_id = extract_id(&output);

    // Create permanent note with high value
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "permanent", "High Value Permanent"])
        .output()
        .unwrap();
    let perm_id = extract_id(&output);

    // Create permanent note with low value
    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "permanent", "Low Value Permanent"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &moc_id, "90"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &perm_id, "85"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Search with --min-value and --exclude-mocs
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "json",
            "search",
            "--min-value",
            "80",
            "--exclude-mocs",
            "value",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let results: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let results_array = results.as_array().expect("Expected JSON array");

    assert_eq!(
        results_array.len(),
        1,
        "Should find only permanent note with value >= 80"
    );

    assert_eq!(
        results_array[0]["title"].as_str().unwrap(),
        "High Value Permanent"
    );
    assert_eq!(results_array[0]["type"].as_str().unwrap(), "permanent");
}
