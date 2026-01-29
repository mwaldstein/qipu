use crate::cli::support::qipu;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_list_json_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "JSON List Test"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"title\": \"JSON List Test\""));
}

#[test]
fn test_list_records_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--tag", "test", "--tag", "example", "Test Note"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "list"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("H qipu=1 records=1"));
    assert!(stdout.contains("mode=list"));
    assert!(stdout.contains("notes=1"));
    assert!(stdout.contains("N qp-"));
    assert!(stdout.contains("\"Test Note\""));
    assert!(stdout.contains("tags=example,test"));
}

#[test]
fn test_list_records_format_empty() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "list"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("H qipu=1 records=1"));
    assert!(stdout.contains("mode=list"));
    assert!(stdout.contains("notes=0"));
}

#[test]
fn test_list_records_format_multiple_notes() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args([
            "create",
            "--type",
            "fleeting",
            "--tag",
            "urgent",
            "Fleeting Note",
        ])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "permanent", "Permanent Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "moc", "MOC Note"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "list"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("notes=3"));
    assert!(stdout.contains("fleeting"));
    assert!(stdout.contains("permanent"));
    assert!(stdout.contains("moc"));
    assert!(stdout.contains("\"Fleeting Note\""));
    assert!(stdout.contains("\"Permanent Note\""));
    assert!(stdout.contains("\"MOC Note\""));
    assert!(stdout.contains("tags=urgent"));
    assert!(stdout.matches("tags=-").count() >= 2);
}

#[test]
fn test_list_records_format_truncated_field() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "list"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("H qipu=1 records=1"),
        "list records output should have valid header"
    );
    assert!(
        stdout.contains("mode=list"),
        "list records output should contain mode=list"
    );
}
