use crate::cli::support::qipu;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_export_pdf_requires_output_file() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
#[ignore] // Requires pandoc to be installed
fn test_export_pdf_basic() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note
    let note_path = dir.path().join(".qipu/notes/qp-1111-test-note.md");
    fs::write(
        &note_path,
        "---\nid: qp-1111\ntitle: Test Note\n---\n# Body\n\nThis is a test note.",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Export to PDF
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
        .success();

    // Verify PDF was created
    assert!(output_pdf.exists());

    // Verify it's a PDF file (starts with %PDF)
    let pdf_content = fs::read(&output_pdf).unwrap();
    assert!(
        pdf_content.starts_with(b"%PDF"),
        "output should be a valid PDF file"
    );
}

#[test]
#[ignore] // Requires pandoc to be installed
fn test_export_pdf_bundle_mode() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create multiple notes
    let note_a_path = dir.path().join(".qipu/notes/qp-aaaa-note-a.md");
    fs::write(
        &note_a_path,
        "---\nid: qp-aaaa\ntitle: Note A\n---\n# Note A\n\nFirst note content",
    )
    .unwrap();

    let note_b_path = dir.path().join(".qipu/notes/qp-bbbb-note-b.md");
    fs::write(
        &note_b_path,
        "---\nid: qp-bbbb\ntitle: Note B\n---\n# Note B\n\nSecond note content",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Export multiple notes to PDF
    let output_pdf = dir.path().join("bundle.pdf");
    qipu()
        .current_dir(dir.path())
        .args([
            "export",
            "--note",
            "qp-aaaa",
            "--note",
            "qp-bbbb",
            "--output",
            output_pdf.to_str().unwrap(),
            "--mode",
            "bundle",
            "--pdf",
        ])
        .assert()
        .success();

    // Verify PDF was created
    assert!(output_pdf.exists());

    // Verify it's a PDF file
    let pdf_content = fs::read(&output_pdf).unwrap();
    assert!(
        pdf_content.starts_with(b"%PDF"),
        "output should be a valid PDF file"
    );
}

#[test]
fn test_export_pdf_without_pandoc() {
    // This test verifies graceful failure when pandoc is not available
    // We skip if pandoc is actually installed
    use std::process::Command;
    if Command::new("pandoc").arg("--version").output().is_ok() {
        // Skip test if pandoc is installed
        return;
    }

    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
