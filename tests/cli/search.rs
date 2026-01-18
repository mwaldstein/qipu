use crate::cli::support::qipu;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

// ============================================================================
// Search command tests
// ============================================================================

#[test]
fn test_search_empty_store() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["search", "test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No results found"));
}

#[test]
fn test_search_finds_title() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Knowledge Management"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["search", "knowledge"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Knowledge Management"));
}

#[test]
fn test_search_by_tag() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--tag", "rust", "Rust Programming"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Other Note"])
        .assert()
        .success();

    // Search with tag filter
    qipu()
        .current_dir(dir.path())
        .args(["search", "--tag", "rust", "programming"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust Programming"))
        .stdout(predicate::str::contains("Other Note").not());
}

#[test]
fn test_search_by_type() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "permanent", "Permanent Idea"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "fleeting", "Fleeting Idea"])
        .assert()
        .success();

    // Search with type filter
    qipu()
        .current_dir(dir.path())
        .args(["search", "--type", "permanent", "idea"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Permanent Idea"))
        .stdout(predicate::str::contains("Fleeting Idea").not());
}

#[test]
fn test_search_json_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Search Test Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "search", "test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"title\": \"Search Test Note\""))
        .stdout(predicate::str::contains("\"relevance\":"));
}

#[test]
fn test_search_title_only_match() {
    // Regression test for title-only matches being missed by ripgrep
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note with a unique word in the title but NOT in the body
    qipu()
        .current_dir(dir.path())
        .args(["create", "Xylophone"])
        .assert()
        .success();

    // Add body content that does NOT contain the title word
    let notes_dir = dir.path().join(".qipu/notes");
    let note_files: Vec<_> = std::fs::read_dir(&notes_dir)
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(note_files.len(), 1);
    let note_path = note_files[0].path();
    let mut content = std::fs::read_to_string(&note_path).unwrap();
    content.push_str("\nThis is some body content without the search term.");
    std::fs::write(&note_path, content).unwrap();

    // Rebuild index to pick up the changes
    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success();

    // Search for the title word - it should be found even though it's not in the body
    qipu()
        .current_dir(dir.path())
        .args(["search", "xylophone"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Xylophone"));
}

#[test]
fn test_search_recency_boost() {
    // Test that recently updated notes rank higher than older notes with similar content
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes with identical content
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

    // Add the same body content to both notes
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

    // Add the "updated" field to notes - one recent (within 7 days), one old (over 90 days ago)
    for note_file in &note_files {
        let note_path = note_file.path();
        let content = fs::read_to_string(&note_path).unwrap();

        // Find the line with "created:" and add "updated:" on the next line
        let lines: Vec<&str> = content.lines().collect();
        let mut new_lines = Vec::new();

        for line in lines {
            new_lines.push(line.to_string());
            if line.starts_with("created:") {
                // Add updated field right after created
                let updated_date = if content.contains("title: Old Note") {
                    // Set updated to 100 days ago
                    chrono::Utc::now() - chrono::Duration::days(100)
                } else {
                    // Set updated to today (recent)
                    chrono::Utc::now()
                };
                new_lines.push(format!("updated: {}", updated_date.to_rfc3339()));
            }
        }

        let modified_content = new_lines.join("\n");
        fs::write(&note_path, modified_content).unwrap();
    }

    // Rebuild index to pick up the changes
    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success();

    // Search with JSON output to check relevance scores
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

    // Both notes should be found
    assert_eq!(results_array.len(), 2);

    // The first result should be "New Note" (more recent, higher relevance)
    let first_result = &results_array[0];
    let first_title = first_result["title"].as_str().unwrap();
    assert_eq!(
        first_title, "New Note",
        "Recently updated note should rank first"
    );

    // Verify that the first result has higher relevance than the second
    let first_relevance = first_result["relevance"].as_f64().unwrap();
    let second_relevance = results_array[1]["relevance"].as_f64().unwrap();
    assert!(
        first_relevance > second_relevance,
        "Recent note ({}) should have higher relevance than old note ({})",
        first_relevance,
        second_relevance
    );
}

#[test]
fn test_search_title_match_ranks_above_body_match() {
    // Test that title matches rank higher than body matches (2.0x boost vs 1.0x base)
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create note with unique word in title only
    qipu()
        .current_dir(dir.path())
        .args(["create", "Xylophone Guide"])
        .assert()
        .success();

    // Create note with unique word in body only
    qipu()
        .current_dir(dir.path())
        .args(["create", "Music Notes"])
        .assert()
        .success();

    // Modify notes to ensure search term appears only in the specified field
    let notes_dir = dir.path().join(".qipu/notes");
    let note_files: Vec<_> = fs::read_dir(&notes_dir)
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(note_files.len(), 2);

    for note_file in &note_files {
        let note_path = note_file.path();
        let content = fs::read_to_string(&note_path).unwrap();

        if content.contains("title: Xylophone Guide") {
            // Replace body with content that does NOT contain "xylophone"
            // Keep header intact, change only the body
            let lines: Vec<&str> = content.lines().collect();
            let mut new_lines = Vec::new();
            let mut found_empty_line = false;

            for line in &lines {
                new_lines.push(line.to_string());
                if line.is_empty() {
                    found_empty_line = true;
                    break;
                }
            }

            if found_empty_line {
                new_lines
                    .push("This document provides guidance on musical instruments.".to_string());
            }
            fs::write(&note_path, new_lines.join("\n")).unwrap();
        } else if content.contains("title: Music Notes") {
            // Ensure "xylophone" only appears in body, not title
            // Replace body to include "xylophone"
            let lines: Vec<&str> = content.lines().collect();
            let mut new_lines = Vec::new();
            let mut found_empty_line = false;

            for line in &lines {
                new_lines.push(line.to_string());
                if line.is_empty() {
                    found_empty_line = true;
                    break;
                }
            }

            if found_empty_line {
                new_lines.push(
                    "The xylophone is a percussion instrument consisting of wooden bars."
                        .to_string(),
                );
            }
            fs::write(&note_path, new_lines.join("\n")).unwrap();
        }
    }

    // Debug: print note contents to verify
    for note_file in &note_files {
        let note_path = note_file.path();
        let content = fs::read_to_string(&note_path).unwrap();
        eprintln!("Note file: {:?}", note_path);
        eprintln!("Content:\n{}\n---\n", content);
    }

    // Rebuild index to pick up the changes
    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success();

    // Search for "xylophone" - title match should rank above body match
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "search", "xylophone"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let results: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let results_array = results.as_array().expect("Expected JSON array");

    // Both notes should be found (one from title metadata, one from body search)
    assert_eq!(
        results_array.len(),
        2,
        "Expected 2 results: one from title match, one from body match"
    );

    // The first result should be "Xylophone Guide" (title match with 2.0x boost)
    let first_result = &results_array[0];
    let first_title = first_result["title"].as_str().unwrap();
    assert_eq!(
        first_title, "Xylophone Guide",
        "Note with title match should rank first due to 2.0x boost"
    );

    // Verify relevance scores show title boost > body match
    let first_relevance = first_result["relevance"].as_f64().unwrap();
    let second_relevance = results_array[1]["relevance"].as_f64().unwrap();
    assert!(
        first_relevance > second_relevance,
        "Title match ({}) should have higher relevance than body match ({})",
        first_relevance,
        second_relevance
    );
}

#[test]
fn test_search_exact_tag_match_ranks_above_body() {
    // Test that exact tag matches rank above partial matches in body text
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create note with tag "rust" - exact match for query "rust"
    qipu()
        .current_dir(dir.path())
        .args(["create", "--tag", "rust", "Programming Language"])
        .assert()
        .success();

    // Create note with "rust" in the body but NOT in tags (only partial substring match)
    qipu()
        .current_dir(dir.path())
        .args(["create", "--tag", "programming", "Rust Tutorial"])
        .assert()
        .success();

    // Add body content to second note containing "rust" as substring within larger text
    let notes_dir = dir.path().join(".qipu/notes");
    let note_files: Vec<_> = fs::read_dir(&notes_dir)
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(note_files.len(), 2);

    // Find the "Rust Tutorial" note and add body text
    for note_file in &note_files {
        let note_path = note_file.path();
        let content = fs::read_to_string(&note_path).unwrap();
        if content.contains("title: Rust Tutorial") {
            let mut modified = content.clone();
            modified
                .push_str("\nThis tutorial discusses the rustacean community and rusty concepts.");
            fs::write(&note_path, modified).unwrap();
        }
    }

    // Rebuild index
    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success();

    // Search for "rust" - exact tag match should rank higher than body match
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "search", "rust"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let results: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let results_array = results.as_array().expect("Expected JSON array");

    // Both notes should be found
    assert_eq!(results_array.len(), 2);

    // The first result should be "Programming Language" (exact tag match)
    let first_result = &results_array[0];
    let first_title = first_result["title"].as_str().unwrap();
    assert_eq!(
        first_title, "Programming Language",
        "Note with exact tag match should rank first"
    );

    // Verify relevance scores show exact tag match beats body match
    let first_relevance = first_result["relevance"].as_f64().unwrap();
    let second_relevance = results_array[1]["relevance"].as_f64().unwrap();
    assert!(
        first_relevance > second_relevance,
        "Exact tag match ({}) should have higher relevance than body match ({})",
        first_relevance,
        second_relevance
    );
}
