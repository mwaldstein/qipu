use crate::support::{qipu, setup_test_dir};
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_doctor_ontology_invalid_note_type() {
    let dir = setup_test_dir();

    let note_content = r#"---
id: qp-note1
title: Test Note
type: invalid-type
---
Test content."#;

    fs::write(
        dir.path().join(".qipu/notes/qp-note1-invalid-type.md"),
        note_content,
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["doctor", "--check", "ontology"])
        .assert()
        .code(3)
        .stdout(predicate::str::contains("invalid-note-type"))
        .stdout(predicate::str::contains("invalid-type"));

    qipu()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("invalid-note-type").not());
}

#[test]
fn test_doctor_ontology_invalid_link_type() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Target Note"])
        .assert()
        .success();

    let source_note_content = r#"---
id: qp-source
title: Source Note
links:
  - type: invalid-link
    id: qp-target
---
Content."#;

    fs::write(
        dir.path().join(".qipu/notes/qp-source-source-note.md"),
        source_note_content,
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["doctor", "--check", "ontology"])
        .assert()
        .code(3)
        .stdout(predicate::str::contains("invalid-link-type"))
        .stdout(predicate::str::contains("invalid-link"));
}

#[test]
fn test_doctor_ontology_deprecated_graph_types() {
    let dir = setup_test_dir();

    let config_content = r#"[graph.types.custom-link]
cost = 1.5
"#;

    fs::write(dir.path().join(".qipu/config.toml"), config_content).unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["doctor", "--check", "ontology"])
        .assert()
        .success()
        .stdout(predicate::str::contains("deprecated-config"))
        .stdout(predicate::str::contains("[graph.types.custom-link]"))
        .stdout(predicate::str::contains(
            "[ontology.link_types.custom-link]",
        ));
}
