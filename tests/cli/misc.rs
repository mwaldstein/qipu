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
fn test_visible_store_discovery() {
    let dir = tempdir().unwrap();

    // Manually create a visible "qipu/" store structure
    let store_path = dir.path().join("qipu");
    std::fs::create_dir_all(&store_path).unwrap();
    std::fs::create_dir_all(store_path.join("notes")).unwrap();
    std::fs::create_dir_all(store_path.join("mocs")).unwrap();
    std::fs::create_dir_all(store_path.join("attachments")).unwrap();
    std::fs::create_dir_all(store_path.join("templates")).unwrap();

    // Create minimal config file
    std::fs::write(store_path.join("config.toml"), "# Qipu configuration\n").unwrap();

    // Should discover the visible "qipu/" store
    qipu()
        .current_dir(dir.path())
        .arg("list")
        .assert()
        .success();
}

#[test]
fn test_hidden_store_preferred_over_visible() {
    let dir = tempdir().unwrap();

    // Create both hidden and visible stores
    let hidden_path = dir.path().join(".qipu");
    let visible_path = dir.path().join("qipu");

    std::fs::create_dir_all(&hidden_path).unwrap();
    std::fs::create_dir_all(hidden_path.join("notes")).unwrap();
    std::fs::create_dir_all(hidden_path.join("mocs")).unwrap();
    std::fs::create_dir_all(hidden_path.join("attachments")).unwrap();
    std::fs::create_dir_all(hidden_path.join("templates")).unwrap();
    std::fs::write(hidden_path.join("config.toml"), "# Hidden config\n").unwrap();

    std::fs::create_dir_all(&visible_path).unwrap();
    std::fs::create_dir_all(visible_path.join("notes")).unwrap();
    std::fs::create_dir_all(visible_path.join("mocs")).unwrap();
    std::fs::create_dir_all(visible_path.join("attachments")).unwrap();
    std::fs::create_dir_all(visible_path.join("templates")).unwrap();
    std::fs::write(visible_path.join("config.toml"), "# Visible config\n").unwrap();

    // The hidden .qipu/ should be preferred over qipu/
    qipu()
        .current_dir(dir.path())
        .arg("list")
        .assert()
        .success();

    // Create a note in hidden store to verify it's being used
    qipu()
        .current_dir(dir.path())
        .args(["create", "Test in hidden store"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));

    // Verify the note was created in the hidden store (not visible)
    let hidden_notes: Vec<_> = std::fs::read_dir(hidden_path.join("notes"))
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    let visible_notes: Vec<_> = std::fs::read_dir(visible_path.join("notes"))
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();

    assert_eq!(hidden_notes.len(), 1);
    assert_eq!(visible_notes.len(), 0);
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
fn test_root_flag_affects_discovery_start_dir() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("root");
    let subdir = dir.path().join("somewhere/else");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::create_dir_all(&subdir).unwrap();

    // Init at root location
    qipu().current_dir(&root).arg("init").assert().success();

    // From a different directory, --root should allow discovery
    qipu()
        .current_dir(&subdir)
        .args(["--root", root.to_str().unwrap(), "list"])
        .assert()
        .success();

    // Without --root, discovery from subdir should fail
    // Use QIPU_STORE to prevent discovery of /tmp/.qipu from other tests
    let nonexistent_store = dir.path().join("nonexistent-store");
    qipu()
        .current_dir(&subdir)
        .env("QIPU_STORE", &nonexistent_store)
        .arg("list")
        .assert()
        .code(3)
        .stderr(predicate::str::contains("store not found"));
}

#[test]
fn test_discovery_stops_at_project_boundary_with_parent_store() {
    let dir = tempdir().unwrap();
    let parent_dir = dir.path().join("parent");
    let project_dir = dir.path().join("parent/project");
    let project_subdir = project_dir.join("subdir");
    std::fs::create_dir_all(&project_subdir).unwrap();

    // Create parent store
    let parent_store = parent_dir.join(".qipu");
    std::fs::create_dir_all(&parent_store).unwrap();
    std::fs::create_dir_all(parent_store.join("notes")).unwrap();
    std::fs::create_dir_all(parent_store.join("mocs")).unwrap();
    std::fs::create_dir_all(parent_store.join("attachments")).unwrap();
    std::fs::create_dir_all(parent_store.join("templates")).unwrap();
    std::fs::write(parent_store.join("config.toml"), "# Parent store config\n").unwrap();

    // Create project marker (.git) in project directory
    std::fs::create_dir_all(project_dir.join(".git")).unwrap();

    // From project_subdir, should NOT discover parent store
    // Discovery should stop at .git boundary
    // Use QIPU_STORE to prevent discovery of /tmp/.qipu from other tests
    let nonexistent_store = dir.path().join("nonexistent-store");
    qipu()
        .current_dir(&project_subdir)
        .env("QIPU_STORE", &nonexistent_store)
        .arg("list")
        .assert()
        .code(3)
        .stderr(predicate::str::contains("store not found"));
}

#[test]
fn test_discovery_stops_at_cargo_toml_boundary() {
    let dir = tempdir().unwrap();
    let parent_dir = dir.path().join("parent");
    let project_dir = dir.path().join("parent/rust_project");
    std::fs::create_dir_all(&project_dir).unwrap();

    // Create parent store
    let parent_store = parent_dir.join(".qipu");
    std::fs::create_dir_all(&parent_store).unwrap();
    std::fs::create_dir_all(parent_store.join("notes")).unwrap();
    std::fs::create_dir_all(parent_store.join("mocs")).unwrap();
    std::fs::create_dir_all(parent_store.join("attachments")).unwrap();
    std::fs::create_dir_all(parent_store.join("templates")).unwrap();
    std::fs::write(parent_store.join("config.toml"), "# Parent store config\n").unwrap();

    // Create Cargo.toml as project marker
    std::fs::write(
        project_dir.join("Cargo.toml"),
        "[package]\nname = \"test\"\n",
    )
    .unwrap();

    // From project directory, should NOT discover parent store
    // Discovery should stop at Cargo.toml boundary
    // Use QIPU_STORE to prevent discovery of /tmp/.qipu from other tests
    let nonexistent_store = dir.path().join("nonexistent-store");
    qipu()
        .current_dir(&project_dir)
        .env("QIPU_STORE", &nonexistent_store)
        .arg("list")
        .assert()
        .code(3)
        .stderr(predicate::str::contains("store not found"));
}

#[test]
fn test_relative_store_resolved_against_root() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("root");
    let subdir = dir.path().join("somewhere/else");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::create_dir_all(&subdir).unwrap();

    // Create a store in a subdirectory of root
    let store_path = root.join("mystore");

    // Init using relative path from root
    qipu()
        .current_dir(&subdir)
        .args([
            "--root",
            root.to_str().unwrap(),
            "--store",
            "mystore",
            "init",
        ])
        .assert()
        .success();

    // Verify store was created at root/mystore, not subdir/mystore
    assert!(store_path.join("config.toml").exists());
    assert!(!subdir.join("mystore").exists());

    // Should be able to use with relative --store and --root
    qipu()
        .current_dir(&subdir)
        .args([
            "--root",
            root.to_str().unwrap(),
            "--store",
            "mystore",
            "list",
        ])
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
// JSON format parse error tests
// ============================================================================

#[test]
fn test_missing_required_arg_json_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
// Argument validation tests (exit code 2 for usage errors)
// ============================================================================

#[test]
fn test_invalid_since_date_exit_code_2() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["list", "--since", "not-a-date"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("invalid --since date"));
}

#[test]
fn test_invalid_direction_exit_code_2() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
