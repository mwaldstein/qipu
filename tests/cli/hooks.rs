use crate::support::qipu;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_hooks_status_outside_git_shows_guidance() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["hooks", "status"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Use: git init"))
        .stderr(predicate::str::contains("Then: qipu hooks install"));
}

#[test]
fn test_hooks_unknown_hook_shows_guidance() {
    let dir = tempdir().unwrap();

    std::process::Command::new("git")
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["hooks", "install", "bad-hook"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Unknown hook"))
        .stderr(predicate::str::contains("Use: qipu hooks install"));
}

#[test]
fn test_hooks_existing_unmanaged_hook_shows_force_command() {
    let dir = tempdir().unwrap();

    std::process::Command::new("git")
        .arg("init")
        .current_dir(dir.path())
        .output()
        .unwrap();
    let hooks_dir = dir.path().join(".git/hooks");
    std::fs::write(hooks_dir.join("pre-commit"), "#!/bin/sh\n").unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["hooks", "install", "pre-commit"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "Use: qipu hooks install pre-commit --force",
        ));
}
