use crate::cli::support::qipu;
use predicates::prelude::*;
use tempfile::tempdir;

// ============================================================================
// Output format tests
// ============================================================================

#[test]
fn test_context_json_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "JSON Context Note"])
        .output()
        .unwrap();
    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "context", "--note", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"store\""))
        .stdout(predicate::str::contains("\"notes\""))
        .stdout(predicate::str::contains("\"title\": \"JSON Context Note\""));
}

#[test]
fn test_context_records_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Records Context Note"])
        .output()
        .unwrap();
    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "context", "--note", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("H qipu=1 records=1 store="))
        .stdout(predicate::str::contains("N "))
        .stdout(predicate::str::contains("Records Context Note"));
}

#[test]
fn test_context_records_escapes_quotes_in_title() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Title with \"quotes\" inside"])
        .output()
        .unwrap();
    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "context", "--note", &id])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // The title should be escaped with backslash before quotes
    assert!(
        stdout.contains(r#"Title with \"quotes\" inside"#),
        "Expected escaped quotes in title, got: {}",
        stdout
    );

    // Ensure it's not double-escaped or unescaped
    assert!(
        !stdout.contains(r#"Title with ""quotes"" inside"#),
        "Title should not be double-quoted"
    );
    assert!(
        !stdout.contains(r#"Title with "quotes" inside"#) || stdout.contains(r#"\"quotes\""#),
        "Quotes must be escaped"
    );
}
