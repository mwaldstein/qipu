use crate::cli::support::qipu;
use predicates::prelude::*;
use tempfile::tempdir;

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
fn test_unknown_argument_json_usage_error() {
    qipu()
        .args(["--format", "json", "list", "--bogus-flag"]) // parse/usage error
        .assert()
        .code(2)
        .stderr(predicate::str::contains("\"type\":\"usage_error\""));
}

#[test]
fn test_duplicate_format_json_usage_error() {
    qipu()
        .args(["--format", "json", "--format", "human", "list"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("\"type\":\"duplicate_format\""));
}

#[test]
fn test_unknown_command_exit_code_2() {
    qipu().arg("nonexistent").assert().code(2);
}

#[test]
fn test_unknown_command_json_usage_error() {
    qipu()
        .args(["--format", "json", "nonexistent"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("\"type\":\"usage_error\""));
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

    // Init at custom location
    qipu()
        .current_dir(dir.path())
        .args(["--store", store_dir.to_str().unwrap(), "init"])
        .assert()
        .success();

    // Verify structure was created under the explicit store path
    assert!(store_dir.join("config.toml").exists());
    assert!(store_dir.join("notes").exists());
    assert!(store_dir.join("mocs").exists());
    assert!(store_dir.join("attachments").exists());
    assert!(store_dir.join("templates").exists());

    // Should be able to use with --store
    qipu()
        .current_dir(dir.path())
        .args(["--store", store_dir.to_str().unwrap(), "list"])
        .assert()
        .success();
}

#[test]
fn test_store_flag_plain_directory_is_invalid() {
    let dir = tempdir().unwrap();
    let store_dir = dir.path().join("not-a-store");
    std::fs::create_dir_all(&store_dir).unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["--store", store_dir.to_str().unwrap(), "list"])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("invalid store"));
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
