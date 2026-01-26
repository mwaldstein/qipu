use crate::cli::support::{extract_id_from_bytes, qipu};
use tempfile::tempdir;

// ============================================================================
// JSON Schema Compliance Tests
// ============================================================================

#[test]
fn test_create_json_has_required_fields() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "create", "Test Note"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert!(json["id"].is_string(), "id should be a string");
    assert!(
        json["id"].as_str().unwrap().starts_with("qp-"),
        "id should start with qp-"
    );
    assert_eq!(json["title"], "Test Note");
    assert!(json["type"].is_string(), "type should be a string");
    assert!(json["tags"].is_array(), "tags should be an array");
    assert!(
        json["created"].is_string(),
        "created should be a string (RFC3339)"
    );
    assert!(
        json.as_object().unwrap().contains_key("updated"),
        "updated field should be present"
    );
}

#[test]
fn test_create_json_with_provenance_has_all_fields() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "json",
            "create",
            "--source",
            "https://example.com",
            "--author",
            "Test Author",
            "--generated-by",
            "gpt-4o",
            "--prompt-hash",
            "abc123",
            "--verified",
            "true",
            "Provenance Note",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(json["source"], "https://example.com");
    assert_eq!(json["author"], "Test Author");
    assert_eq!(json["generated_by"], "gpt-4o");
    assert_eq!(json["prompt_hash"], "abc123");
    assert_eq!(json["verified"], true);
}

#[test]
fn test_capture_json_has_required_fields() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "json",
            "capture",
            "--title",
            "Capture Test",
            "--type",
            "fleeting",
            "--tag",
            "test",
        ])
        .write_stdin("Capture content")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert!(json["id"].is_string(), "id should be a string");
    assert!(
        json["id"].as_str().unwrap().starts_with("qp-"),
        "id should start with qp-"
    );
    assert_eq!(json["title"], "Capture Test");
    assert_eq!(json["type"], "fleeting");
    assert!(json["tags"].is_array(), "tags should be an array");
    let tags = json["tags"].as_array().unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0], "test");
    assert!(
        json["created"].is_string(),
        "created should be a string (RFC3339)"
    );
    assert!(
        json.as_object().unwrap().contains_key("updated"),
        "updated field should be present"
    );
}

#[test]
fn test_show_json_has_required_fields() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let create_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Show Test"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let id = extract_id_from_bytes(&create_output);

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(json["id"], id);
    assert_eq!(json["title"], "Show Test");
    assert!(json["type"].is_string(), "type should be a string");
    assert!(json["tags"].is_array(), "tags should be an array");
    assert!(json["body"].is_string(), "body should be a string");
    assert!(
        json["created"].is_string(),
        "created should be a string (RFC3339)"
    );
    assert!(
        json.as_object().unwrap().contains_key("updated"),
        "updated field should be present"
    );
    assert!(
        json.as_object().unwrap().contains_key("value"),
        "value field should be present"
    );
    assert!(
        json.as_object().unwrap().contains_key("verified"),
        "verified field should be present"
    );
}

#[test]
fn test_show_json_custom_omitted_by_default() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let create_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Custom Test"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let id = extract_id_from_bytes(&create_output);

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &id, "priority", "high"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert!(
        !json.as_object().unwrap().contains_key("custom"),
        "custom should be omitted by default"
    );
}

#[test]
fn test_show_json_custom_opt_in() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let create_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Custom Test"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let id = extract_id_from_bytes(&create_output);

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &id, "priority", "high"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id, "--custom"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert!(
        json.as_object().unwrap().contains_key("custom"),
        "custom should be present with --custom flag"
    );
    assert_eq!(json["custom"]["priority"], "high");
}

#[test]
fn test_list_json_has_required_fields() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Note 1", "--tag", "test"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Note 2", "--type", "permanent"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let results = json.as_array().unwrap();

    assert_eq!(results.len(), 2);

    for result in results {
        assert!(result["id"].is_string(), "id should be a string");
        assert!(
            result["id"].as_str().unwrap().starts_with("qp-"),
            "id should start with qp-"
        );
        assert!(result["title"].is_string(), "title should be a string");
        assert!(result["type"].is_string(), "type should be a string");
        assert!(result["tags"].is_array(), "tags should be an array");
        assert!(
            result["created"].is_string(),
            "created should be a string (RFC3339)"
        );
        assert!(
            result.as_object().unwrap().contains_key("updated"),
            "updated field should be present"
        );
    }
}

#[test]
fn test_list_json_empty() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let results = json.as_array().unwrap();

    assert_eq!(results.len(), 0);
}

#[test]
fn test_search_json_has_required_fields() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Programming in Rust"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Learning Go"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "search", "programming"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let results = json.as_array().unwrap();

    assert!(results.len() > 0);

    for result in results {
        assert!(result["id"].is_string(), "id should be a string");
        assert!(
            result["id"].as_str().unwrap().starts_with("qp-"),
            "id should start with qp-"
        );
        assert!(result["title"].is_string(), "title should be a string");
        assert!(result["type"].is_string(), "type should be a string");
        assert!(result["tags"].is_array(), "tags should be an array");
        assert!(
            result["created"].is_string(),
            "created should be a string (RFC3339)"
        );
        assert!(
            result["relevance"].is_number(),
            "relevance should be a number"
        );
    }
}

#[test]
fn test_search_json_empty() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "search", "nonexistent"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let results = json.as_array().unwrap();

    assert_eq!(results.len(), 0);
}

#[test]
fn test_prime_json_has_required_fields() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "prime"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert!(json["store"].is_string(), "store should be a string");
    assert!(json["primer"].is_object(), "primer should be an object");
    assert!(
        json["primer"]["commands"].is_array(),
        "commands should be an array"
    );
    assert!(json["mocs"].is_array(), "mocs should be an array");
    assert!(
        json["recent_notes"].is_array(),
        "recent_notes should be an array"
    );
}
