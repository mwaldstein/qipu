use crate::cli::support::qipu;
use predicates::prelude::*;
use tempfile::tempdir;

// ============================================================================
// Prime command tests (per specs/llm-context.md)
// ============================================================================

#[test]
fn test_prime_empty_store() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("prime")
        .assert()
        .success()
        .stdout(predicate::str::contains("Qipu Knowledge Store Primer"))
        .stdout(predicate::str::contains("About Qipu"))
        .stdout(predicate::str::contains("Quick Reference"))
        .stdout(predicate::str::contains("qipu list"));
}

#[test]
fn test_prime_with_mocs() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a MOC
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
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create some notes
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
fn test_prime_json_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a MOC and a note
    qipu()
        .current_dir(dir.path())
        .args(["create", "Test MOC", "--type", "moc"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note", "--type", "fleeting"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "prime"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"store\""))
        .stdout(predicate::str::contains("\"primer\""))
        .stdout(predicate::str::contains("\"mocs\""))
        .stdout(predicate::str::contains("\"recent_notes\""))
        .stdout(predicate::str::contains("\"commands\""));
}

#[test]
fn test_prime_records_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a MOC and a note
    qipu()
        .current_dir(dir.path())
        .args(["create", "Test MOC", "--type", "moc"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note", "--type", "fleeting"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "prime"])
        .assert()
        .success()
        .stdout(predicate::str::contains("H qipu=1 records=1 store="))
        .stdout(predicate::str::contains("truncated=false"))
        .stdout(predicate::str::contains("D Qipu is"))
        .stdout(predicate::str::contains("C list"))
        .stdout(predicate::str::contains("M ")) // MOC record
        .stdout(predicate::str::contains("N ")); // Note record
}

#[test]
fn test_prime_missing_store() {
    let dir = tempdir().unwrap();
    // Use QIPU_STORE to prevent discovery of /tmp/.qipu from other tests
    let nonexistent_store = dir.path().join("nonexistent-store");

    // No init - should fail with exit code 3
    qipu()
        .current_dir(dir.path())
        .env("QIPU_STORE", &nonexistent_store)
        .arg("prime")
        .assert()
        .code(3)
        .stderr(predicate::str::contains("store not found"));
}
