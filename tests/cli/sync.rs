use crate::support::qipu;
use predicates::prelude::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_sync_basic() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["sync"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Indexed 0 notes"));
}

#[test]
fn test_sync_validate() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["sync", "--validate"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Indexed 0 notes"))
        .stdout(predicate::str::contains("Store validated"));
}

#[test]
fn test_sync_git_automation() {
    let dir = tempdir().unwrap();

    // Check if git is available
    let git_available = Command::new("git")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if !git_available {
        return;
    }

    // Initialize a git repo
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

    // Create an initial commit
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

    // Init qipu with branch
    qipu()
        .current_dir(dir.path())
        .args(["init", "--branch", "qipu-notes"])
        .assert()
        .success();

    // Create a note (this will happen on the main branch but we want to sync it to the qipu-notes branch)
    // Actually, in the current implementation, qipu commands happen on whatever branch is current.
    // The --branch workflow in init switches to the branch, does the init, then switches back.
    // The sync --commit command should switch to the branch, add/commit, then switch back.

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success();

    // Verify change is NOT on main branch (if we were on main)
    // Actually, the note is created in the filesystem.
    // If it's NOT staged/committed on main, then sync --commit should handle it.

    qipu()
        .current_dir(dir.path())
        .args(["sync", "--commit"])
        .assert()
        .success();

    // Verify we're back on main branch
    let current_branch = Command::new("git")
        .args([
            "-C",
            dir.path().to_str().unwrap(),
            "rev-parse",
            "--abbrev-ref",
            "HEAD",
        ])
        .output()
        .unwrap();
    let branch_name = String::from_utf8_lossy(&current_branch.stdout)
        .trim()
        .to_string();
    assert!(branch_name == "main" || branch_name == "master");

    // Verify commit exists on qipu-notes branch
    let log_output = Command::new("git")
        .args([
            "-C",
            dir.path().to_str().unwrap(),
            "log",
            "qipu-notes",
            "--oneline",
        ])
        .output()
        .unwrap();
    let log_content = String::from_utf8_lossy(&log_output.stdout);
    assert!(log_content.contains("qipu sync: update notes and indexes"));
}

#[test]
fn test_sync_push_fails_no_remote() {
    let dir = tempdir().unwrap();

    // Check if git is available
    let git_available = Command::new("git")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if !git_available {
        return;
    }

    // Initialize a git repo
    Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    // Init qipu with branch
    qipu()
        .current_dir(dir.path())
        .args(["init", "--branch", "qipu-notes"])
        .assert()
        .success();

    // Try to sync with push - should fail because no 'origin' remote exists
    qipu()
        .current_dir(dir.path())
        .args(["sync", "--push"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to push"));
}
