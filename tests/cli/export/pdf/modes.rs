use crate::support::qipu;
use std::fs;
use tempfile::tempdir;

#[test]
#[ignore] // Requires pandoc to be installed
fn test_export_pdf_outline_mode() {
    let dir = setup_test_dir();

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
    let dir = setup_test_dir();

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
