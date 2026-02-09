use crate::support::setup_test_dir;
use crate::support::{extract_id, extract_id_from_bytes, qipu};

#[test]
fn test_create_json_has_required_fields() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "create", "Test Note"])
        .output()
        .unwrap()
        .stdout;

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
    let dir = setup_test_dir();

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
        .output()
        .unwrap()
        .stdout;

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
    let dir = setup_test_dir();

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
        .output()
        .unwrap()
        .stdout;

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
    let dir = setup_test_dir();

    let create_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Show Test"])
        .output()
        .unwrap()
        .stdout;

    let id = extract_id_from_bytes(&create_output);

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id])
        .output()
        .unwrap()
        .stdout;

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
    let dir = setup_test_dir();

    let create_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Custom Test"])
        .output()
        .unwrap()
        .stdout;

    let id = extract_id_from_bytes(&create_output);

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &id, "priority", "high"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id])
        .output()
        .unwrap()
        .stdout;

    let json_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert!(
        !json.as_object().unwrap().contains_key("custom"),
        "custom should be omitted by default"
    );
}

#[test]
fn test_show_json_custom_opt_in() {
    let dir = setup_test_dir();

    let create_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Custom Test"])
        .output()
        .unwrap()
        .stdout;

    let id = extract_id_from_bytes(&create_output);

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &id, "priority", "high"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id, "--custom"])
        .output()
        .unwrap()
        .stdout;

    let json_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert!(
        json.as_object().unwrap().contains_key("custom"),
        "custom should be present with --custom flag"
    );
    assert_eq!(json["custom"]["priority"], "high");
}

#[test]
fn test_update_json_has_required_fields() {
    let dir = setup_test_dir();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Update Test"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Update and verify JSON output has required fields
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "json",
            "update",
            "--title",
            "Updated Title",
            &id,
        ])
        .output()
        .unwrap()
        .stdout;

    let json_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert!(json["id"].is_string(), "id should be a string");
    assert!(
        json["id"].as_str().unwrap().starts_with("qp-"),
        "id should start with qp-"
    );
    assert_eq!(json["title"], "Updated Title");
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
fn test_inbox_json_has_required_fields() {
    let dir = setup_test_dir();

    // Create a fleeting note (should appear in inbox)
    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "fleeting", "Inbox Test"])
        .assert()
        .success();

    // Query inbox in JSON format
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "inbox"])
        .output()
        .unwrap()
        .stdout;

    let json_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let results = json.as_array().unwrap();

    assert!(
        !results.is_empty(),
        "inbox should contain the fleeting note"
    );

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
