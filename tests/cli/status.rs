use crate::support::{qipu, setup_test_dir};
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_status_initialized_store_human() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Ready: yes"))
        .stdout(predicate::str::contains("Store: .qipu"))
        .stdout(predicate::str::contains("Notes: 0"))
        .stdout(predicate::str::contains("Index: total=0"))
        .stdout(predicate::str::contains("Database: schema_version="));
}

#[test]
fn test_status_missing_store_exits_3() {
    let dir = tempdir().unwrap();
    let missing_store = dir.path().join("missing-store");

    qipu()
        .current_dir(dir.path())
        .env("QIPU_STORE", &missing_store)
        .arg("status")
        .assert()
        .code(3)
        .stdout(predicate::str::contains("Ready: no"))
        .stdout(predicate::str::contains("Store: not found"))
        .stdout(predicate::str::contains("Next: qipu init"));
}

#[test]
fn test_status_missing_database_does_not_create_one() {
    let dir = setup_test_dir();
    let db_path = dir.path().join(".qipu").join("qipu.db");
    fs::remove_file(&db_path).unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .code(3)
        .stdout(predicate::str::contains("Ready: no"));

    assert!(!db_path.exists());
}

#[test]
fn test_status_json_format() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["status", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ready\": true"))
        .stdout(predicate::str::contains("\"store_found\": true"))
        .stdout(predicate::str::contains("\"store\": \".qipu\""))
        .stdout(predicate::str::contains("\"database\""))
        .stdout(predicate::str::contains("\"index\""));
}

#[test]
fn test_status_records_format() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["status", "--format", "records"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "H qipu=1 records=1 mode=status ready=true",
        ))
        .stdout(predicate::str::contains("store=.qipu"))
        .stdout(predicate::str::contains("database.schema_version="))
        .stdout(predicate::str::contains("index.total=0"));
}

#[test]
fn test_status_honors_explicit_store() {
    let dir = setup_test_dir();
    let other_dir = tempdir().unwrap();
    let store = dir.path().join(".qipu");

    qipu()
        .current_dir(other_dir.path())
        .arg("--store")
        .arg(&store)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Ready: yes"))
        .stdout(predicate::str::contains("Store:"))
        .stdout(predicate::str::contains(".qipu"));
}

#[test]
fn test_status_help_lists_maintenance_command() {
    qipu()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Maintenance commands:"))
        .stdout(predicate::str::contains(
            "status      Check whether a usable qipu store is available",
        ));
}
