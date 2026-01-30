use crate::support::{extract_id, qipu};
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_link_tree_type_filter() {
    let dir = setup_test_dir();

    let output_root = qipu()
        .current_dir(dir.path())
        .args(["create", "Root"])
        .output()
        .unwrap();
    let id_root = extract_id(&output_root);

    let output_child1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Child 1"])
        .output()
        .unwrap();
    let id_child1 = extract_id(&output_child1);

    let output_child2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Child 2"])
        .output()
        .unwrap();
    let id_child2 = extract_id(&output_child2);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_root, &id_child1, "--type", "supports"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_root, &id_child2, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id_root, "--type", "supports"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Child 1"))
        .stdout(predicate::str::contains("Child 2").not());
}

#[test]
fn test_link_tree_exclude_type_filter() {
    let dir = setup_test_dir();

    let output_root = qipu()
        .current_dir(dir.path())
        .args(["create", "Root"])
        .output()
        .unwrap();
    let id_root = extract_id(&output_root);

    let output_child1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Child 1"])
        .output()
        .unwrap();
    let id_child1 = extract_id(&output_child1);

    let output_child2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Child 2"])
        .output()
        .unwrap();
    let id_child2 = extract_id(&output_child2);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_root, &id_child1, "--type", "supports"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_root, &id_child2, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id_root, "--exclude-type", "supports"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Child 2"))
        .stdout(predicate::str::contains("Child 1").not());
}

#[test]
fn test_link_tree_typed_only() {
    let dir = setup_test_dir();

    let output_child1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Typed Child"])
        .output()
        .unwrap();
    let id_child1 = extract_id(&output_child1);

    let output_child2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Inline Child"])
        .output()
        .unwrap();
    let id_child2 = extract_id(&output_child2);

    let output_root = qipu()
        .current_dir(dir.path())
        .args(["capture", "--title", "Root"])
        .write_stdin(format!("See [[{}]]", id_child2))
        .output()
        .unwrap();
    let id_root = extract_id(&output_root);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_root, &id_child1, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id_root, "--typed-only"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Typed Child"))
        .stdout(predicate::str::contains("Inline Child").not());
}

#[test]
fn test_link_tree_inline_only() {
    let dir = setup_test_dir();

    let output_child1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Typed Child"])
        .output()
        .unwrap();
    let id_child1 = extract_id(&output_child1);

    let output_child2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Inline Child"])
        .output()
        .unwrap();
    let id_child2 = extract_id(&output_child2);

    let output_root = qipu()
        .current_dir(dir.path())
        .args(["capture", "--title", "Root"])
        .write_stdin(format!("See [[{}]]", id_child2))
        .output()
        .unwrap();
    let id_root = extract_id(&output_root);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_root, &id_child1, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id_root, "--inline-only"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Inline Child"))
        .stdout(predicate::str::contains("Typed Child").not());
}
