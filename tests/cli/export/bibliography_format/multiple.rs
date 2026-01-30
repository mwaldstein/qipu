use crate::support::{qipu, setup_test_dir};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_export_bibliography_bibtex_multiple_sources() {
    let dir = setup_test_dir();

    let note_a_path = dir.path().join(".qipu/notes/qp-aaaa-note-a.md");
    fs::write(
        &note_a_path,
        "---\nid: qp-aaaa\ntitle: Note A\nsources:\n  - url: https://example.com/alpha\n    title: Alpha Article\n---\nBody A",
    )
    .unwrap();

    let note_b_path = dir.path().join(".qipu/notes/qp-bbbb-note-b.md");
    fs::write(
        &note_b_path,
        "---\nid: qp-bbbb\ntitle: Note B\nsources:\n  - url: https://example.com/beta\n    title: Beta Article\n    accessed: 2024-02-01\n  - url: https://example.com/gamma\n---\nBody B",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let result = qipu()
        .current_dir(dir.path())
        .args([
            "export",
            "--note",
            "qp-aaaa",
            "--note",
            "qp-bbbb",
            "--mode",
            "bibliography",
            "--bib-format",
            "bibtex",
        ])
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);

    assert!(output.contains("title = {Alpha Article}"));
    assert!(output.contains("title = {Beta Article}"));
    assert!(output.contains("url = {https://example.com/gamma}"));

    let alpha_pos = output.find("https://example.com/alpha").unwrap();
    let beta_pos = output.find("https://example.com/beta").unwrap();
    let gamma_pos = output.find("https://example.com/gamma").unwrap();
    assert!(alpha_pos < beta_pos);
    assert!(beta_pos < gamma_pos);
}
