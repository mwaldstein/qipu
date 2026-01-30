use crate::support::qipu;
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
        .stdout(predicate::str::contains("\"path\":"))
        .stdout(predicate::str::contains("\"relevance\":"));
}

#[test]
fn test_search_title_only_match() {
    // Regression test for title-only matches being correctly indexed
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

// Removed test_search_title_match_ranks_above_body_match
// BM25 column weights (2.0/1.5/1.0) provide multiplicative weighting but do not
// guarantee strict ordering (title match > body match). The additive boost test was
// testing buggy behavior. With pure BM25 weights, ordering depends on term frequency,
// document length, and other factors - not just field weights.

// Removed test_search_exact_tag_match_ranks_above_body
// BM25 column weights (2.0/1.5/1.0) provide multiplicative weighting but do not
// guarantee strict ordering (tag match > body match). The additive boost test was
// testing buggy behavior where +3.0 tag boost was distorting rankings. With pure BM25
// weights, ordering depends on term frequency, document length, and other factors.

#[test]
fn test_search_title_only_match_with_body_matches() {
    // Regression test: ensure title-only matches are found alongside body matches
    // when using SQLite FTS5 search
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create note 1: unique word ONLY in title
    qipu()
        .current_dir(dir.path())
        .args(["create", "Xylophone Musical"])
        .assert()
        .success();

    // Create note 2: unique word in body only
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

        if content.contains("title: Xylophone Musical") {
            // Xylophone Musical: word "musical" only in title, NOT in body
            let mut modified = content.clone();
            modified.push_str("\nThis note discusses instruments but avoids using the keyword.");
            fs::write(&note_path, modified).unwrap();
        } else if content.contains("title: Generic Note") {
            // Generic Note: word "musical" in body
            let mut modified = content.clone();
            modified.push_str("\nThis note contains information about musical instruments.");
            fs::write(&note_path, modified).unwrap();
        }
    }

    // Rebuild index
    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success();

    // Search for "musical" which appears in:
    // - Xylophone Musical title (FTS5 will find it via title field)
    // - Generic Note body (FTS5 will find it via body field)
    // Both should appear in results
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "search", "musical"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let results: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let results_array = results.as_array().expect("Expected JSON array");

    // Both notes should be found
    assert_eq!(
        results_array.len(),
        2,
        "Both title-only match and body match should be returned"
    );

    // Verify we got both titles
    let titles: Vec<&str> = results_array
        .iter()
        .map(|r| r["title"].as_str().unwrap())
        .collect();
    assert!(
        titles.contains(&"Xylophone Musical"),
        "Title-only match should be found"
    );
    assert!(
        titles.contains(&"Generic Note"),
        "Body match should be found"
    );
}

#[test]
fn test_search_multi_word_and_semantics() {
    // Test that multi-word queries use AND semantics (terms can appear separately)
    // rather than phrase search (terms must appear together)
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note where "rust" and "programming" appear separately (not as a phrase)
    qipu()
        .current_dir(dir.path())
        .args(["create", "Rust Concepts"])
        .assert()
        .success();

    // Modify note to have words separately
    let notes_dir = dir.path().join(".qipu/notes");
    let note_files: Vec<_> = fs::read_dir(&notes_dir)
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(note_files.len(), 1);
    let note_path = note_files[0].path();
    let mut content = fs::read_to_string(&note_path).unwrap();
    // Words appear separately, not as phrase "rust programming"
    content.push_str("\nThis note discusses the rust language and programming concepts.");
    fs::write(&note_path, content).unwrap();

    // Rebuild index
    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success();

    // Search for "rust programming" - should find the note even though words appear separately
    qipu()
        .current_dir(dir.path())
        .args(["search", "rust programming"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust Concepts"));
}
