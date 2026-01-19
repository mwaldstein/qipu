use crate::cli::support::qipu;
use predicates::prelude::*;
use tempfile::tempdir;

// ============================================================================
// Logging command tests
// ============================================================================

#[test]
fn test_log_level_debug_shows_debug_messages() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--log-level", "debug", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("parse_args"));
}

#[test]
fn test_log_level_warn_hides_debug_messages() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--log-level", "warn", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("parse_args").not());
}

#[test]
fn test_verbose_shows_debug_messages() {
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
        .stdout(predicate::str::contains("parse_args"));
}

#[test]
fn test_log_json_produces_valid_json() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--log-json", "--log-level", "debug", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"timestamp\""))
        .stdout(predicate::str::contains("\"level\""))
        .stdout(predicate::str::contains("\"message\""));
}

#[test]
fn test_qipu_log_env_overrides_cli_flags() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .env("QIPU_LOG", "qipu=debug")
        .args(["--log-level", "warn", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("parse_args"));
}

#[test]
fn test_qipu_log_env_without_target() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .env("QIPU_LOG", "debug")
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("parse_args"));
}
