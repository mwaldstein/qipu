use crate::support::qipu;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_export_bibliography_bibtex_format() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let note_path = dir.path().join(".qipu/notes/qp-aaaa-source-note.md");
    fs::write(
        &note_path,
        "---\nid: qp-aaaa\ntitle: Research Note\nsources:\n  - url: https://example.com/article\n    title: Example Article\n    accessed: 2024-01-15\n---\nBody with citation",
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
    assert!(output.contains("title = {Example Article}"));
    assert!(output.contains("url = {https://example.com/article}"));
    assert!(output.contains("note = {Accessed: 2024-01-15}"));
    assert!(output.contains("note = {From: Research Note}"));
}

#[test]
fn test_export_bibliography_bibtex_empty() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let note_path = dir.path().join(".qipu/notes/qp-aaaa-no-sources.md");
    fs::write(
        &note_path,
        "---\nid: qp-aaaa\ntitle: Note Without Sources\n---\nBody without citations",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
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
        .success()
        .stdout(predicate::str::contains("% No sources found"));
}

#[test]
fn test_export_bibliography_bibtex_missing_title() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let note_path = dir.path().join(".qipu/notes/qp-aaaa-no-title.md");
    fs::write(
        &note_path,
        "---\nid: qp-aaaa\ntitle: Note Without Title\nsources:\n  - url: https://example.com/no-title\n    accessed: 2024-01-15\n---\nBody",
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
    assert!(output.contains("url = {https://example.com/no-title}"));
    assert!(output.contains("note = {Accessed: 2024-01-15}"));
    assert!(output.contains("note = {From: Note Without Title}"));
    assert!(!output.contains("title = "));
}

#[test]
fn test_export_bibliography_bibtex_missing_accessed() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let note_path = dir.path().join(".qipu/notes/qp-aaaa-no-accessed.md");
    fs::write(
        &note_path,
        "---\nid: qp-aaaa\ntitle: Note Without Accessed\nsources:\n  - url: https://example.com/no-accessed\n    title: Article Title\n---\nBody",
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
    assert!(output.contains("title = {Article Title}"));
    assert!(output.contains("url = {https://example.com/no-accessed}"));
    assert!(output.contains("note = {From: Note Without Accessed}"));
    assert!(!output.contains("note = {Accessed:"));
}

#[test]
fn test_export_bibliography_bibtex_url_only() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let note_path = dir.path().join(".qipu/notes/qp-aaaa-url-only.md");
    fs::write(
        &note_path,
        "---\nid: qp-aaaa\ntitle: URL Only Note\nsources:\n  - url: https://example.com/url-only\n---\nBody",
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
    assert!(output.contains("url = {https://example.com/url-only}"));
    assert!(output.contains("note = {From: URL Only Note}"));
    assert!(!output.contains("title = "));
    assert!(!output.contains("note = {Accessed:"));
}
