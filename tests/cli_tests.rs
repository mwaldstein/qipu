//! Integration tests for qipu CLI
//!
//! These tests run the qipu binary and verify correct behavior per spec.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

/// Get a Command for qipu
fn qipu() -> Command {
    Command::cargo_bin("qipu").unwrap()
}

// ============================================================================
// Help and Version tests (per specs/cli-tool.md)
// ============================================================================

#[test]
fn test_help_flag() {
    qipu()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: qipu"))
        .stdout(predicate::str::contains("Commands:"))
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("list"));
}

#[test]
fn test_version_flag() {
    qipu()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("qipu"));
}

#[test]
fn test_subcommand_help() {
    qipu()
        .args(["create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Create a new note"));
}

// ============================================================================
// Exit code tests (per specs/cli-tool.md)
// ============================================================================

#[test]
fn test_unknown_format_exit_code_2() {
    qipu()
        .args(["--format", "invalid", "list"])
        .assert()
        .code(2);
}

#[test]
fn test_unknown_command_exit_code_2() {
    qipu().arg("nonexistent").assert().code(2);
}

#[test]
fn test_missing_store_exit_code_3() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("list")
        .assert()
        .code(3)
        .stderr(predicate::str::contains("store not found"));
}

// ============================================================================
// Init command tests
// ============================================================================

#[test]
fn test_init_creates_store() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized qipu store"));

    // Verify structure was created
    assert!(dir.path().join(".qipu").exists());
    assert!(dir.path().join(".qipu/notes").exists());
    assert!(dir.path().join(".qipu/mocs").exists());
    assert!(dir.path().join(".qipu/templates").exists());
    assert!(dir.path().join(".qipu/config.toml").exists());
}

#[test]
fn test_init_idempotent() {
    let dir = tempdir().unwrap();

    // First init
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Second init should also succeed (idempotent)
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();
}

#[test]
fn test_init_visible() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["init", "--visible"])
        .assert()
        .success();

    // Should create visible directory
    assert!(dir.path().join("qipu").exists());
    assert!(!dir.path().join(".qipu").exists());
}

#[test]
fn test_init_json_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "init"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"ok\""))
        .stdout(predicate::str::contains("\"store\""));
}

// ============================================================================
// Create command tests
// ============================================================================

#[test]
fn test_create_note() {
    let dir = tempdir().unwrap();

    // Init first
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create note
    qipu()
        .current_dir(dir.path())
        .args(["create", "My Test Note"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));
}

#[test]
fn test_create_with_type() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "permanent", "Permanent Note"])
        .assert()
        .success();
}

#[test]
fn test_create_with_tags() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--tag", "test", "--tag", "demo", "Tagged Note"])
        .assert()
        .success();
}

#[test]
fn test_create_json_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "create", "JSON Note"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\":"))
        .stdout(predicate::str::contains("\"title\": \"JSON Note\""));
}

#[test]
fn test_new_alias() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // 'new' should work like 'create'
    qipu()
        .current_dir(dir.path())
        .args(["new", "Alias Test"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));
}

// ============================================================================
// List command tests
// ============================================================================

#[test]
fn test_list_empty_store() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No notes found"));
}

#[test]
fn test_list_with_notes() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note
    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success();

    // List should show it
    qipu()
        .current_dir(dir.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Test Note"))
        .stdout(predicate::str::contains("qp-"));
}

#[test]
fn test_list_json_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "JSON List Test"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"title\": \"JSON List Test\""));
}

#[test]
fn test_list_filter_by_type() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create notes of different types
    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "fleeting", "Fleeting Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "permanent", "Permanent Note"])
        .assert()
        .success();

    // Filter by type
    qipu()
        .current_dir(dir.path())
        .args(["list", "--type", "fleeting"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Fleeting Note"))
        .stdout(predicate::str::contains("Permanent Note").not());
}

// ============================================================================
// Show command tests
// ============================================================================

#[test]
fn test_show_note() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create and capture ID
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Show Test"])
        .output()
        .unwrap();

    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Show should display the note
    qipu()
        .current_dir(dir.path())
        .args(["show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Show Test"))
        .stdout(predicate::str::contains(&id));
}

#[test]
fn test_show_nonexistent() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["show", "qp-nonexistent"])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("not found"));
}

// ============================================================================
// Inbox command tests
// ============================================================================

#[test]
fn test_inbox_empty() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("inbox")
        .assert()
        .success()
        .stdout(predicate::str::contains("Inbox is empty"));
}

#[test]
fn test_inbox_shows_fleeting() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "fleeting", "Inbox Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("inbox")
        .assert()
        .success()
        .stdout(predicate::str::contains("Inbox Note"));
}

#[test]
fn test_inbox_excludes_permanent() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "permanent", "Not In Inbox"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("inbox")
        .assert()
        .success()
        .stdout(predicate::str::contains("Inbox is empty"));
}

// ============================================================================
// Store discovery tests
// ============================================================================

#[test]
fn test_store_discovery_walks_up() {
    let dir = tempdir().unwrap();
    let subdir = dir.path().join("sub/dir/deep");
    std::fs::create_dir_all(&subdir).unwrap();

    // Init at top level
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // List from deep subdir should find store
    qipu().current_dir(&subdir).arg("list").assert().success();
}

#[test]
fn test_explicit_store_path() {
    let dir = tempdir().unwrap();
    let store_dir = dir.path().join("custom-store");
    std::fs::create_dir_all(&store_dir).unwrap();

    // Init at custom location
    qipu()
        .current_dir(dir.path())
        .args(["--store", store_dir.to_str().unwrap(), "init"])
        .assert()
        .success();

    // Should be able to use with --store
    qipu()
        .current_dir(dir.path())
        .args(["--store", store_dir.to_str().unwrap(), "list"])
        .assert()
        .success();
}

// ============================================================================
// Global flags tests
// ============================================================================

#[test]
fn test_quiet_flag() {
    let dir = tempdir().unwrap();

    // With --quiet, error output should be suppressed
    qipu()
        .current_dir(dir.path())
        .args(["--quiet", "list"])
        .assert()
        .code(3)
        .stderr(predicate::str::is_empty()); // Error suppressed in quiet mode
}

#[test]
fn test_verbose_flag() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--verbose", "list"])
        .assert()
        .success()
        .stderr(predicate::str::contains("discover_store"));
}

// ============================================================================
// Index command tests
// ============================================================================

#[test]
fn test_index_empty_store() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success()
        .stdout(predicate::str::contains("Indexed 0 notes"));
}

#[test]
fn test_index_with_notes() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create notes
    qipu()
        .current_dir(dir.path())
        .args(["create", "--tag", "test", "Note 1"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--tag", "test", "Note 2"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success()
        .stdout(predicate::str::contains("Indexed 2 notes"));
}

#[test]
fn test_index_json_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "index"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"ok\""))
        .stdout(predicate::str::contains("\"notes_indexed\":"));
}

#[test]
fn test_index_rebuild() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success();

    // First index
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Rebuild should also work
    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Indexed 1 notes"));
}

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
