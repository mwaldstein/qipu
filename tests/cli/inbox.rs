use crate::cli::support::qipu;
use predicates::prelude::*;
use tempfile::tempdir;

// ============================================================================
// Inbox command tests
// ============================================================================

#[test]
fn test_inbox_empty() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("inbox")
        .assert()
        .success()
        .stdout(predicate::str::contains("Inbox is empty"));
}

#[test]
fn test_inbox_shows_fleeting() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "fleeting", "Inbox Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("inbox")
        .assert()
        .success()
        .stdout(predicate::str::contains("Inbox Note"));
}

#[test]
fn test_inbox_excludes_permanent() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "permanent", "Not In Inbox"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("inbox")
        .assert()
        .success()
        .stdout(predicate::str::contains("Inbox is empty"));
}

#[test]
fn test_inbox_exclude_linked() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a MOC
    let moc_output = qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "moc", "Project MOC"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let moc_id = String::from_utf8_lossy(&moc_output).trim().to_string();

    // Create two fleeting notes
    let fleeting1_output = qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "fleeting", "Linked Note"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let fleeting1_id = String::from_utf8_lossy(&fleeting1_output)
        .trim()
        .to_string();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "fleeting", "Unlinked Note"])
        .assert()
        .success();

    // Link the first fleeting note from the MOC
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &moc_id, &fleeting1_id, "--type", "related"])
        .assert()
        .success();

    // Build index to make sure links are tracked
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Without --exclude-linked, should show both fleeting notes
    qipu()
        .current_dir(dir.path())
        .arg("inbox")
        .assert()
        .success()
        .stdout(predicate::str::contains("Linked Note"))
        .stdout(predicate::str::contains("Unlinked Note"));

    // With --exclude-linked, should only show the unlinked note
    qipu()
        .current_dir(dir.path())
        .args(["inbox", "--exclude-linked"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Unlinked Note")
                .and(predicate::str::contains("Linked Note").not()),
        );
}
