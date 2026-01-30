use crate::support::{qipu, setup_test_dir};
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
fn test_version_consistency() {
    let cargo_toml_content = std::fs::read_to_string("Cargo.toml").unwrap();
    let cargo_version: toml::Value = cargo_toml_content.parse().unwrap();
    let expected_version = cargo_version["package"]["version"].as_str().unwrap();

    let output = qipu()
        .arg("--version")
        .output()
        .expect("Failed to execute qipu --version");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected_output = format!("qipu {}\n", expected_version);

    assert_eq!(
        stdout, expected_output,
        "Version output does not match Cargo.toml"
    );

    let output = std::process::Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .output();

    if let Ok(git_output) = output {
        if git_output.status.success() {
            let git_tag = String::from_utf8(git_output.stdout).unwrap();
            let git_version = git_tag.trim().strip_prefix('v').unwrap_or(git_tag.trim());

            let head = std::process::Command::new("git")
                .args(["rev-parse", "HEAD"])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string());

            let tag_commit = std::process::Command::new("git")
                .args(["rev-parse", git_tag.trim()])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string());

            if let (Some(head_commit), Some(tag_commit_ref)) = (head, tag_commit) {
                if head_commit == tag_commit_ref {
                    assert_eq!(
                        expected_version, git_version,
                        "At release tag {}, but Cargo.toml has version {}. Update Cargo.toml to match git tag before releasing.",
                        git_tag.trim(), expected_version
                    );
                }
            }
        }
    }
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
fn test_unknown_argument_json_equals_format_usage_error() {
    qipu()
        .args(["--format=json", "list", "--bogus-flag"]) // parse/usage error
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
fn test_duplicate_format_equals_syntax() {
    qipu()
        .args(["--format=json", "--format=human", "list"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("\"type\":\"duplicate_format\""));
}

#[test]
fn test_duplicate_format_mixed_syntax() {
    qipu()
        .args(["--format", "json", "--format=human", "list"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("\"type\":\"duplicate_format\""));
}

#[test]
fn test_duplicate_format_human_output() {
    qipu()
        .args(["--format", "json", "--format", "human", "list"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "--format may only be specified once",
        ));
}

#[test]
fn test_duplicate_format_after_command() {
    qipu()
        .args(["list", "--format", "json", "--format", "human"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("\"type\":\"duplicate_format\""));
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
fn test_unknown_command_json_equals_format_usage_error() {
    qipu()
        .args(["--format=json", "nonexistent"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("\"type\":\"usage_error\""));
}

#[test]
fn test_missing_store_exit_code_3() {
    let dir = tempdir().unwrap();
    // Use QIPU_STORE to prevent discovery of /tmp/.qipu from other tests
    let nonexistent_store = dir.path().join("nonexistent-store");
    qipu()
        .current_dir(dir.path())
        .env("QIPU_STORE", &nonexistent_store)
        .arg("list")
        .assert()
        .code(3)
        .stderr(predicate::str::contains("store not found"));
}

// ============================================================================
// JSON format parse error tests
// ============================================================================

#[test]
fn test_missing_required_arg_json_format() {
    let dir = setup_test_dir();

    // Missing required argument (e.g., note ID for link tree)
    qipu()
        .current_dir(dir.path())
        .args(["--format=json", "link", "tree"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("\"type\":\"usage_error\""));
}

#[test]
fn test_invalid_value_json_format() {
    let dir = setup_test_dir();

    // Invalid value for a flag
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "list", "--min-value", "invalid"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("\"type\":\"usage_error\""));
}

// ============================================================================
// Global flags tests
// ============================================================================

#[test]
fn test_quiet_flag() {
    let dir = tempdir().unwrap();
    // Use QIPU_STORE to prevent discovery of /tmp/.qipu from other tests
    let nonexistent_store = dir.path().join("nonexistent-store");

    // With --quiet, error output should be suppressed
    qipu()
        .current_dir(dir.path())
        .env("QIPU_STORE", &nonexistent_store)
        .args(["--quiet", "list"])
        .assert()
        .code(3)
        .stderr(predicate::str::is_empty()); // Error suppressed in quiet mode
}

#[test]
fn test_verbose_flag() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["--verbose", "list"])
        .assert()
        .success()
        .stderr(predicate::str::contains("discover_store"));
}

// ============================================================================
// Argument validation tests (exit code 2 for usage errors)
// ============================================================================

#[test]
fn test_invalid_since_date_exit_code_2() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["list", "--since", "not-a-date"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("invalid --since date"));
}

#[test]
fn test_invalid_direction_exit_code_2() {
    let dir = setup_test_dir();

    // Create a note to link
    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", "test-note", "--direction", "invalid"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("invalid --direction"));
}
