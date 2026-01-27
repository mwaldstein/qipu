use crate::cli::support::{extract_id_from_bytes, qipu};
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

#[test]
fn test_json_log_contains_span_name() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .arg("capture")
        .write_stdin("Test note")
        .arg("--title")
        .arg("Test")
        .assert()
        .success();

    let _note_id = extract_id_from_bytes(&output.get_output().stdout);

    let output = qipu()
        .current_dir(dir.path())
        .args(["--log-json", "--log-level", "debug", "list"])
        .assert()
        .success();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let json_lines: Vec<&str> = stderr.lines().filter(|l| !l.is_empty()).collect();

    let found_span_name = json_lines.iter().any(|line| {
        if let Ok(json) = serde_json::from_str::<Value>(line) {
            json.get("span")
                .and_then(|s| s.as_object())
                .map(|span_obj| span_obj.contains_key("name"))
                .unwrap_or(false)
        } else {
            false
        }
    });

    assert!(found_span_name, "Should find log with span name field");
}

#[test]
fn test_json_log_span_name_is_string() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .arg("capture")
        .write_stdin("Test note")
        .arg("--title")
        .arg("Test")
        .assert()
        .success();

    let _note_id = extract_id_from_bytes(&output.get_output().stdout);

    let output = qipu()
        .current_dir(dir.path())
        .args(["--log-json", "--log-level", "debug", "list"])
        .assert()
        .success();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let json_lines: Vec<&str> = stderr.lines().filter(|l| !l.is_empty()).collect();

    let found = json_lines.iter().any(|line| {
        if let Ok(json) = serde_json::from_str::<Value>(line) {
            if let Some(span) = json.get("span").and_then(|s| s.as_object()) {
                if let Some(name) = span.get("name").and_then(|n| n.as_str()) {
                    return !name.is_empty();
                }
            }
        }
        false
    });

    assert!(found, "Should find log with non-empty span name string");
}

#[test]
fn test_json_log_has_spans_array() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .arg("capture")
        .write_stdin("Test note")
        .arg("--title")
        .arg("Test")
        .assert()
        .success();

    let note_id = extract_id_from_bytes(&output.get_output().stdout);

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--log-json",
            "--log-level",
            "debug",
            "link",
            "tree",
            &note_id,
        ])
        .assert()
        .success();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let json_lines: Vec<&str> = stderr.lines().filter(|l| !l.is_empty()).collect();

    let found_spans_array = json_lines.iter().any(|line| {
        if let Ok(json) = serde_json::from_str::<Value>(line) {
            json.get("spans").is_some()
        } else {
            false
        }
    });

    assert!(found_spans_array, "Should find log with spans array field");
}

#[test]
fn test_json_log_span_has_name() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .arg("capture")
        .write_stdin("Test note")
        .arg("--title")
        .arg("Test")
        .assert()
        .success();

    let note_id = extract_id_from_bytes(&output.get_output().stdout);

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--log-json",
            "--log-level",
            "debug",
            "link",
            "tree",
            &note_id,
        ])
        .assert()
        .success();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let json_lines: Vec<&str> = stderr.lines().filter(|l| !l.is_empty()).collect();

    let found = json_lines.iter().any(|line| {
        if let Ok(json) = serde_json::from_str::<Value>(line) {
            if let Some(span) = json.get("span").and_then(|s| s.as_object()) {
                if let Some(name) = span.get("name").and_then(|n| n.as_str()) {
                    return !name.is_empty();
                }
            }
        }
        false
    });

    assert!(found, "Should find log with span name field");
}

#[test]
fn test_json_log_bfs_traverse_has_custom_fields() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .arg("capture")
        .write_stdin("Test note")
        .arg("--title")
        .arg("Test")
        .assert()
        .success();

    let note_id = extract_id_from_bytes(&output.get_output().stdout);

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--log-json",
            "--log-level",
            "debug",
            "link",
            "tree",
            &note_id,
            "--ignore-value",
        ])
        .assert()
        .success();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let json_lines: Vec<&str> = stderr.lines().filter(|l| !l.is_empty()).collect();

    let found_bfs_traverse = json_lines.iter().any(|line| {
        if let Ok(json) = serde_json::from_str::<Value>(line) {
            if let Some(span) = json.get("span").and_then(|s| s.as_object()) {
                if let Some(name) = span.get("name").and_then(|n| n.as_str()) {
                    if name == "bfs_traverse" {
                        return span.contains_key("root")
                            && span.contains_key("direction")
                            && span.contains_key("max_hops");
                    }
                }
            }
        }
        false
    });

    assert!(
        found_bfs_traverse,
        "Should find bfs_traverse span with custom fields (root, direction, max_hops)"
    );
}

#[test]
fn test_json_log_dijkstra_traverse_has_custom_fields() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .arg("capture")
        .write_stdin("Test note")
        .arg("--title")
        .arg("Test")
        .assert()
        .success();

    let note_id = extract_id_from_bytes(&output.get_output().stdout);

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--log-json",
            "--log-level",
            "debug",
            "link",
            "tree",
            &note_id,
        ])
        .assert()
        .success();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let json_lines: Vec<&str> = stderr.lines().filter(|l| !l.is_empty()).collect();

    let found_dijkstra_traverse = json_lines.iter().any(|line| {
        if let Ok(json) = serde_json::from_str::<Value>(line) {
            if let Some(span) = json.get("span").and_then(|s| s.as_object()) {
                if let Some(name) = span.get("name").and_then(|n| n.as_str()) {
                    if name == "dijkstra_traverse" {
                        return span.contains_key("root")
                            && span.contains_key("direction")
                            && span.contains_key("max_hops")
                            && span.contains_key("ignore_value");
                    }
                }
            }
        }
        false
    });

    assert!(
        found_dijkstra_traverse,
        "Should find dijkstra_traverse span with custom fields (root, direction, max_hops, ignore_value)"
    );
}

#[test]
fn test_json_log_bfs_path_has_custom_fields() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .arg("capture")
        .write_stdin("Test note")
        .arg("--title")
        .arg("Test")
        .assert()
        .success();

    let note_id = extract_id_from_bytes(&output.get_output().stdout);

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--log-json",
            "--log-level",
            "debug",
            "link",
            "path",
            &note_id,
            &note_id,
        ])
        .assert()
        .success();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let json_lines: Vec<&str> = stderr.lines().filter(|l| !l.is_empty()).collect();

    let found_bfs_path = json_lines.iter().any(|line| {
        if let Ok(json) = serde_json::from_str::<Value>(line) {
            if let Some(span) = json.get("span").and_then(|s| s.as_object()) {
                if let Some(name) = span.get("name").and_then(|n| n.as_str()) {
                    if name == "bfs_find_path" {
                        return span.contains_key("from")
                            && span.contains_key("to")
                            && span.contains_key("direction")
                            && span.contains_key("max_hops")
                            && span.contains_key("ignore_value");
                    }
                }
            }
        }
        false
    });

    assert!(
        found_bfs_path,
        "Should find bfs_find_path span with custom fields (from, to, direction, max_hops, ignore_value)"
    );
}

#[test]
fn test_error_field_present_in_json_logs() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
