use crate::support::setup_test_dir;
use crate::support::{extract_id, qipu};
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_replacement_mode_rejects_standard_types() {
    let dir = setup_test_dir();

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
    let dir = setup_test_dir();

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
