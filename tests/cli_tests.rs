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

// ============================================================================
// Link command tests
// ============================================================================

#[test]
fn test_link_list_empty() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note without links
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Isolated Note"])
        .output()
        .unwrap();

    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // First build index
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // List links should show no links
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("No links found"));
}

#[test]
fn test_link_add_and_list() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Source Note"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Target Note"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    // Add a link
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "supports"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added link"));

    // Build index
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // List links from source should show outbound link
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id1])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id2))
        .stdout(predicate::str::contains("supports"));

    // List links from target should show inbound link
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id2])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id1))
        .stdout(predicate::str::contains("supports"));
}

#[test]
fn test_link_add_idempotent() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note A"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note B"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    // Add a link
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added link"));

    // Adding the same link again should report unchanged
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2])
        .assert()
        .success()
        .stdout(predicate::str::contains("already exists"));
}

#[test]
fn test_link_remove() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note A"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note B"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    // Add a link
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "derived-from"])
        .assert()
        .success();

    // Remove the link
    qipu()
        .current_dir(dir.path())
        .args(["link", "remove", &id1, &id2, "--type", "derived-from"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed link"));

    // Build index
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // List links should show no links
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id1])
        .assert()
        .success()
        .stdout(predicate::str::contains("No links found"));
}

#[test]
fn test_link_list_json_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes and link them
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "JSON Source"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "JSON Target"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // List in JSON format
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "link", "list", &id1])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"direction\": \"out\""))
        .stdout(predicate::str::contains("\"source\": \"typed\""));
}

#[test]
fn test_link_list_direction_filter() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes and link them
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Direction Source"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Direction Target"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // List only outbound from source
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id1, "--direction", "out"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id2));

    // List only inbound to source should be empty
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id1, "--direction", "in"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No links found"));

    // List only inbound to target should show the link
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id2, "--direction", "in"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id1));
}

#[test]
fn test_link_list_records_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes and link them
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Records Source"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Records Target"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // List in records format
    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "link", "list", &id1])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "H qipu=1 records=1 mode=link.list",
        ))
        .stdout(predicate::str::contains("E "));
}

// ============================================================================
// Link tree command tests
// ============================================================================

#[test]
fn test_link_tree_single_node() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a single note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Root Note"])
        .output()
        .unwrap();
    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Tree should show just the root
    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id))
        .stdout(predicate::str::contains("Root Note"));
}

#[test]
fn test_link_tree_with_links() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a chain of notes
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Root"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Child 1"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    let output3 = qipu()
        .current_dir(dir.path())
        .args(["create", "Child 2"])
        .output()
        .unwrap();
    let id3 = String::from_utf8_lossy(&output3.stdout).trim().to_string();

    // Link root -> child1 -> child2
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id2, &id3])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Tree from root should show all nodes
    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id1])
        .assert()
        .success()
        .stdout(predicate::str::contains("Root"))
        .stdout(predicate::str::contains("Child 1"))
        .stdout(predicate::str::contains("Child 2"));
}

#[test]
fn test_link_tree_json_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "JSON Root"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "JSON Child"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "link", "tree", &id1])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"root\":"))
        .stdout(predicate::str::contains("\"nodes\":"))
        .stdout(predicate::str::contains("\"edges\":"))
        .stdout(predicate::str::contains("\"spanning_tree\":"));
}

#[test]
fn test_link_tree_records_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Records Root"])
        .output()
        .unwrap();
    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "link", "tree", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "H qipu=1 records=1 mode=link.tree",
        ))
        .stdout(predicate::str::contains("N "));
}

#[test]
fn test_link_tree_max_hops() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a chain of 5 notes
    let mut ids = Vec::new();
    for i in 0..5 {
        let output = qipu()
            .current_dir(dir.path())
            .args(["create", &format!("Node {}", i)])
            .output()
            .unwrap();
        ids.push(String::from_utf8_lossy(&output.stdout).trim().to_string());
    }

    // Link them in a chain
    for i in 0..4 {
        qipu()
            .current_dir(dir.path())
            .args(["link", "add", &ids[i], &ids[i + 1]])
            .assert()
            .success();
    }

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // With max-hops=2, should only see nodes 0, 1, 2
    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &ids[0], "--max-hops", "2"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Node 0"))
        .stdout(predicate::str::contains("Node 1"))
        .stdout(predicate::str::contains("Node 2"))
        .stdout(predicate::str::contains("Node 3").not());
}

#[test]
fn test_link_tree_direction_out() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create 3 notes: A -> B, C -> A
    let output_a = qipu()
        .current_dir(dir.path())
        .args(["create", "Node A"])
        .output()
        .unwrap();
    let id_a = String::from_utf8_lossy(&output_a.stdout).trim().to_string();

    let output_b = qipu()
        .current_dir(dir.path())
        .args(["create", "Node B"])
        .output()
        .unwrap();
    let id_b = String::from_utf8_lossy(&output_b.stdout).trim().to_string();

    let output_c = qipu()
        .current_dir(dir.path())
        .args(["create", "Node C"])
        .output()
        .unwrap();
    let id_c = String::from_utf8_lossy(&output_c.stdout).trim().to_string();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_b])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_c, &id_a])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Tree from A with direction=out should show A -> B but not C
    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id_a, "--direction", "out"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Node A"))
        .stdout(predicate::str::contains("Node B"))
        .stdout(predicate::str::contains("Node C").not());
}

// ============================================================================
// Link path command tests
// ============================================================================

#[test]
fn test_link_path_direct() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Start"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "End"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Find path from start to end
    qipu()
        .current_dir(dir.path())
        .args(["link", "path", &id1, &id2])
        .assert()
        .success()
        .stdout(predicate::str::contains("Start"))
        .stdout(predicate::str::contains("End"))
        .stdout(predicate::str::contains("Path length: 1 hop"));
}

#[test]
fn test_link_path_multi_hop() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create A -> B -> C
    let output_a = qipu()
        .current_dir(dir.path())
        .args(["create", "Node A"])
        .output()
        .unwrap();
    let id_a = String::from_utf8_lossy(&output_a.stdout).trim().to_string();

    let output_b = qipu()
        .current_dir(dir.path())
        .args(["create", "Node B"])
        .output()
        .unwrap();
    let id_b = String::from_utf8_lossy(&output_b.stdout).trim().to_string();

    let output_c = qipu()
        .current_dir(dir.path())
        .args(["create", "Node C"])
        .output()
        .unwrap();
    let id_c = String::from_utf8_lossy(&output_c.stdout).trim().to_string();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_b])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_b, &id_c])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Find path from A to C (2 hops)
    qipu()
        .current_dir(dir.path())
        .args(["link", "path", &id_a, &id_c])
        .assert()
        .success()
        .stdout(predicate::str::contains("Node A"))
        .stdout(predicate::str::contains("Node B"))
        .stdout(predicate::str::contains("Node C"))
        .stdout(predicate::str::contains("Path length: 2 hop"));
}

#[test]
fn test_link_path_not_found() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two unconnected notes
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Isolated 1"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Isolated 2"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Path should not be found
    qipu()
        .current_dir(dir.path())
        .args(["link", "path", &id1, &id2])
        .assert()
        .success()
        .stdout(predicate::str::contains("No path found"));
}

#[test]
fn test_link_path_json_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "JSON Start"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "JSON End"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "link", "path", &id1, &id2])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"from\":"))
        .stdout(predicate::str::contains("\"to\":"))
        .stdout(predicate::str::contains("\"found\": true"))
        .stdout(predicate::str::contains("\"nodes\":"))
        .stdout(predicate::str::contains("\"edges\":"));
}

#[test]
fn test_link_path_records_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Records Start"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Records End"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "link", "path", &id1, &id2])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "H qipu=1 records=1 mode=link.path",
        ))
        .stdout(predicate::str::contains("found=true"));
}

// ============================================================================
// Prime command tests (per specs/llm-context.md)
// ============================================================================

#[test]
fn test_prime_empty_store() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("prime")
        .assert()
        .success()
        .stdout(predicate::str::contains("Qipu Knowledge Store Primer"))
        .stdout(predicate::str::contains("About Qipu"))
        .stdout(predicate::str::contains("Quick Reference"))
        .stdout(predicate::str::contains("qipu list"));
}

#[test]
fn test_prime_with_mocs() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a MOC
    qipu()
        .current_dir(dir.path())
        .args([
            "create",
            "Research Topics",
            "--type",
            "moc",
            "--tag",
            "research",
        ])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("prime")
        .assert()
        .success()
        .stdout(predicate::str::contains("Key Maps of Content"))
        .stdout(predicate::str::contains("Research Topics"));
}

#[test]
fn test_prime_with_recent_notes() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create some notes
    qipu()
        .current_dir(dir.path())
        .args(["create", "First Note", "--type", "fleeting"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Second Note", "--type", "permanent"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("prime")
        .assert()
        .success()
        .stdout(predicate::str::contains("Recently Updated Notes"))
        .stdout(predicate::str::contains("First Note"))
        .stdout(predicate::str::contains("Second Note"));
}

#[test]
fn test_prime_json_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a MOC and a note
    qipu()
        .current_dir(dir.path())
        .args(["create", "Test MOC", "--type", "moc"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note", "--type", "fleeting"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "prime"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"generated_at\""))
        .stdout(predicate::str::contains("\"store\""))
        .stdout(predicate::str::contains("\"primer\""))
        .stdout(predicate::str::contains("\"mocs\""))
        .stdout(predicate::str::contains("\"recent_notes\""))
        .stdout(predicate::str::contains("\"commands\""));
}

#[test]
fn test_prime_records_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a MOC and a note
    qipu()
        .current_dir(dir.path())
        .args(["create", "Test MOC", "--type", "moc"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note", "--type", "fleeting"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "prime"])
        .assert()
        .success()
        .stdout(predicate::str::contains("H qipu=1 records=1 mode=prime"))
        .stdout(predicate::str::contains("D Qipu is"))
        .stdout(predicate::str::contains("C list"))
        .stdout(predicate::str::contains("M ")) // MOC record
        .stdout(predicate::str::contains("N ")); // Note record
}

#[test]
fn test_prime_missing_store() {
    let dir = tempdir().unwrap();

    // No init - should fail with exit code 3
    qipu()
        .current_dir(dir.path())
        .arg("prime")
        .assert()
        .code(3)
        .stderr(predicate::str::contains("store not found"));
}
