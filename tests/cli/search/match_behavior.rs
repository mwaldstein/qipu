use crate::support::{qipu, setup_test_dir};
use predicates::prelude::*;
use std::fs;

#[test]
fn test_search_title_only_match() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Xylophone"])
        .assert()
        .success();

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

    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["search", "xylophone"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Xylophone"));
}

#[test]
fn test_search_title_only_match_with_body_matches() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Xylophone Musical"])
        .assert()
        .success();

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
        let mut content = fs::read_to_string(&note_path).unwrap();

        if content.contains("title: Xylophone Musical") {
            content.push_str("\nThis note discusses instruments but avoids using the keyword.");
            fs::write(&note_path, content).unwrap();
        } else if content.contains("title: Generic Note") {
            content.push_str("\nThis note contains information about musical instruments.");
            fs::write(&note_path, content).unwrap();
        }
    }

    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "search", "musical"])
        .output()
        .unwrap()
        .stdout;

    let stdout = String::from_utf8_lossy(&output);
    let results: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let results_array = results.as_array().expect("Expected JSON array");

    assert_eq!(
        results_array.len(),
        2,
        "Both title-only match and body match should be returned"
    );

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
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Rust Concepts"])
        .assert()
        .success();

    let notes_dir = dir.path().join(".qipu/notes");
    let note_files: Vec<_> = fs::read_dir(&notes_dir)
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(note_files.len(), 1);
    let note_path = note_files[0].path();
    let mut content = fs::read_to_string(&note_path).unwrap();
    content.push_str("\nThis note discusses the rust language and programming concepts.");
    fs::write(&note_path, content).unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["search", "rust programming"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust Concepts"));
}
