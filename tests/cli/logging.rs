use crate::support::{qipu, setup_test_dir};
use predicates::prelude::*;
use serde_json::Value;

// ============================================================================
// Logging command tests
// ============================================================================

#[test]
fn test_log_level_debug_shows_debug_messages() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["--log-level", "debug", "list"])
        .assert()
        .success()
        .stderr(predicate::str::contains("parse_args"));
}

#[test]
fn test_log_level_warn_hides_debug_messages() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["--log-level", "warn", "list"])
        .assert()
        .success()
        .stderr(predicate::str::contains("parse_args").not());
}

#[test]
fn test_verbose_shows_debug_messages() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["--verbose", "list"])
        .assert()
        .success()
        .stderr(predicate::str::contains("parse_args"));
}

#[test]
fn test_log_json_produces_valid_json() {
    let dir = setup_test_dir();

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
    let dir = setup_test_dir();

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
    let dir = setup_test_dir();

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
    let dir = setup_test_dir();

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
    let dir = setup_test_dir();

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
    let dir = setup_test_dir();

    // Case-insensitive log level should work
    qipu()
        .current_dir(dir.path())
        .args(["--log-level", "DEBUG", "list"])
        .assert()
        .success();
}

#[test]
fn test_default_log_policy_is_warn() {
    let dir = setup_test_dir();

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
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["--log-level", "trace", "list"])
        .assert()
        .success()
        .stderr(predicate::str::contains("parse_args"));
}

#[test]
fn test_json_log_contains_level_field() {
    let dir = setup_test_dir();

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
    let dir = setup_test_dir();

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

#[test]
fn test_error_field_present_in_json_logs() {
    let dir = setup_test_dir();

    let note_path = dir.path().join(".qipu/notes/qp-test123-invalid.md");
    std::fs::create_dir_all(note_path.parent().unwrap()).unwrap();

    std::fs::write(&note_path, "---\nid: [invalid yaml\ntitle: Test\n---\n").unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["--log-json", "--log-level", "warn", "index"])
        .assert()
        .success()
        .stderr(predicate::str::contains("\"fields\""))
        .stderr(predicate::str::contains("\"error\""));
}

#[test]
fn test_error_json_log_contains_error_field() {
    let dir = setup_test_dir();

    let note_path = dir.path().join(".qipu/notes/qp-test456-invalid.md");
    std::fs::create_dir_all(note_path.parent().unwrap()).unwrap();

    std::fs::write(&note_path, "---\nid: [invalid yaml\ntitle: Test\n---\n").unwrap();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--log-json", "--log-level", "warn", "index"])
        .assert()
        .success();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let json_lines: Vec<&str> = stderr.lines().filter(|l| !l.is_empty()).collect();

    let found_error_field = json_lines.iter().any(|line| {
        if let Ok(json) = serde_json::from_str::<Value>(line) {
            if let Some(fields) = json.get("fields").and_then(|f| f.as_object()) {
                return fields.contains_key("error") && fields.contains_key("path");
            }
        }
        false
    });

    assert!(
        found_error_field,
        "Should find log with error and path fields"
    );
}

#[test]
fn test_error_log_level_is_warn() {
    let dir = setup_test_dir();

    let note_path = dir.path().join(".qipu/notes/qp-test789-invalid.md");
    std::fs::create_dir_all(note_path.parent().unwrap()).unwrap();

    std::fs::write(&note_path, "---\nid: [invalid yaml\ntitle: Test\n---\n").unwrap();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--log-json", "--log-level", "warn", "index"])
        .assert()
        .success();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let json_lines: Vec<&str> = stderr.lines().filter(|l| !l.is_empty()).collect();

    let found_warn_level = json_lines.iter().any(|line| {
        if let Ok(json) = serde_json::from_str::<Value>(line) {
            if let Some(fields) = json.get("fields").and_then(|f| f.as_object()) {
                if fields.get("message").and_then(|m| m.as_str()) == Some("Failed to parse note") {
                    return json.get("level").and_then(|l| l.as_str()) == Some("WARN");
                }
            }
        }
        false
    });

    assert!(found_warn_level, "Error log should have WARN level");
}

#[test]
fn test_error_log_contains_message_field() {
    let dir = setup_test_dir();

    let note_path = dir.path().join(".qipu/notes/qp-test000-invalid.md");
    std::fs::create_dir_all(note_path.parent().unwrap()).unwrap();

    std::fs::write(&note_path, "---\nid: [invalid yaml\ntitle: Test\n---\n").unwrap();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--log-json", "--log-level", "warn", "index"])
        .assert()
        .success();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let json_lines: Vec<&str> = stderr.lines().filter(|l| !l.is_empty()).collect();

    let found_message = json_lines.iter().any(|line| {
        if let Ok(json) = serde_json::from_str::<Value>(line) {
            if let Some(fields) = json.get("fields").and_then(|f| f.as_object()) {
                return fields.get("message").and_then(|m| m.as_str())
                    == Some("Failed to parse note");
            }
        }
        false
    });

    assert!(
        found_message,
        "Should find log with 'Failed to parse note' message"
    );
}
