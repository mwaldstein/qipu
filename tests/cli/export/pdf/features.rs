use crate::support::qipu;
use std::fs;
use tempfile::tempdir;

#[test]
#[ignore] // Requires pandoc to be installed
fn test_export_pdf_with_attachments() {
    let dir = setup_test_dir();

    // Create attachment
    let attachment_dir = dir.path().join(".qipu/attachments");
    fs::create_dir_all(&attachment_dir).unwrap();
    let attachment_file = attachment_dir.join("image.png");
    fs::write(&attachment_file, b"fake image data").unwrap();

    // Create note with attachment reference
    let note_path = dir.path().join(".qipu/notes/qp-1111-test-note.md");
    fs::write(
        &note_path,
        "---\nid: qp-1111\ntitle: Test Note\n---\n# Note\n\n![Image](../attachments/image.png)",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Export with attachments to PDF
    let output_pdf = dir.path().join("with-attachments.pdf");
    qipu()
        .current_dir(dir.path())
        .args([
            "export",
            "--note",
            "qp-1111",
            "--output",
            output_pdf.to_str().unwrap(),
            "--pdf",
            "--with-attachments",
        ])
        .assert()
        .success();

    // Verify PDF was created
    assert!(output_pdf.exists());
    let pdf_content = fs::read(&output_pdf).unwrap();
    assert!(
        pdf_content.starts_with(b"%PDF"),
        "output should be a valid PDF file"
    );

    // Verify attachment was copied
    let attachments_dir = dir.path().join("attachments");
    assert!(attachments_dir.exists());
    let copied_attachment = attachments_dir.join("image.png");
    assert!(copied_attachment.exists());
}

#[test]
#[ignore] // Requires pandoc to be installed
fn test_export_pdf_empty_content() {
    let dir = setup_test_dir();

    // Create note with empty body
    let note_path = dir.path().join(".qipu/notes/qp-1111-test-note.md");
    fs::write(&note_path, "---\nid: qp-1111\ntitle: Empty Note\n---\n").unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Export empty note to PDF
    let output_pdf = dir.path().join("empty.pdf");
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
    let pdf_content = fs::read(&output_pdf).unwrap();
    assert!(
        pdf_content.starts_with(b"%PDF"),
        "output should be a valid PDF file"
    );
}

#[test]
#[ignore] // Requires pandoc to be installed
fn test_export_pdf_special_characters() {
    let dir = setup_test_dir();

    // Create note with special characters
    let special_content = "# Note with Special Characters\n\n\
        Unicode: Café, naïve, 日本語\n\n\
        Math: E = mc², x² + y² = z²\n\n\
        Symbols: ©, ™, €, ¥, £\n\n\
        Code: `const λ = x => x * 2`";

    let note_path = dir.path().join(".qipu/notes/qp-1111-test-note.md");
    fs::write(
        &note_path,
        format!(
            "---\nid: qp-1111\ntitle: Special Chars\n---\n{}",
            special_content
        ),
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Export to PDF
    let output_pdf = dir.path().join("special-chars.pdf");
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
    let pdf_content = fs::read(&output_pdf).unwrap();
    assert!(
        pdf_content.starts_with(b"%PDF"),
        "output should be a valid PDF file"
    );
}
