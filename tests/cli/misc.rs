use crate::cli::support::qipu;
use predicates::prelude::*;
use serde_json::Value;
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
    qipu()
        .current_dir(&subdir)
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
        .stdout(predicate::str::contains("discover_store"));
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

// ============================================================================
// JSON stdout cleanliness regression tests
// ============================================================================

#[test]
fn test_json_stdout_init() {
    let dir = tempdir().unwrap();
    let stdout = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "init"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8_lossy(&stdout);
    let json: Value = serde_json::from_str(&json_str).expect("stdout should be valid JSON");
    assert!(json.is_object());
}

#[test]
fn test_json_stdout_create() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let stdout = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "create", "Test note"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8_lossy(&stdout);
    let json: Value = serde_json::from_str(&json_str).expect("stdout should be valid JSON");
    assert!(json.is_object());
}

#[test]
fn test_json_stdout_list() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let stdout = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8_lossy(&stdout);
    let json: Value = serde_json::from_str(&json_str).expect("stdout should be valid JSON");
    assert!(json.is_array());
}

#[test]
fn test_json_stdout_show() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let stdout = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "create", "Test note"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8_lossy(&stdout);
    let json: Value = serde_json::from_str(&json_str).expect("stdout should be valid JSON");
    let note_id = json["id"].as_str().expect("note id should exist");

    let stdout = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", note_id])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8_lossy(&stdout);
    let json: Value = serde_json::from_str(&json_str).expect("stdout should be valid JSON");
    assert!(json.is_object());
}

#[test]
fn test_json_stdout_search() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test note about rust"])
        .assert()
        .success();

    let stdout = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "search", "rust"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8_lossy(&stdout);
    let json: Value = serde_json::from_str(&json_str).expect("stdout should be valid JSON");
    assert!(json.is_object() || json.is_array());
}

#[test]
fn test_json_stdout_context() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let stdout = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "create", "Test note"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8_lossy(&stdout);
    let json: Value = serde_json::from_str(&json_str).expect("stdout should be valid JSON");
    let note_id = json["id"].as_str().expect("note id should exist");

    let stdout = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "context", "--note", note_id])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8_lossy(&stdout);
    let json: Value = serde_json::from_str(&json_str).expect("stdout should be valid JSON");
    assert!(json.is_object());
}

#[test]
fn test_json_stdout_value_set() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let stdout = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "create", "Test note"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8_lossy(&stdout);
    let json: Value = serde_json::from_str(&json_str).expect("stdout should be valid JSON");
    let note_id = json["id"].as_str().expect("note id should exist");

    let stdout = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "value", "set", note_id, "75"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8_lossy(&stdout);
    let json: Value = serde_json::from_str(&json_str).expect("stdout should be valid JSON");
    assert!(json.is_object());
}

#[test]
fn test_json_stdout_value_show() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let stdout = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "create", "Test note"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8_lossy(&stdout);
    let json: Value = serde_json::from_str(&json_str).expect("stdout should be valid JSON");
    let note_id = json["id"].as_str().expect("note id should exist");

    let stdout = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "value", "show", note_id])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8_lossy(&stdout);
    let json: Value = serde_json::from_str(&json_str).expect("stdout should be valid JSON");
    assert!(json.is_object());
}

#[test]
fn test_json_stdout_custom_set() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let stdout = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "create", "Test note"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8_lossy(&stdout);
    let json: Value = serde_json::from_str(&json_str).expect("stdout should be valid JSON");
    let note_id = json["id"].as_str().expect("note id should exist");

    let stdout = qipu()
        .current_dir(dir.path())
        .args([
            "--format", "json", "custom", "set", note_id, "priority", "high",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8_lossy(&stdout);
    let json: Value = serde_json::from_str(&json_str).expect("stdout should be valid JSON");
    assert!(json.is_object());
}

#[test]
fn test_json_stdout_custom_get() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let stdout = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "create", "Test note"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8_lossy(&stdout);
    let json: Value = serde_json::from_str(&json_str).expect("stdout should be valid JSON");
    let note_id = json["id"].as_str().expect("note id should exist");

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", note_id, "priority", "high"])
        .assert()
        .success();

    let stdout = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "custom", "get", note_id, "priority"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8_lossy(&stdout);
    let json: Value = serde_json::from_str(&json_str).expect("stdout should be valid JSON");
    assert!(json.is_object());
}

#[test]
fn test_json_stdout_custom_show() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let stdout = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "create", "Test note"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8_lossy(&stdout);
    let json: Value = serde_json::from_str(&json_str).expect("stdout should be valid JSON");
    let note_id = json["id"].as_str().expect("note id should exist");

    let stdout = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "custom", "show", note_id])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8_lossy(&stdout);
    let json: Value = serde_json::from_str(&json_str).expect("stdout should be valid JSON");
    assert!(json.is_object());
}

#[test]
fn test_json_stdout_custom_unset() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let stdout = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "create", "Test note"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8_lossy(&stdout);
    let json: Value = serde_json::from_str(&json_str).expect("stdout should be valid JSON");
    let note_id = json["id"].as_str().expect("note id should exist");

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", note_id, "priority", "high"])
        .assert()
        .success();

    let stdout = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "custom", "unset", note_id, "priority"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8_lossy(&stdout);
    let json: Value = serde_json::from_str(&json_str).expect("stdout should be valid JSON");
    assert!(json.is_object());
}

#[test]
fn test_json_stdout_capture() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let stdout = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "capture", "--title", "Captured note"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8_lossy(&stdout);
    let json: Value = serde_json::from_str(&json_str).expect("stdout should be valid JSON");
    assert!(json.is_object());
}

#[test]
fn test_json_stdout_index() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let stdout = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "index"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8_lossy(&stdout);
    let json: Value = serde_json::from_str(&json_str).expect("stdout should be valid JSON");
    assert!(json.is_object());
}

#[test]
fn test_json_stdout_doctor() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let stdout = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "doctor"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8_lossy(&stdout);
    let json: Value = serde_json::from_str(&json_str).expect("stdout should be valid JSON");
    assert!(json.is_object());
}

#[test]
fn test_json_stdout_export() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let stdout = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "create", "Test note"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8_lossy(&stdout);
    let json: Value = serde_json::from_str(&json_str).expect("stdout should be valid JSON");
    let note_id = json["id"].as_str().expect("note id should exist");

    let stdout = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "export", "--note", note_id])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8_lossy(&stdout);
    let json: Value = serde_json::from_str(&json_str).expect("stdout should be valid JSON");
    assert!(json.is_object());
}
