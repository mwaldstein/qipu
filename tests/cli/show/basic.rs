use crate::support::{extract_id, qipu};
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_show_note() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create and capture ID
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Show Test"])
        .output()
        .unwrap();

    let id = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap()
        .to_string();

    // Show should display the note
    qipu()
        .current_dir(dir.path())
        .args(["show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Show Test"))
        .stdout(predicate::str::contains(&id));
}

#[test]
fn test_show_nonexistent() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["show", "qp-nonexistent"])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_show_links_no_links() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Note with Links"])
        .output()
        .unwrap();

    let id = extract_id(&output);

    // Show --links with JSON format
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id, "--links"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"links\""))
        .stdout(predicate::str::contains(&id));
}

#[test]
fn test_show_links_records_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes and link them
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Source"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Target"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success();

    // Show --links with records format should include header and edge lines
    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "show", &id1, "--links"])
        .assert()
        .success()
        .stdout(predicate::str::contains("H qipu=1"))
        .stdout(predicate::str::contains("mode=show.links"))
        .stdout(predicate::str::contains("E "));
}
