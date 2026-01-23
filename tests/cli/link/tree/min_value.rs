use crate::cli::support::{extract_id, qipu};
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_link_tree_min_value_filter_all_match() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Root Note"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "High Value Note"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id1, "90"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id2, "95"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id1, "--min-value", "80"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Root Note"))
        .stdout(predicate::str::contains("High Value Note"));
}

#[test]
fn test_link_tree_min_value_filter_some_match() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Root Note"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Low Value Note"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    let output3 = qipu()
        .current_dir(dir.path())
        .args(["create", "High Value Note"])
        .output()
        .unwrap();
    let id3 = extract_id(&output3);

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id1, "90"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id2, "30"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id3, "95"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id3, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id1, "--min-value", "80"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Root Note"))
        .stdout(predicate::str::contains("High Value Note"))
        .stdout(predicate::str::contains("Low Value Note").not());
}

#[test]
fn test_link_tree_min_value_filter_with_defaults() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Root Note"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Default Value Note"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id1, "90"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id1, "--min-value", "50"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Root Note"))
        .stdout(predicate::str::contains("Default Value Note"));
}

#[test]
fn test_link_tree_min_value_filter_excludes_root() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Low Value Root"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "High Value Child"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id1, "20"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id2, "90"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id1, "--min-value", "80"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Low Value Root").not())
        .stdout(predicate::str::contains("High Value Child").not())
        .stdout(predicate::str::contains("No notes found"));
}
