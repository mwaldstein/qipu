//! Tests for context command ontology replacement
use crate::support::setup_test_dir;
use crate::support::{extract_id, qipu};
use std::fs;

#[test]
fn test_context_json_with_replacement_ontology() {
    let dir = setup_test_dir();

    let config_path = dir.path().join(".qipu/config.toml");
    let config_content = r#"
version = 1
default_note_type = "bug"

[ontology]
mode = "replacement"

[ontology.note_types.bug]
description = "A software bug"

[ontology.note_types.feature]
description = "A feature request"

[ontology.link_types.blocks]
description = "Blocks another issue"
inverse = "blocked-by"
"#;
    fs::write(config_path, config_content).unwrap();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Bug Report", "--type", "bug"])
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

    assert_eq!(json["ontology"]["mode"], "replacement");

    let note_types = json["ontology"]["note_types"].as_array().unwrap();
    assert_eq!(note_types.len(), 2, "Should have exactly 2 note types");

    let type_names: Vec<_> = note_types
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert!(type_names.contains(&"bug"));
    assert!(type_names.contains(&"feature"));

    let link_types = json["ontology"]["link_types"].as_array().unwrap();
    assert_eq!(link_types.len(), 1, "Should have exactly 1 link type");

    let blocks = &link_types[0];
    assert_eq!(blocks["name"], "blocks");
    assert_eq!(blocks["inverse"], "blocked-by");
    assert_eq!(blocks["description"], "Blocks another issue");
}

#[test]
fn test_context_records_with_replacement_ontology() {
    let dir = setup_test_dir();

    let config_path = dir.path().join(".qipu/config.toml");
    let config_content = r#"
version = 1
default_note_type = "bug"

[ontology]
mode = "replacement"

[ontology.note_types.bug]
description = "A software bug"

[ontology.note_types.feature]
description = "A feature request"

[ontology.link_types.blocks]
description = "Blocks another issue"
inverse = "blocked-by"
"#;
    fs::write(config_path, config_content).unwrap();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Bug Report", "--type", "bug"])
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

    assert!(stdout.contains("O mode=replacement"));

    assert!(stdout.contains("T note_type=\"bug\" description=\"A software bug\""));
    assert!(stdout.contains("T note_type=\"feature\" description=\"A feature request\""));

    assert!(stdout.contains(
        "L link_type=\"blocks\" inverse=\"blocked-by\" description=\"Blocks another issue\""
    ));
}
