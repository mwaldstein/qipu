use crate::cli::support::qipu;
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

    // Create a note with sources
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

    // Export in bibliography mode with BibTeX format
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

    // Verify BibTeX format
    assert!(output.contains("@misc{"));
    assert!(output.contains("title = {Example Article}"));
    assert!(output.contains("url = {https://example.com/article}"));
    assert!(output.contains("note = {Accessed: 2024-01-15}"));
    assert!(output.contains("note = {From: Research Note}"));
}

#[test]
fn test_export_bibliography_csl_json_format() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note with sources
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

    // Export in bibliography mode with CSL JSON format
    let result = qipu()
        .current_dir(dir.path())
        .args([
            "export",
            "--note",
            "qp-aaaa",
            "--mode",
            "bibliography",
            "--bib-format",
            "csl-json",
        ])
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);

    // Verify CSL JSON format
    assert!(output.contains("\"type\": \"webpage\""));
    assert!(output.contains("\"URL\": \"https://example.com/article\""));
    assert!(output.contains("\"title\": \"Example Article\""));
    assert!(output.contains("\"accessed\""));
    assert!(output.contains("\"date-parts\""));
    assert!(output.contains("\"note\": \"From: Research Note\""));
}

#[test]
fn test_export_bibliography_bibtex_multiple_sources() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create multiple notes with sources
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

    // Export all notes in bibliography mode with BibTeX format
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

    // Verify multiple BibTeX entries are present (sorted by URL)
    assert!(output.contains("title = {Alpha Article}"));
    assert!(output.contains("title = {Beta Article}"));
    assert!(output.contains("url = {https://example.com/gamma}"));

    // Verify deterministic ordering (alpha < beta < gamma alphabetically)
    let alpha_pos = output.find("https://example.com/alpha").unwrap();
    let beta_pos = output.find("https://example.com/beta").unwrap();
    let gamma_pos = output.find("https://example.com/gamma").unwrap();
    assert!(alpha_pos < beta_pos);
    assert!(beta_pos < gamma_pos);
}

#[test]
fn test_export_bibliography_bibtex_empty() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note without sources
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

    // Export in bibliography mode with BibTeX format
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
fn test_export_bibliography_csl_json_empty() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note without sources
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

    // Export in bibliography mode with CSL JSON format
    qipu()
        .current_dir(dir.path())
        .args([
            "export",
            "--note",
            "qp-aaaa",
            "--mode",
            "bibliography",
            "--bib-format",
            "csl-json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("[]"));
}
