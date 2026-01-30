use crate::support::{extract_id, qipu};
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

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
