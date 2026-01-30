use crate::support::{qipu, setup_test_dir};
use predicates::prelude::*;
use std::fs;

#[test]
fn test_export_records_truncated_field() {
    let dir = setup_test_dir();

    let note_path = dir.path().join(".qipu/notes/qp-1111-test-note.md");
    fs::write(&note_path, "---\nid: qp-1111\ntitle: Test Note\n---\nBody").unwrap();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "export", "--note", "qp-1111"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("truncated=false"),
        "export records output should contain truncated=false in header"
    );
}

#[test]
fn test_export_basic() {
    let dir = setup_test_dir();

    // Create a note with known ID
    let note_path = dir.path().join(".qipu/notes/qp-1111-test-note.md");
    fs::write(&note_path, "---\nid: qp-1111\ntitle: Test Note\n---\nBody").unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["export", "--note", "qp-1111"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Test Note"))
        .stdout(predicate::str::contains("qp-1111"));
}

#[test]
fn test_export_with_attachments() {
    let dir = setup_test_dir();

    // Create an attachment
    let attachments_dir = dir.path().join(".qipu/attachments");
    fs::create_dir_all(&attachments_dir).unwrap();
    fs::write(attachments_dir.join("test.png"), "image data").unwrap();

    // Create a note referencing the attachment
    let note_content = "See ![diagram](../attachments/test.png)";
    let note_path = dir.path().join(".qipu/notes/qp-1234-attachment-note.md");
    fs::write(
        &note_path,
        format!(
            "---\nid: qp-1234\ntitle: Attachment Note\n---\n{}",
            note_content
        ),
    )
    .unwrap();

    // Export with attachments
    qipu()
        .current_dir(dir.path())
        .args([
            "export",
            "--note",
            "qp-1234",
            "--output",
            "export.md",
            "--with-attachments",
        ])
        .assert()
        .success();

    // Verify attachment was copied
    assert!(dir.path().join("attachments/test.png").exists());
    assert_eq!(
        fs::read_to_string(dir.path().join("attachments/test.png")).unwrap(),
        "image data"
    );

    // Verify exported content has rewritten links (../attachments/ -> ./attachments/)
    let export_content = fs::read_to_string(dir.path().join("export.md")).unwrap();
    assert!(
        export_content.contains("./attachments/test.png"),
        "exported content should have rewritten attachment links to ./attachments/"
    );
    assert!(
        !export_content.contains("../attachments/test.png"),
        "exported content should not contain original ../attachments/ links"
    );
}
