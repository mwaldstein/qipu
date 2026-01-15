use crate::cli::support::qipu;
use predicates::prelude::*;
use tempfile::tempdir;

// ============================================================================
// Setup command tests (Phase 6.3)
// ============================================================================

#[test]
fn test_setup_list() {
    // Test human format
    qipu()
        .args(["setup", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Available integrations:"))
        .stdout(predicate::str::contains("agents-md"));
}

#[test]
fn test_setup_list_json() {
    let output = qipu()
        .args(["setup", "--list", "--format", "json"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(json.is_array());
    assert_eq!(json[0]["name"], "agents-md");
    assert_eq!(json[0]["status"], "available");
}

#[test]
fn test_setup_list_records() {
    qipu()
        .args(["setup", "--list", "--format", "records"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "H qipu=1 records=1 mode=setup.list",
        ))
        .stdout(predicate::str::contains("D integration name=agents-md"));
}

#[test]
fn test_setup_print() {
    qipu()
        .args(["setup", "--print"])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Qipu Agent Integration"))
        .stdout(predicate::str::contains("## Quick Start"))
        .stdout(predicate::str::contains("qipu prime"));
}

#[test]
fn test_setup_print_json() {
    let output = qipu()
        .args(["setup", "--print", "--format", "json"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["integration"], "agents-md");
    assert!(json["content"]
        .as_str()
        .unwrap()
        .contains("Qipu Agent Integration"));
}

#[test]
fn test_setup_install() {
    let dir = tempdir().unwrap();

    // Install
    qipu()
        .current_dir(dir.path())
        .args(["setup", "agents-md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created AGENTS.md"));

    // Verify file was created
    assert!(dir.path().join("AGENTS.md").exists());

    // Verify content
    let content = std::fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
    assert!(content.contains("# Qipu Agent Integration"));
}

#[test]
fn test_setup_install_already_exists() {
    let dir = tempdir().unwrap();

    // Create AGENTS.md first
    std::fs::write(dir.path().join("AGENTS.md"), "existing content").unwrap();

    // Try to install again
    qipu()
        .current_dir(dir.path())
        .args(["setup", "agents-md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("AGENTS.md already exists"));

    // Verify original content is preserved
    let content = std::fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
    assert_eq!(content, "existing content");
}

#[test]
fn test_setup_check_installed() {
    let dir = tempdir().unwrap();

    // Install first
    qipu()
        .current_dir(dir.path())
        .args(["setup", "agents-md"])
        .assert()
        .success();

    // Check
    qipu()
        .current_dir(dir.path())
        .args(["setup", "agents-md", "--check"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "AGENTS.md integration is installed",
        ));
}

#[test]
fn test_setup_check_not_installed() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["setup", "agents-md", "--check"])
        .assert()
        .success()
        .stdout(predicate::str::contains("is not installed"));
}

#[test]
fn test_setup_remove() {
    let dir = tempdir().unwrap();

    // Install first
    qipu()
        .current_dir(dir.path())
        .args(["setup", "agents-md"])
        .assert()
        .success();

    assert!(dir.path().join("AGENTS.md").exists());

    // Remove
    qipu()
        .current_dir(dir.path())
        .args(["setup", "agents-md", "--remove"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed AGENTS.md"));

    // Verify file was removed
    assert!(!dir.path().join("AGENTS.md").exists());
}

#[test]
fn test_setup_remove_not_exists() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["setup", "agents-md", "--remove"])
        .assert()
        .success()
        .stdout(predicate::str::contains("does not exist"));
}

#[test]
fn test_setup_unknown_integration() {
    qipu()
        .args(["setup", "unknown-tool"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Unknown integration"));
}

#[test]
fn test_setup_no_args_usage_error() {
    qipu()
        .arg("setup")
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "Specify --list, --print, or provide a tool name",
        ));
}
