use crate::support::setup_test_dir;
use crate::support::{extract_id, qipu};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_context_json_with_extended_ontology() {
    let dir = setup_test_dir();

    let config_path = dir.path().join(".qipu/config.toml");
    let config_content = r#"
version = 1
default_note_type = "fleeting"

[ontology]
mode = "extended"

[ontology.note_types.task]
description = "A task item"
usage = "Use for task tracking"

[ontology.link_types.depends-on]
description = "Dependency relationship"
inverse = "required-by"
usage = "A depends on B"
"#;
    fs::write(config_path, config_content).unwrap();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Task Note", "--type", "task"])
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

    assert_eq!(json["ontology"]["mode"], "extended");

    let note_types = json["ontology"]["note_types"].as_array().unwrap();
    let type_names: Vec<_> = note_types
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert!(type_names.contains(&"fleeting"));
    assert!(type_names.contains(&"task"));

    let task_type = note_types
        .iter()
        .find(|t| t["name"] == "task")
        .expect("task type should exist");
    assert_eq!(task_type["description"], "A task item");
    assert_eq!(task_type["usage"], "Use for task tracking");

    let link_types = json["ontology"]["link_types"].as_array().unwrap();
    let link_names: Vec<_> = link_types
        .iter()
        .map(|l| l["name"].as_str().unwrap())
        .collect();
    assert!(link_names.contains(&"depends-on"));

    let depends_on = link_types
        .iter()
        .find(|l| l["name"] == "depends-on")
        .expect("depends-on link type should exist");
    assert_eq!(depends_on["inverse"], "required-by");
    assert_eq!(depends_on["description"], "Dependency relationship");
    assert_eq!(depends_on["usage"], "A depends on B");
}

#[test]
fn test_context_records_with_extended_ontology() {
    let dir = setup_test_dir();

    let config_path = dir.path().join(".qipu/config.toml");
    let config_content = r#"
version = 1
default_note_type = "fleeting"

[ontology]
mode = "extended"

[ontology.note_types.task]
description = "A task item"
usage = "Use for task tracking"

[ontology.link_types.depends-on]
description = "Dependency relationship"
inverse = "required-by"
usage = "A depends on B"
"#;
    fs::write(config_path, config_content).unwrap();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Task Note", "--type", "task"])
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

    assert!(stdout.contains("O mode=extended"));

    assert!(stdout.contains("T note_type=\"fleeting\""));
    assert!(stdout.contains("T note_type=\"task\" description=\"A task item\""));
    assert!(stdout.contains("U note_type=\"task\" usage=\"Use for task tracking\""));

    assert!(stdout.contains("L link_type=\"depends-on\" inverse=\"required-by\" description=\"Dependency relationship\""));
    assert!(stdout.contains("U link_type=\"depends-on\" usage=\"A depends on B\""));
}

#[test]
fn test_context_human_with_include_ontology() {
    let dir = setup_test_dir();

    let config_path = dir.path().join(".qipu/config.toml");
    let config_content = r#"
version = 1
default_note_type = "fleeting"

[ontology]
mode = "extended"

[ontology.note_types.task]
description = "A task item"
usage = "Use for task tracking"
"#;
    fs::write(config_path, config_content).unwrap();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Task Note", "--type", "task"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    let output = qipu()
        .current_dir(dir.path())
        .args(["context", "--note", &id, "--include-ontology"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("## Ontology"));
    assert!(stdout.contains("Mode: extended"));

    assert!(stdout.contains("### Note Types"));
    assert!(stdout.contains("fleeting"));
    assert!(stdout.contains("  task - A task item"));
    assert!(stdout.contains("    Usage: Use for task tracking"));
}
