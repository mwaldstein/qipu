use crate::support::{qipu, setup_test_dir};
use std::fs;

#[test]
fn test_search_recency_boost() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Old Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "New Note"])
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
        let mut content = fs::read_to_string(&note_path).unwrap();
        content.push_str("\nThis note contains information about programming.");
        fs::write(&note_path, content).unwrap();
    }

    for note_file in &note_files {
        let note_path = note_file.path();
        let content = fs::read_to_string(&note_path).unwrap();

        let lines: Vec<&str> = content.lines().collect();
        let mut new_lines = Vec::new();

        for line in lines {
            new_lines.push(line.to_string());
            if line.starts_with("created:") {
                let updated_date = if content.contains("title: Old Note") {
                    chrono::Utc::now() - chrono::Duration::days(100)
                } else {
                    chrono::Utc::now()
                };
                new_lines.push(format!("updated: {}", updated_date.to_rfc3339()));
            }
        }

        let modified_content = new_lines.join("\n");
        fs::write(&note_path, modified_content).unwrap();
    }

    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "search", "programming"])
        .output()
        .unwrap()
        .stdout;

    let stdout = String::from_utf8_lossy(&output);
    let results: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let results_array = results.as_array().expect("Expected JSON array");

    assert_eq!(results_array.len(), 2);

    let first_result = &results_array[0];
    let first_title = first_result["title"].as_str().unwrap();
    assert_eq!(
        first_title, "New Note",
        "Recently updated note should rank first"
    );

    let first_relevance = first_result["relevance"].as_f64().unwrap();
    let second_relevance = results_array[1]["relevance"].as_f64().unwrap();
    assert!(
        first_relevance > second_relevance,
        "Recent note ({}) should have higher relevance than old note ({})",
        first_relevance,
        second_relevance
    );
}
