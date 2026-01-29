use crate::cli::support::{extract_id, qipu};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_context_json_with_default_ontology() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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

#[test]
fn test_context_json_with_extended_ontology() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
fn test_context_json_with_replacement_ontology() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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

#[test]
fn test_context_human_with_include_ontology() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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

#[test]
fn test_context_json_without_include_ontology() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
