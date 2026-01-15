use crate::cli::support::qipu;
use predicates::prelude::*;
use tempfile::tempdir;

// ============================================================================
// Show command tests
// ============================================================================

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

    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();

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
        .args(["create", "Note Without Links"])
        .output()
        .unwrap();

    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Show with --links should work and show no links
    qipu()
        .current_dir(dir.path())
        .args(["show", &id, "--links"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Links for"))
        .stdout(predicate::str::contains("No links found"));
}

#[test]
fn test_show_links_with_links() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Source Note"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Target Note"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    // Add link from note1 to note2
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    // Rebuild index
    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success();

    // Show --links on source note should show outbound link
    qipu()
        .current_dir(dir.path())
        .args(["show", &id1, "--links"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Outbound links"))
        .stdout(predicate::str::contains(&id2))
        .stdout(predicate::str::contains("related"))
        .stdout(predicate::str::contains("typed"));

    // Show --links on target note should show inbound link (backlink)
    qipu()
        .current_dir(dir.path())
        .args(["show", &id2, "--links"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Inbound links"))
        .stdout(predicate::str::contains(&id1));
}

#[test]
fn test_show_links_json_format() {
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

    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();

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
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Target"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

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
