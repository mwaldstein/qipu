//! Tests for `qipu load` command

use crate::support::{qipu, setup_test_dir};
use tempfile::tempdir;

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
