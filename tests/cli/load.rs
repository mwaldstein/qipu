//! Tests for `qipu load` command

use crate::support::{qipu, setup_test_dir};
use predicates::prelude::*;

#[test]
fn test_load_with_apply_config() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["capture", "--type", "permanent"])
        .write_stdin("Test note content")
        .assert()
        .success();

    let pack_file = dir.path().join("test.pack");
    qipu()
        .current_dir(dir.path())
        .args(["dump", "-o", pack_file.to_str().unwrap()])
        .assert()
        .success();

    let config_path = dir.path().join(".qipu").join("config.toml");

    std::fs::remove_file(&config_path).unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["load", pack_file.to_str().unwrap(), "--apply-config"])
        .assert()
        .success();

    assert!(config_path.exists(), "config.toml should be restored");
}

#[test]
fn test_load_without_apply_config() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["capture", "--type", "permanent"])
        .write_stdin("Test note content")
        .assert()
        .success();

    let pack_file = dir.path().join("test.pack");
    qipu()
        .current_dir(dir.path())
        .args(["dump", "-o", pack_file.to_str().unwrap()])
        .assert()
        .success();

    let config_path = dir.path().join(".qipu").join("config.toml");

    std::fs::remove_file(&config_path).unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["load", pack_file.to_str().unwrap()])
        .assert()
        .success();

    assert!(
        !config_path.exists(),
        "config.toml should not be restored without --apply-config"
    );
}

#[test]
fn test_load_pack_without_config() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["capture", "--type", "permanent"])
        .write_stdin("Test note content")
        .assert()
        .success();

    let config_path = dir.path().join(".qipu").join("config.toml");

    std::fs::remove_file(&config_path).unwrap();

    let pack_file = dir.path().join("test.pack");
    qipu()
        .current_dir(dir.path())
        .args(["dump", "-o", pack_file.to_str().unwrap()])
        .assert()
        .success();

    let _ = std::fs::remove_file(&config_path);

    qipu()
        .current_dir(dir.path())
        .args(["load", pack_file.to_str().unwrap(), "--apply-config"])
        .assert()
        .success();

    assert!(
        !config_path.exists(),
        "config.toml should not be created if pack has no config"
    );
}

#[test]
fn test_load_invalid_strategy_shows_guidance() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["load", "notes.pack", "--strategy", "merge"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "Use: qipu load <pack-file> --strategy merge-links",
        ))
        .stderr(predicate::str::contains(
            "Other strategies: skip, overwrite",
        ));
}

#[test]
fn test_load_missing_pack_file_shows_guidance() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["load", "missing.pack"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Use: qipu load <pack-file>"))
        .stderr(predicate::str::contains("Create a pack with: qipu dump"));
}
