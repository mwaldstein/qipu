use crate::cli::support::qipu;
use predicates::prelude::*;
use serde_json::Value;
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
        .stderr(predicate::str::contains("parse_args"));
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
        .stderr(predicate::str::contains("parse_args").not());
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
        .stderr(predicate::str::contains("parse_args"));
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
        .stderr(predicate::str::contains("\"timestamp\""))
        .stderr(predicate::str::contains("\"level\""))
        .stderr(predicate::str::contains("\"message\""));
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
        .stderr(predicate::str::contains("parse_args"));
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
        .stderr(predicate::str::contains("parse_args"));
}

#[test]
fn test_invalid_log_level_rejected() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Invalid log level should be rejected
    qipu()
        .current_dir(dir.path())
        .args(["--log-level", "invalid", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid log level"))
        .stderr(predicate::str::contains("error, warn, info, debug, trace"));
}

#[test]
fn test_valid_log_levels_accepted() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Test all valid log levels
    for level in ["error", "warn", "info", "debug", "trace"] {
        qipu()
            .current_dir(dir.path())
            .args(["--log-level", level, "list"])
            .assert()
            .success();
    }
}

#[test]
fn test_log_level_case_insensitive() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Case-insensitive log level should work
    qipu()
        .current_dir(dir.path())
        .args(["--log-level", "DEBUG", "list"])
        .assert()
        .success();
}

#[test]
fn test_default_log_policy_is_warn() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Default (no flags) should show warnings but not debug messages
    qipu()
        .current_dir(dir.path())
        .arg("list")
        .assert()
        .success()
        .stderr(predicate::str::contains("parse_args").not()); // No debug messages

    // Verify warn-level messages would be shown by creating a condition that triggers a warning
    // Note: In normal operation, list doesn't produce warnings, so we just verify debug is off
}

#[test]
fn test_log_level_trace_shows_debug_messages() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--log-level", "trace", "list"])
        .assert()
        .success()
        .stderr(predicate::str::contains("parse_args"));
}

#[test]
fn test_json_log_contains_elapsed_field() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--log-json", "--log-level", "debug", "list"])
        .assert()
        .success();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let json_lines: Vec<&str> = stderr.lines().filter(|l| !l.is_empty()).collect();

    let found_parse_args = json_lines.iter().any(|line| {
        if let Ok(json) = serde_json::from_str::<Value>(line) {
            if let Some(fields) = json.get("fields").and_then(|f| f.as_object()) {
                return fields.get("message").and_then(|m| m.as_str()) == Some("parse_args")
                    && fields.contains_key("elapsed");
            }
        }
        false
    });

    assert!(
        found_parse_args,
        "Should find parse_args log with elapsed field"
    );
}

#[test]
fn test_json_log_elapsed_field_is_numeric_string() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--log-json", "--log-level", "debug", "list"])
        .assert()
        .success();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let json_lines: Vec<&str> = stderr.lines().filter(|l| !l.is_empty()).collect();

    let found = json_lines.iter().any(|line| {
        if let Ok(json) = serde_json::from_str::<Value>(line) {
            if let Some(fields) = json.get("fields").and_then(|f| f.as_object()) {
                if fields.get("message").and_then(|m| m.as_str()) == Some("parse_args") {
                    if let Some(elapsed) = fields.get("elapsed").and_then(|e| e.as_str()) {
                        return elapsed.ends_with("ms")
                            || elapsed.ends_with("s")
                            || elapsed.ends_with("ns");
                    }
                }
            }
        }
        false
    });

    assert!(
        found,
        "Should find parse_args log with numeric elapsed string"
    );
}

#[test]
fn test_json_log_contains_context_params_fields() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("capture")
        .write_stdin("Test note")
        .arg("--tag")
        .arg("test")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--log-json",
            "--log-level",
            "debug",
            "--verbose",
            "context",
            "--tag",
            "test",
        ])
        .assert()
        .success();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let json_lines: Vec<&str> = stderr.lines().filter(|l| !l.is_empty()).collect();

    let found_context_params = json_lines.iter().any(|line| {
        if let Ok(json) = serde_json::from_str::<Value>(line) {
            let fields = json.get("fields").and_then(|f| f.as_object());
            if let Some(fields_obj) = fields {
                fields_obj.get("message").and_then(|m| m.as_str()) == Some("context_params")
                    && fields_obj.contains_key("tag")
                    && fields_obj.contains_key("note_ids_count")
                    && fields_obj.contains_key("with_body")
                    && fields_obj.contains_key("transitive")
            } else {
                false
            }
        } else {
            false
        }
    });

    assert!(
        found_context_params,
        "Should find context_params log with structured fields"
    );
}

#[test]
fn test_json_log_context_params_field_types() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("capture")
        .write_stdin("Test note")
        .arg("--tag")
        .arg("test")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--log-json",
            "--log-level",
            "debug",
            "--verbose",
            "context",
            "--tag",
            "test",
        ])
        .assert()
        .success();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let json_lines: Vec<&str> = stderr.lines().filter(|l| !l.is_empty()).collect();

    let found = json_lines.iter().any(|line| {
        if let Ok(json) = serde_json::from_str::<Value>(line) {
            if let Some(fields) = json.get("fields").and_then(|f| f.as_object()) {
                if fields.get("message").and_then(|m| m.as_str()) == Some("context_params") {
                    let note_ids_count = fields.get("note_ids_count").and_then(|v| v.as_u64());
                    let with_body = fields.get("with_body").and_then(|v| v.as_bool());
                    let transitive = fields.get("transitive").and_then(|v| v.as_bool());
                    return note_ids_count.is_some() && with_body.is_some() && transitive.is_some();
                }
            }
        }
        false
    });

    assert!(
        found,
        "Should find context_params log with correctly typed fields"
    );
}

#[test]
fn test_json_log_contains_level_field() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--log-json", "--log-level", "debug", "list"])
        .assert()
        .success();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let json_lines: Vec<&str> = stderr.lines().filter(|l| !l.is_empty()).collect();

    let found = json_lines.iter().any(|line| {
        if let Ok(json) = serde_json::from_str::<Value>(line) {
            json["level"].is_string() && json["level"].as_str() == Some("DEBUG")
        } else {
            false
        }
    });

    assert!(found, "Should find log with DEBUG level field");
}

#[test]
fn test_json_log_level_values_are_valid() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let valid_levels = ["ERROR", "WARN", "INFO", "DEBUG", "TRACE"];

    for level in valid_levels {
        let level_lower = level.to_lowercase();
        let output = qipu()
            .current_dir(dir.path())
            .args(["--log-json", "--log-level", &level_lower, "list"])
            .assert()
            .success();

        let stderr = String::from_utf8_lossy(&output.get_output().stderr);
        let json_lines: Vec<&str> = stderr.lines().filter(|l| !l.is_empty()).collect();

        for line in json_lines {
            if let Ok(json) = serde_json::from_str::<Value>(line) {
                if let Some(log_level) = json["level"].as_str() {
                    assert!(
                        valid_levels.contains(&log_level),
                        "Invalid log level: {}",
                        log_level
                    );
                }
            }
        }
    }
}
