//! Tests for context command with default ontology
use crate::support::setup_test_dir;
use crate::support::{extract_id, qipu};

#[test]
fn test_context_json_with_default_ontology() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Ontology Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "json",
            "context",
            "--note",
            &id,
            "--include-ontology",
        ])
        .output()
        .unwrap();

    let json_str = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert!(json["ontology"].is_object(), "Should have ontology object");

    assert_eq!(json["ontology"]["mode"], "default");

    let note_types = json["ontology"]["note_types"].as_array().unwrap();
    assert!(note_types.len() >= 4, "Should have at least 4 note types");

    let type_names: Vec<_> = note_types
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert!(type_names.contains(&"fleeting"));
    assert!(type_names.contains(&"literature"));
    assert!(type_names.contains(&"permanent"));
    assert!(type_names.contains(&"moc"));

    let link_types = json["ontology"]["link_types"].as_array().unwrap();
    assert!(link_types.len() >= 9, "Should have at least 9 link types");

    let link_names: Vec<_> = link_types
        .iter()
        .map(|l| l["name"].as_str().unwrap())
        .collect();
    assert!(link_names.contains(&"related"));
    assert!(link_names.contains(&"derived-from"));
    assert!(link_names.contains(&"supports"));
}

#[test]
fn test_context_records_with_default_ontology() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Ontology Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "records",
            "context",
            "--note",
            &id,
            "--include-ontology",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("O mode=default"));

    assert!(stdout.contains("T note_type=\"fleeting\""));
    assert!(stdout.contains("T note_type=\"literature\""));
    assert!(stdout.contains("T note_type=\"permanent\""));
    assert!(stdout.contains("T note_type=\"moc\""));

    assert!(stdout.contains("L link_type=\"related\""));
    assert!(stdout.contains("L link_type=\"derived-from\""));
    assert!(stdout.contains("L link_type=\"supports\""));
}
