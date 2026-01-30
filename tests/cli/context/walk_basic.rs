use crate::support::{create_note, qipu, setup_test_dir};
use predicates::prelude::*;

#[test]
fn test_context_walk_basic() {
    let dir = setup_test_dir();

    let root_id = create_note(&dir, "Root Note");
    let child_id = create_note(&dir, "Child Note");
    let grandchild_id = create_note(&dir, "Grandchild Note");

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &root_id, &child_id, "--type", "supports"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "add",
            &child_id,
            &grandchild_id,
            "--type",
            "supports",
        ])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--walk",
            &root_id,
            "--walk-max-hops",
            "2",
            "--related",
            "0",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Root Note"))
        .stdout(predicate::str::contains("Child Note"))
        .stdout(predicate::str::contains("Grandchild Note"));
}

#[test]
fn test_context_walk_max_hops() {
    let dir = setup_test_dir();

    let root_id = create_note(&dir, "Root Note");
    let child_id = create_note(&dir, "Child Note");
    let grandchild_id = create_note(&dir, "Grandchild Note");

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &root_id, &child_id, "--type", "supports"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "add",
            &child_id,
            &grandchild_id,
            "--type",
            "supports",
        ])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--walk",
            &root_id,
            "--walk-max-hops",
            "1",
            "--related",
            "0",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Root Note"))
        .stdout(predicate::str::contains("Child Note"))
        .stdout(predicate::str::contains("Grandchild Note").not());
}

#[test]
fn test_context_walk_direction() {
    let dir = setup_test_dir();

    let root_id = create_note(&dir, "Root Note");
    let child_id = create_note(&dir, "Child Note");

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &root_id, &child_id, "--type", "supports"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--walk",
            &root_id,
            "--walk-direction",
            "out",
            "--related",
            "0",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Root Note"))
        .stdout(predicate::str::contains("Child Note"));

    qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--walk",
            &child_id,
            "--walk-direction",
            "in",
            "--related",
            "0",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Child Note"))
        .stdout(predicate::str::contains("Root Note"));
}

#[test]
fn test_context_walk_with_type_filter() {
    let dir = setup_test_dir();

    let root_id = create_note(&dir, "Root Note");
    let child1_id = create_note(&dir, "Child Note 1");
    let child2_id = create_note(&dir, "Child Note 2");

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &root_id, &child1_id, "--type", "supports"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "add",
            &root_id,
            &child2_id,
            "--type",
            "derived-from",
        ])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--walk",
            &root_id,
            "--walk-type",
            "supports",
            "--related",
            "0",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Root Note"))
        .stdout(predicate::str::contains("Child Note 1"))
        .stdout(predicate::str::contains("Child Note 2").not());
}

#[test]
fn test_context_walk_json_format() {
    let dir = setup_test_dir();

    let root_id = create_note(&dir, "Root Note");
    let child_id = create_note(&dir, "Child Note");

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &root_id, &child_id, "--type", "supports"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--walk",
            &root_id,
            "--format",
            "json",
            "--related",
            "0",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""notes":"#))
        .stdout(predicate::str::contains(r#""id":"#));
}
