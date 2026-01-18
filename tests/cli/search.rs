use crate::cli::support::qipu;
use predicates::prelude::*;
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
