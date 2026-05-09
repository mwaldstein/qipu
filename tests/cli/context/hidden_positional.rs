//! Tests for hidden context positional-note compatibility alias.
use crate::support::{extract_id, qipu, setup_test_dir};
use predicates::prelude::*;

#[test]
fn test_context_hidden_positional_note_alias_selects_note() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Hidden Alias Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["context", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Qipu Context Bundle"))
        .stdout(predicate::str::contains("Hidden Alias Note"))
        .stdout(predicate::str::contains(&id));
}

#[test]
fn test_context_hidden_positional_note_alias_absent_from_help() {
    qipu()
        .args(["context", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: qipu context [OPTIONS]"))
        .stdout(predicate::str::contains("[ID]").not())
        .stdout(predicate::str::contains("Select notes by ID"));
}

#[test]
fn test_context_hidden_positional_note_alias_missing_note_is_data_error() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["context", "qp-does-not-exist"])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("not found"));
}
