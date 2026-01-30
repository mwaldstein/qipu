use crate::support::qipu;
use predicates::prelude::*;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_export_pdf_requires_output_file() {
    let dir = setup_test_dir();

    // Create a note
    let note_path = dir.path().join(".qipu/notes/qp-1111-test-note.md");
    fs::write(&note_path, "---\nid: qp-1111\ntitle: Test Note\n---\nBody").unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // PDF to stdout should fail
    qipu()
        .current_dir(dir.path())
        .args(["export", "--note", "qp-1111", "--pdf"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--pdf requires --output"));
}

#[test]
fn test_export_pdf_without_pandoc() {
    // This test verifies graceful failure when pandoc is not available
    // We skip if pandoc is actually installed
    if Command::new("pandoc").arg("--version").output().is_ok() {
        // Skip test if pandoc is installed
        return;
    }

    let dir = setup_test_dir();

    // Create a note
    let note_path = dir.path().join(".qipu/notes/qp-1111-test-note.md");
    fs::write(&note_path, "---\nid: qp-1111\ntitle: Test Note\n---\nBody").unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Export to PDF should fail with helpful message
    let output_pdf = dir.path().join("export.pdf");
    qipu()
        .current_dir(dir.path())
        .args([
            "export",
            "--note",
            "qp-1111",
            "--output",
            output_pdf.to_str().unwrap(),
            "--pdf",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("pandoc not found"))
        .stderr(predicate::str::contains("pandoc.org/installing"));
}
