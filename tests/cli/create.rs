use crate::cli::support::qipu;
use predicates::prelude::*;
use tempfile::tempdir;

// ============================================================================
// Create command tests
// ============================================================================

#[test]
fn test_create_note() {
    let dir = tempdir().unwrap();

    // Init first
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create note
    qipu()
        .current_dir(dir.path())
        .args(["create", "My Test Note"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));
}

#[test]
fn test_create_with_type() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "permanent", "Permanent Note"])
        .assert()
        .success();
}

#[test]
fn test_create_with_tags() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--tag", "test", "--tag", "demo", "Tagged Note"])
        .assert()
        .success();
}

#[test]
fn test_create_json_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "create", "JSON Note"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"id\":"))
        .stdout(predicate::str::contains("\"title\": \"JSON Note\""));
}

#[test]
fn test_new_alias() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // 'new' should work like 'create'
    qipu()
        .current_dir(dir.path())
        .args(["new", "Alias Test"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("qp-"));
}

#[test]
fn test_create_json_with_provenance() {
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
            "TestBot",
            "--generated-by",
            "gpt-4o",
            "--prompt-hash",
            "abc123",
            "--verified",
            "false",
            "Provenance Test",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Verify provenance fields are present
    assert_eq!(json["source"], "https://example.com");
    assert_eq!(json["author"], "TestBot");
    assert_eq!(json["generated_by"], "gpt-4o");
    assert_eq!(json["prompt_hash"], "abc123");
    assert_eq!(json["verified"], false);

    // Verify standard fields are also present
    assert!(json["id"].as_str().unwrap().starts_with("qp-"));
    assert_eq!(json["title"], "Provenance Test");
}
