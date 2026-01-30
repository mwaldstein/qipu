use crate::support::{qipu, setup_test_dir};
use std::fs;

#[test]
fn test_export_bibliography_bibtex_special_chars_in_title() {
    let dir = setup_test_dir();

    let note_path = dir.path().join(".qipu/notes/qp-aaaa-title-chars.md");
    fs::write(
        &note_path,
        "---\nid: qp-aaaa\ntitle: Title Chars\nsources:\n  - url: https://example.com/article\n    title: Title with $pecial & {characters}\n    accessed: 2024-01-15\n---\nBody",
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
            "--mode",
            "bibliography",
            "--bib-format",
            "bibtex",
        ])
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);

    assert!(output.contains("@misc{"));
    assert!(output.contains("title = {Title with \\$pecial \\& \\{characters\\}}"));
    assert!(output.contains("url = {https://example.com/article}"));
    assert!(output.contains("note = {Accessed: 2024-01-15}"));
}
