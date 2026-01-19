use crate::cli::support::qipu;
use predicates::prelude::*;
use tempfile::tempdir;

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
// Stealth mode tests
// ============================================================================

#[test]
fn test_init_stealth_adds_to_project_gitignore() {
    let dir = tempdir().unwrap();

    // Create a .gitignore with some existing content
    let gitignore_path = dir.path().join(".gitignore");
    std::fs::write(&gitignore_path, "*.log\n*.tmp\n").unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["init", "--stealth"])
        .assert()
        .success();

    // Verify store was created
    assert!(dir.path().join(".qipu").exists());

    // Verify .gitignore has the .qipu/ entry with trailing slash
    let gitignore_content = std::fs::read_to_string(&gitignore_path).unwrap();
    assert!(gitignore_content.contains(".qipu/"));
    assert!(gitignore_content.contains("*.log")); // Original content preserved
    assert!(gitignore_content.contains("*.tmp"));
}

#[test]
fn test_init_stealth_creates_gitignore_if_not_exists() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["init", "--stealth"])
        .assert()
        .success();

    // Verify .gitignore was created with .qipu/ entry
    let gitignore_path = dir.path().join(".gitignore");
    assert!(gitignore_path.exists());

    let gitignore_content = std::fs::read_to_string(&gitignore_path).unwrap();
    assert!(gitignore_content.contains(".qipu/"));
}

#[test]
fn test_init_stealth_idempotent() {
    let dir = tempdir().unwrap();

    // Run init --stealth twice
    qipu()
        .current_dir(dir.path())
        .args(["init", "--stealth"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["init", "--stealth"])
        .assert()
        .success();

    // Verify .gitignore doesn't have duplicate entries
    let gitignore_path = dir.path().join(".gitignore");
    let gitignore_content = std::fs::read_to_string(&gitignore_path).unwrap();

    let entry_count = gitignore_content
        .lines()
        .filter(|l| l.trim() == ".qipu/")
        .count();
    assert_eq!(
        entry_count, 1,
        "Expected exactly one .qipu/ entry, found {}",
        entry_count
    );
}

#[test]
fn test_init_stealth_creates_store_internal_gitignore() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["init", "--stealth"])
        .assert()
        .success();

    // Verify store-internal .gitignore exists
    let store_gitignore_path = dir.path().join(".qipu/.gitignore");
    assert!(
        store_gitignore_path.exists(),
        "Store-internal .gitignore should exist"
    );

    let store_gitignore_content = std::fs::read_to_string(&store_gitignore_path).unwrap();
    assert!(
        store_gitignore_content.contains("qipu.db"),
        "Store .gitignore should contain qipu.db"
    );
}

#[test]
fn test_init_without_stealth_no_project_gitignore_modification() {
    let dir = tempdir().unwrap();

    // Create a .gitignore with some existing content
    let gitignore_path = dir.path().join(".gitignore");
    std::fs::write(&gitignore_path, "*.log\n*.tmp\n").unwrap();

    // Run init without --stealth
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Verify .gitignore was NOT modified
    let gitignore_content = std::fs::read_to_string(&gitignore_path).unwrap();
    assert!(
        !gitignore_content.contains(".qipu/"),
        "Project .gitignore should not contain .qipu/ without --stealth"
    );
    assert_eq!(gitignore_content, "*.log\n*.tmp\n");
}

#[test]
fn test_init_without_stealth_no_gitignore_created() {
    let dir = tempdir().unwrap();

    // Run init without --stealth
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Verify project root .gitignore was NOT created
    let gitignore_path = dir.path().join(".gitignore");
    assert!(
        !gitignore_path.exists(),
        "Project .gitignore should not be created without --stealth"
    );
}

// ============================================================================
// Protected branch workflow tests (Phase 1.3)
// ============================================================================

#[test]
fn test_init_branch_workflow() {
    use std::process::Command;

    let dir = tempdir().unwrap();

    // First check if git is available
    let git_available = Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !git_available {
        // Test error case when git is not available
        qipu()
            .current_dir(dir.path())
            .args(["init", "--branch", "qipu-metadata"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("Git is required"));
    } else {
        // Git is available - create a proper git repo with initial commit
        Command::new("git")
            .args(["init"])
            .current_dir(dir.path())
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(dir.path())
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(dir.path())
            .output()
            .unwrap();

        // Create an initial commit so HEAD exists
        std::fs::write(dir.path().join("README.md"), "test").unwrap();
        Command::new("git")
            .args(["add", "README.md"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "initial commit"])
            .current_dir(dir.path())
            .output()
            .unwrap();

        // Init with branch should succeed
        qipu()
            .current_dir(dir.path())
            .args(["init", "--branch", "qipu-metadata"])
            .assert()
            .success();

        // Verify store was created
        assert!(dir.path().join(".qipu").exists());

        // Verify we're back on original branch (main or master)
        let current_branch = Command::new("git")
            .args([
                "-C",
                dir.path().to_str().unwrap(),
                "branch",
                "--show-current",
            ])
            .output()
            .unwrap();
        let branch_name = String::from_utf8_lossy(&current_branch.stdout)
            .trim()
            .to_string();
        // Should be on main or master, not qipu-metadata
        assert!(branch_name == "main" || branch_name == "master" || branch_name.is_empty());
    }
}

#[test]
fn test_init_branch_saves_config() {
    use std::process::Command;

    let dir = tempdir().unwrap();

    // Initialize a git repo first
    Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .ok(); // Ignore if git not available

    // Try init with branch
    let result = qipu()
        .current_dir(dir.path())
        .args(["init", "--branch", "qipu-metadata"])
        .assert();

    // Only proceed if git is available
    if result.get_output().status.success() {
        // Verify config file contains branch info
        let config_path = dir.path().join(".qipu/config.toml");
        assert!(config_path.exists());

        let config_content = std::fs::read_to_string(&config_path).unwrap();
        assert!(
            config_content.contains("branch = \"qipu-metadata\""),
            "Config should contain branch field"
        );
    }
}

#[test]
fn test_init_branch_json_output() {
    use std::process::Command;

    let dir = tempdir().unwrap();

    // Initialize a git repo first
    Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .ok();

    // Try init with branch and JSON format
    let result = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "init", "--branch", "qipu-metadata"])
        .assert();

    // Only verify JSON if git is available
    if result.get_output().status.success() {
        let stdout = String::from_utf8_lossy(&result.get_output().stdout);
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(json["status"], "ok");
        assert!(json["store"].as_str().unwrap().ends_with(".qipu"));
    }
}
