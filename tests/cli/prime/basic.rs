use crate::support::{qipu, setup_test_dir};
use predicates::prelude::*;

#[test]
fn test_prime_empty_store() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .arg("prime")
        .assert()
        .success()
        .stdout(predicate::str::contains("Qipu Knowledge Store Primer"))
        .stdout(predicate::str::contains("About Qipu"))
        .stdout(predicate::str::contains("Quick Reference"))
        .stdout(predicate::str::contains("qipu list"))
        .stdout(predicate::str::contains("Session Protocol"))
        .stdout(predicate::str::contains("Before ending session:"))
        .stdout(predicate::str::contains(
            "Knowledge not committed is knowledge lost",
        ));
}

#[test]
fn test_prime_with_mocs() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args([
            "create",
            "Research Topics",
            "--type",
            "moc",
            "--tag",
            "research",
        ])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("prime")
        .assert()
        .success()
        .stdout(predicate::str::contains("Key Maps of Content"))
        .stdout(predicate::str::contains("Research Topics"));
}

#[test]
fn test_prime_with_recent_notes() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["create", "First Note", "--type", "fleeting"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Second Note", "--type", "permanent"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("prime")
        .assert()
        .success()
        .stdout(predicate::str::contains("Recently Updated Notes"))
        .stdout(predicate::str::contains("First Note"))
        .stdout(predicate::str::contains("Second Note"));
}

#[test]
fn test_prime_missing_store() {
    let dir = setup_test_dir();
    let nonexistent_store = dir.path().join("nonexistent-store");

    qipu()
        .current_dir(dir.path())
        .env("QIPU_STORE", &nonexistent_store)
        .arg("prime")
        .assert()
        .code(3)
        .stderr(predicate::str::contains("store not found"));
}

#[test]
fn test_prime_invalid_format() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "invalid", "prime"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn test_prime_success_exit_code_empty_store() {
    let dir = setup_test_dir();

    qipu().current_dir(dir.path()).arg("prime").assert().code(0);
}

#[test]
fn test_prime_success_exit_code_with_mocs() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test MOC", "--type", "moc"])
        .assert()
        .success();

    qipu().current_dir(dir.path()).arg("prime").assert().code(0);
}

#[test]
fn test_prime_success_exit_code_with_notes() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success();

    qipu().current_dir(dir.path()).arg("prime").assert().code(0);
}

#[test]
fn test_prime_success_exit_code_json_format() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "prime"])
        .assert()
        .code(0);
}

#[test]
fn test_prime_success_exit_code_records_format() {
    let dir = setup_test_dir();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "prime"])
        .assert()
        .code(0);
}
