use crate::cli::support::{extract_id, qipu};
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

// ============================================================================
// Custom Ontology Integration Tests
// ============================================================================

#[test]
fn test_ontology_show_default_mode() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("ontology")
        .arg("show")
        .assert()
        .success()
        .stdout(predicate::str::contains("Ontology mode: default"))
        .stdout(predicate::str::contains("fleeting"))
        .stdout(predicate::str::contains("literature"))
        .stdout(predicate::str::contains("permanent"))
        .stdout(predicate::str::contains("moc"))
        .stdout(predicate::str::contains("related"))
        .stdout(predicate::str::contains("supports"));
}

#[test]
fn test_ontology_show_extended_mode() {
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

[ontology.note_types.custom-note]
description = "A custom note type"
usage = "Use for custom notes"

[ontology.link_types.custom-link]
description = "A custom link type"
inverse = "inverse-custom-link"
usage = "Use for custom links"
"#;
    fs::write(config_path, config_content).unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("ontology")
        .arg("show")
        .assert()
        .success()
        .stdout(predicate::str::contains("Ontology mode: extended"))
        .stdout(predicate::str::contains("custom-note"))
        .stdout(predicate::str::contains("A custom note type"))
        .stdout(predicate::str::contains("custom-link"))
        .stdout(predicate::str::contains("A custom link type"))
        .stdout(predicate::str::contains("inverse-custom-link"));
}

#[test]
fn test_ontology_show_replacement_mode() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let config_path = dir.path().join(".qipu/config.toml");
    let config_content = r#"
version = 1
default_note_type = "custom-type"

[ontology]
mode = "replacement"

[ontology.note_types.custom-type]
description = "Only custom type"

[ontology.link_types.custom-link]
description = "Only custom link"
inverse = "inverse-link"
"#;
    fs::write(config_path, config_content).unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("ontology")
        .arg("show")
        .assert()
        .success()
        .stdout(predicate::str::contains("Ontology mode: replacement"))
        .stdout(predicate::str::contains("custom-type"))
        .stdout(predicate::str::contains("custom-link"));
}

#[test]
fn test_ontology_show_json_format() {
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

[ontology.note_types.custom-note]
description = "A custom note type"

[ontology.link_types.custom-link]
description = "A custom link type"
inverse = "inverse-link"
"#;
    fs::write(config_path, config_content).unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["ontology", "show", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""mode": "extended""#))
        .stdout(predicate::str::contains(r#""name": "custom-note""#))
        .stdout(predicate::str::contains(
            r#""description": "A custom note type""#,
        ))
        .stdout(predicate::str::contains(r#""name": "custom-link""#))
        .stdout(predicate::str::contains(r#""inverse": "inverse-link""#));
}

#[test]
fn test_ontology_show_records_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["ontology", "show", "--format", "records"])
        .assert()
        .success()
        .stdout(predicate::str::contains("H qipu=1 records=1 store="))
        .stdout(predicate::str::contains("mode=ontology.show"))
        .stdout(predicate::str::contains("O mode=default"))
        .stdout(predicate::str::contains("N note_type=\"fleeting\""))
        .stdout(predicate::str::contains("L link_type=\"related\""));
}

#[test]
fn test_create_with_custom_note_type() {
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
"#;
    fs::write(config_path, config_content).unwrap();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Task Note", "--type", "task"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let note_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .arg("show")
        .arg(&note_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Task Note"))
        .stdout(predicate::str::contains("task"));
}

#[test]
fn test_link_with_custom_link_type() {
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

[ontology.link_types.depends-on]
description = "Dependency relationship"
inverse = "required-by"
"#;
    fs::write(config_path, config_content).unwrap();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Task A"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Task B"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "depends-on"])
        .assert()
        .success();
}

#[test]
fn test_invalid_note_type_error() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note", "--type", "invalid-type"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid note type"))
        .stderr(predicate::str::contains("fleeting"))
        .stderr(predicate::str::contains("literature"))
        .stderr(predicate::str::contains("permanent"))
        .stderr(predicate::str::contains("moc"));
}

#[test]
fn test_invalid_link_type_error() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note A"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note B"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "invalid-link"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid link type"));
}

#[test]
fn test_prime_shows_custom_ontology() {
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
usage = "Track tasks"

[ontology.link_types.depends-on]
description = "Dependency relationship"
inverse = "required-by"
usage = "Note B depends on Note A"
"#;
    fs::write(config_path, config_content).unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("prime")
        .assert()
        .success()
        .stdout(predicate::str::contains("## Ontology"))
        .stdout(predicate::str::contains("Mode: extended"))
        .stdout(predicate::str::contains("task - A task item"))
        .stdout(predicate::str::contains("Usage: Track tasks"))
        .stdout(predicate::str::contains(
            "depends-on -> required-by (Dependency relationship)",
        ))
        .stdout(predicate::str::contains("Usage: Note B depends on Note A"));
}

#[test]
fn test_graph_types_backward_compatibility() {
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

[graph.types.custom-link]
description = "Custom link via graph.types"
inverse = "inverse-custom"
cost = 0.5
"#;
    fs::write(config_path, config_content).unwrap();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note A"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note B"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "custom-link"])
        .assert()
        .success();
}

#[test]
fn test_pack_roundtrip_with_custom_ontology() {
    let dir1 = tempdir().unwrap();
    let dir2 = tempdir().unwrap();
    let pack_file = dir1.path().join("test.pack.json");

    qipu()
        .current_dir(dir1.path())
        .arg("init")
        .assert()
        .success();

    let config_path = dir1.path().join(".qipu/config.toml");
    let config_content = r#"
version = 1
default_note_type = "fleeting"

[ontology]
mode = "extended"

[ontology.note_types.task]
description = "A task item"

[ontology.link_types.depends-on]
description = "Dependency relationship"
inverse = "required-by"
"#;
    fs::write(config_path, config_content).unwrap();

    let output1 = qipu()
        .current_dir(dir1.path())
        .args(["create", "Task A", "--type", "task"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir1.path())
        .args(["create", "Task B", "--type", "task"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    qipu()
        .current_dir(dir1.path())
        .args(["link", "add", &id1, &id2, "--type", "depends-on"])
        .assert()
        .success();

    qipu()
        .current_dir(dir1.path())
        .args([
            "dump",
            "--output",
            pack_file.to_string_lossy().as_ref(),
            "--format",
            "json",
        ])
        .assert()
        .success();

    qipu()
        .current_dir(dir2.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir2.path())
        .args([
            "load",
            "--apply-config",
            pack_file.to_string_lossy().as_ref(),
        ])
        .assert()
        .success();

    qipu()
        .current_dir(dir2.path())
        .arg("show")
        .arg(&id1)
        .assert()
        .success()
        .stdout(predicate::str::contains("Task A"));

    qipu()
        .current_dir(dir2.path())
        .arg("ontology")
        .arg("show")
        .assert()
        .success()
        .stdout(predicate::str::contains("Ontology mode: extended"))
        .stdout(predicate::str::contains("task"))
        .stdout(predicate::str::contains("depends-on"));
}

#[test]
fn test_replacement_mode_rejects_standard_types() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let config_path = dir.path().join(".qipu/config.toml");
    let config_content = r#"
version = 1
default_note_type = "custom-type"

[ontology]
mode = "replacement"

[ontology.note_types.custom-type]
description = "Only custom type"

[ontology.link_types.custom-link]
description = "Only custom link"
inverse = "inverse-link"
"#;
    fs::write(config_path, config_content).unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note", "--type", "fleeting"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid note type"));

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note A"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note B"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid link type"));
}

#[test]
fn test_extended_mode_allows_standard_and_custom_types() {
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

[ontology.link_types.depends-on]
description = "Dependency relationship"
inverse = "required-by"
"#;
    fs::write(config_path, config_content).unwrap();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Standard Note", "--type", "fleeting"])
        .output()
        .unwrap();
    assert!(output1.status.success());

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Task Note", "--type", "task"])
        .output()
        .unwrap();
    assert!(output2.status.success());

    let id1 = extract_id(&output1);
    let id2 = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "depends-on"])
        .assert()
        .success();
}
