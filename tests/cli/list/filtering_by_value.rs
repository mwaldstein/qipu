use crate::cli::support::{extract_id, qipu};
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_list_filter_by_min_value_all_match() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "High Value Note"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Medium Value Note"])
        .output()
        .unwrap();
    let medium_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["create", "Low Value Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &medium_id, "75"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["list", "--min-value", "50"])
        .assert()
        .success()
        .stdout(predicate::str::contains("High Value Note"))
        .stdout(predicate::str::contains("Medium Value Note"))
        .stdout(predicate::str::contains("Low Value Note"));
}

#[test]
fn test_list_filter_by_min_value_some_match() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "High Value Note"])
        .output()
        .unwrap();
    let high_id = extract_id(&output);

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Medium Value Note"])
        .output()
        .unwrap();
    let medium_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["create", "Low Value Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &high_id, "90"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &medium_id, "75"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["list", "--min-value", "70"])
        .assert()
        .success()
        .stdout(predicate::str::contains("High Value Note"))
        .stdout(predicate::str::contains("Medium Value Note"))
        .stdout(predicate::str::contains("Low Value Note").not());
}

#[test]
fn test_list_filter_by_min_value_none_match() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Note 1"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 2"])
        .output()
        .unwrap();
    let id2 = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id2, "30"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["list", "--min-value", "95"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No notes found"))
        .stdout(predicate::str::contains("Note 1").not())
        .stdout(predicate::str::contains("Note 2").not());
}

#[test]
fn test_list_filter_by_min_value_with_defaults() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Explicit High Value"])
        .output()
        .unwrap();
    let high_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["create", "Default Value Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &high_id, "80"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["list", "--min-value", "50"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Explicit High Value"))
        .stdout(predicate::str::contains("Default Value Note"));
}
