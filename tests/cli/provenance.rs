use crate::cli::support::qipu;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_create_with_provenance() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "create",
            "--author",
            "Alice",
            "--source",
            "https://example.com",
            "--generated-by",
            "gpt-4",
            "--verified",
            "true",
            "Provenance Note",
        ])
        .output()
        .unwrap();

    let id = String::from_utf8(output.stdout).unwrap().trim().to_string();

    // Verify metadata in JSON output
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"author\": \"Alice\""))
        .stdout(predicate::str::contains("\"verified\": true"));
}

#[test]
fn test_verify_command() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();

    let id = String::from_utf8(output.stdout).unwrap().trim().to_string();

    // Toggle verify
    qipu()
        .current_dir(dir.path())
        .args(["verify", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("verified: true"));

    // Explicitly set to false
    qipu()
        .current_dir(dir.path())
        .args(["verify", &id, "--status", "false"])
        .assert()
        .success()
        .stdout(predicate::str::contains("verified: false"));
}

#[test]
fn test_context_prioritizes_verified() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create an unverified note first
    qipu()
        .current_dir(dir.path())
        .args(["create", "--tag", "test", "Unverified Note"])
        .assert()
        .success();

    // Create a verified note second
    qipu()
        .current_dir(dir.path())
        .args([
            "create",
            "--tag",
            "test",
            "--verified",
            "true",
            "Verified Note",
        ])
        .assert()
        .success();

    // Sync to build index
    qipu()
        .current_dir(dir.path())
        .arg("sync")
        .assert()
        .success();

    // Run context command. Verified note should come first.
    qipu()
        .current_dir(dir.path())
        .args(["context", "--tag", "test"])
        .assert()
        .success()
        .stdout(predicate::str::is_match("Verified Note[\\s\\S]*Unverified Note").unwrap());
}
