use crate::support::qipu;
use crate::support::setup_test_dir;

#[test]
fn test_list_json_has_required_fields() {
    let dir = setup_test_dir();

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
        .output()
        .unwrap()
        .stdout;

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
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "list"])
        .output()
        .unwrap()
        .stdout;

    let json_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let results = json.as_array().unwrap();

    assert_eq!(results.len(), 0);
}

#[test]
fn test_search_json_has_required_fields() {
    let dir = setup_test_dir();

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
        .output()
        .unwrap()
        .stdout;

    let json_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let results = json.as_array().unwrap();

    assert!(!results.is_empty());

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
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "search", "nonexistent"])
        .output()
        .unwrap()
        .stdout;

    let json_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let results = json.as_array().unwrap();

    assert_eq!(results.len(), 0);
}

#[test]
fn test_prime_json_has_required_fields() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "prime"])
        .output()
        .unwrap()
        .stdout;

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
