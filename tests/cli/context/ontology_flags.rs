//! Tests for context command ontology flag behavior
use crate::support::setup_test_dir;
use crate::support::{extract_id, qipu};

#[test]
fn test_context_json_without_include_ontology() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "context", "--note", &id])
        .output()
        .unwrap();

    let json_str = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert!(
        json.get("ontology").is_none(),
        "Should not have ontology object without --include-ontology flag"
    );
}

#[test]
fn test_context_records_without_include_ontology() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "context", "--note", &id])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        !stdout.contains("O mode="),
        "Should not have ontology mode line without --include-ontology flag"
    );
    assert!(
        !stdout.contains("T note_type=\""),
        "Should not have note type lines without --include-ontology flag"
    );
    assert!(
        !stdout.contains("L link_type=\""),
        "Should not have link type lines without --include-ontology flag"
    );
}
