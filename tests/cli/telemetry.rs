//! Telemetry command integration tests
//!
//! Tests for telemetry enable/disable/status commands using QIPU_CONFIG_DIR
//! environment variable to isolate tests from the user's real global config.

use crate::support::qipu;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

const TEST_CONFIG_DIR_ENV: &str = "QIPU_CONFIG_DIR";
const NO_TELEMETRY_ENV: &str = "QIPU_NO_TELEMETRY";

// =============================================================================
// Telemetry enable command
// =============================================================================

#[test]
fn test_telemetry_enable_creates_config() {
    let config_dir = tempdir().unwrap();

    qipu()
        .env(TEST_CONFIG_DIR_ENV, config_dir.path())
        .args(["telemetry", "enable"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Telemetry enabled"));

    // Verify config file was created
    let config_path = config_dir.path().join("config.toml");
    assert!(config_path.exists(), "Config file should be created");

    let config_content = fs::read_to_string(&config_path).unwrap();
    assert!(
        config_content.contains("telemetry_enabled = true"),
        "Config should have telemetry enabled"
    );
}

#[test]
fn test_telemetry_enable_updates_existing_config() {
    let config_dir = tempdir().unwrap();
    let config_path = config_dir.path().join("config.toml");

    // Create initial config with telemetry disabled
    fs::write(&config_path, "telemetry_enabled = false\n").unwrap();

    qipu()
        .env(TEST_CONFIG_DIR_ENV, config_dir.path())
        .args(["telemetry", "enable"])
        .assert()
        .success();

    let config_content = fs::read_to_string(&config_path).unwrap();
    assert!(
        config_content.contains("telemetry_enabled = true"),
        "Config should be updated to enable telemetry"
    );
}

// =============================================================================
// Telemetry disable command
// =============================================================================

#[test]
fn test_telemetry_disable_creates_config() {
    let config_dir = tempdir().unwrap();

    qipu()
        .env(TEST_CONFIG_DIR_ENV, config_dir.path())
        .args(["telemetry", "disable"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Telemetry disabled"));

    let config_path = config_dir.path().join("config.toml");
    assert!(config_path.exists(), "Config file should be created");

    let config_content = fs::read_to_string(&config_path).unwrap();
    assert!(
        config_content.contains("telemetry_enabled = false"),
        "Config should have telemetry disabled"
    );
}

#[test]
fn test_telemetry_disable_updates_existing_config() {
    let config_dir = tempdir().unwrap();
    let config_path = config_dir.path().join("config.toml");

    // Create initial config with telemetry enabled
    fs::write(&config_path, "telemetry_enabled = true\n").unwrap();

    qipu()
        .env(TEST_CONFIG_DIR_ENV, config_dir.path())
        .args(["telemetry", "disable"])
        .assert()
        .success();

    let config_content = fs::read_to_string(&config_path).unwrap();
    assert!(
        config_content.contains("telemetry_enabled = false"),
        "Config should be updated to disable telemetry"
    );
}

// =============================================================================
// Telemetry status command
// =============================================================================

#[test]
fn test_telemetry_status_shows_disabled_by_default() {
    let config_dir = tempdir().unwrap();

    qipu()
        .env(TEST_CONFIG_DIR_ENV, config_dir.path())
        .args(["telemetry", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Telemetry: disabled"))
        .stdout(predicate::str::contains("Source: custom config directory"));
}

#[test]
fn test_telemetry_status_shows_enabled_from_config() {
    let config_dir = tempdir().unwrap();
    let config_path = config_dir.path().join("config.toml");

    // Create config with telemetry enabled
    fs::write(&config_path, "telemetry_enabled = true\n").unwrap();

    qipu()
        .env(TEST_CONFIG_DIR_ENV, config_dir.path())
        .args(["telemetry", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Telemetry: enabled"))
        .stdout(predicate::str::contains("Source: custom config directory"));
}

#[test]
fn test_telemetry_status_shows_disabled_from_config() {
    let config_dir = tempdir().unwrap();
    let config_path = config_dir.path().join("config.toml");

    // Create config with telemetry disabled
    fs::write(&config_path, "telemetry_enabled = false\n").unwrap();

    qipu()
        .env(TEST_CONFIG_DIR_ENV, config_dir.path())
        .args(["telemetry", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Telemetry: disabled"))
        .stdout(predicate::str::contains("Source: custom config directory"));
}

#[test]
fn test_telemetry_status_respects_no_telemetry_env() {
    let config_dir = tempdir().unwrap();
    let config_path = config_dir.path().join("config.toml");

    // Create config with telemetry enabled
    fs::write(&config_path, "telemetry_enabled = true\n").unwrap();

    qipu()
        .env(TEST_CONFIG_DIR_ENV, config_dir.path())
        .env(NO_TELEMETRY_ENV, "1")
        .args(["telemetry", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Telemetry: disabled"))
        .stdout(predicate::str::contains(
            "Source: QIPU_NO_TELEMETRY environment variable",
        ));
}

// =============================================================================
// Telemetry show command
// =============================================================================

#[test]
fn test_telemetry_show_shows_status_when_disabled() {
    let config_dir = tempdir().unwrap();

    qipu()
        .env(TEST_CONFIG_DIR_ENV, config_dir.path())
        .args(["telemetry", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Telemetry status: disabled"))
        .stdout(predicate::str::contains("No pending telemetry events"));
}

#[test]
fn test_telemetry_show_shows_status_when_enabled() {
    let config_dir = tempdir().unwrap();
    let config_path = config_dir.path().join("config.toml");

    // Create config with telemetry enabled
    fs::write(&config_path, "telemetry_enabled = true\n").unwrap();

    qipu()
        .env(TEST_CONFIG_DIR_ENV, config_dir.path())
        .args(["telemetry", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Telemetry status: enabled"))
        .stdout(predicate::str::contains("No pending telemetry events"));
}

// =============================================================================
// Integration: full enable/disable/status cycle
// =============================================================================

#[test]
fn test_telemetry_full_cycle() {
    let config_dir = tempdir().unwrap();

    // Initially disabled by default
    qipu()
        .env(TEST_CONFIG_DIR_ENV, config_dir.path())
        .args(["telemetry", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Telemetry: disabled"));

    // Enable it
    qipu()
        .env(TEST_CONFIG_DIR_ENV, config_dir.path())
        .args(["telemetry", "enable"])
        .assert()
        .success();

    // Status should show enabled
    qipu()
        .env(TEST_CONFIG_DIR_ENV, config_dir.path())
        .args(["telemetry", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Telemetry: enabled"));

    // Disable it
    qipu()
        .env(TEST_CONFIG_DIR_ENV, config_dir.path())
        .args(["telemetry", "disable"])
        .assert()
        .success();

    // Status should show disabled
    qipu()
        .env(TEST_CONFIG_DIR_ENV, config_dir.path())
        .args(["telemetry", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Telemetry: disabled"));

    // Verify the config file persisted correctly
    let config_path = config_dir.path().join("config.toml");
    let config_content = fs::read_to_string(&config_path).unwrap();
    assert!(config_content.contains("telemetry_enabled = false"));
}
