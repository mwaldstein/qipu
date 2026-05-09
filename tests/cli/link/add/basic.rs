use crate::support::{extract_id, qipu, setup_test_dir};
use predicates::prelude::*;

#[test]
fn test_link_add_and_list() {
    let dir = setup_test_dir();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Source Note"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Target Note"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "supports"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added link"));

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id1])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id2))
        .stdout(predicate::str::contains("supports"));

    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id2])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id1))
        .stdout(predicate::str::contains("supported-by"))
        .stdout(predicate::str::contains("(virtual)"));

    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id2, "--no-semantic-inversion"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id1))
        .stdout(predicate::str::contains("supports"))
        .stdout(predicate::str::contains("<-"));
}

#[test]
fn test_link_hidden_add_shorthand_adds_typed_link() {
    let dir = setup_test_dir();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Source Note"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Target Note"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["link", &id1, &id2, "--type", "supports"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added link"));

    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id1])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id2))
        .stdout(predicate::str::contains("supports"));
}

#[test]
fn test_link_hidden_add_shorthand_accepts_custom_note_ids() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--id", "note-a", "Source Note"])
        .assert()
        .success();
    qipu()
        .current_dir(dir.path())
        .args(["create", "--id", "note-b", "Target Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "note-a", "note-b", "-T", "supports"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added link"));

    qipu()
        .current_dir(dir.path())
        .args(["link", "list", "note-a"])
        .assert()
        .success()
        .stdout(predicate::str::contains("note-b"))
        .stdout(predicate::str::contains("supports"));
}

#[test]
fn test_link_hidden_add_shorthand_absent_from_help() {
    qipu()
        .args(["link", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("add          Add a typed link"))
        .stdout(predicate::str::contains("<from> <to> --type").not());
}

#[test]
fn test_link_hidden_add_shorthand_requires_type() {
    let dir = setup_test_dir();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Source Note"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Target Note"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["link", &id1, &id2])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "qipu link add <from> <to> --type <type>",
        ));
}

#[test]
fn test_link_hidden_add_shorthand_does_not_swallow_typos() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Target Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["link", "ad", &id, "--type", "related"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized link command"))
        .stderr(predicate::str::contains(
            "qipu link add <from> <to> --type <type>",
        ));
}

#[test]
fn test_link_add_idempotent() {
    let dir = setup_test_dir();

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
        .success()
        .stdout(predicate::str::contains("Added link"));

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success()
        .stdout(predicate::str::contains("already exists"));
}

#[test]
fn test_unknown_type_fallback_inversion() {
    let dir = setup_test_dir();

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
        .args(["link", "add", &id1, &id2, "--type", "custom-unknown"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid link type"));
}

#[test]
fn test_link_add_rejects_self_link() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Self Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id, &id, "--type", "related"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("self-link"));
}
