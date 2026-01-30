use crate::support::{qipu, setup_test_dir};
use std::fs;

#[test]
#[ignore] // Requires pandoc to be installed
fn test_export_pdf_basic() {
    let dir = setup_test_dir();

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
    let dir = setup_test_dir();

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
