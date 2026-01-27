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

#[test]
#[ignore] // Requires pandoc to be installed
fn test_export_pdf_outline_mode() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create notes
    let note_a_path = dir.path().join(".qipu/notes/qp-aaaa-note-a.md");
    fs::write(
        &note_a_path,
        "---\nid: qp-aaaa\ntitle: Note A\n---\n# Note A\n\nContent of Note A",
    )
    .unwrap();

    let note_b_path = dir.path().join(".qipu/notes/qp-bbbb-note-b.md");
    fs::write(
        &note_b_path,
        "---\nid: qp-bbbb\ntitle: Note B\n---\n# Note B\n\nContent of Note B",
    )
    .unwrap();

    // Create MOC
    let moc_path = dir.path().join(".qipu/mocs/qp-moc1-outline.md");
    fs::write(
        &moc_path,
        "---\nid: qp-moc1\ntitle: Outline\ntype: moc\n---\n[[qp-bbbb]]\n[[qp-aaaa]]\n",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Export outline to PDF
    let output_pdf = dir.path().join("outline.pdf");
    qipu()
        .current_dir(dir.path())
        .args([
            "export",
            "--moc",
            "qp-moc1",
            "--mode",
            "outline",
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
fn test_export_pdf_with_anchor_links() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create notes with links
    let note_a_path = dir.path().join(".qipu/notes/qp-aaaa-note-a.md");
    fs::write(
        &note_a_path,
        "---\nid: qp-aaaa\ntitle: Note A\n---\n# Note A\n\nSee [[qp-bbbb]]",
    )
    .unwrap();

    let note_b_path = dir.path().join(".qipu/notes/qp-bbbb-note-b.md");
    fs::write(
        &note_b_path,
        "---\nid: qp-bbbb\ntitle: Note B\n---\n# Note B\n\nContent of Note B",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Export bundle with anchor links to PDF
    let output_pdf = dir.path().join("anchors.pdf");
    qipu()
        .current_dir(dir.path())
        .args([
            "export",
            "--note",
            "qp-aaaa",
            "--note",
            "qp-bbbb",
            "--mode",
            "bundle",
            "--link-mode",
            "anchors",
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
fn test_export_pdf_with_attachments() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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

#[test]
#[ignore] // Requires pandoc to be installed
fn test_export_pdf_nonexistent_output_directory() {
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
