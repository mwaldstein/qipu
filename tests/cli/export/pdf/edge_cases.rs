use crate::support::qipu;
use std::fs;
use tempfile::tempdir;

#[test]
#[ignore] // Requires pandoc to be installed
fn test_export_pdf_nonexistent_output_directory() {
    let dir = setup_test_dir();

    // Create a note
    let note_path = dir.path().join(".qipu/notes/qp-1111-test-note.md");
    fs::write(
        &note_path,
        "---\nid: qp-1111\ntitle: Test Note\n---\n# Test\n\nContent",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Export to PDF in a non-existent directory
    let output_pdf = dir.path().join("subdir/nonexistent/export.pdf");
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

    // Verify PDF was created and directory was created
    assert!(output_pdf.exists());
    assert!(output_pdf.parent().unwrap().exists());
    let pdf_content = fs::read(&output_pdf).unwrap();
    assert!(
        pdf_content.starts_with(b"%PDF"),
        "output should be a valid PDF file"
    );
}
