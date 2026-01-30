use crate::support::{qipu, setup_test_dir};
use predicates::prelude::*;
use std::fs;

#[test]
fn test_export_bibliography_csl_json_format() {
    let dir = setup_test_dir();

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
            "csl-json",
        ])
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);

    assert!(output.contains("\"type\": \"webpage\""));
    assert!(output.contains("\"URL\": \"https://example.com/article\""));
    assert!(output.contains("\"title\": \"Example Article\""));
    assert!(output.contains("\"accessed\""));
    assert!(output.contains("\"date-parts\""));
    assert!(output.contains("\"note\": \"From: Research Note\""));
}

#[test]
fn test_export_bibliography_csl_json_empty() {
    let dir = setup_test_dir();

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
            "csl-json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("[]"));
}

#[test]
fn test_export_bibliography_csl_json_missing_title() {
    let dir = setup_test_dir();

    let note_path = dir.path().join(".qipu/notes/qp-aaaa-no-title-csl.md");
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
            "csl-json",
        ])
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);

    assert!(output.contains("\"type\": \"webpage\""));
    assert!(output.contains("\"URL\": \"https://example.com/no-title\""));
    assert!(output.contains("\"accessed\""));
    assert!(output.contains("\"date-parts\""));
    assert!(output.contains("\"note\": \"From: Note Without Title\""));
    assert!(!output.contains("\"title\""));
}

#[test]
fn test_export_bibliography_csl_json_missing_accessed() {
    let dir = setup_test_dir();

    let note_path = dir.path().join(".qipu/notes/qp-aaaa-no-accessed-csl.md");
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
            "csl-json",
        ])
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);

    assert!(output.contains("\"type\": \"webpage\""));
    assert!(output.contains("\"URL\": \"https://example.com/no-accessed\""));
    assert!(output.contains("\"title\": \"Article Title\""));
    assert!(output.contains("\"note\": \"From: Note Without Accessed\""));
    assert!(!output.contains("\"accessed\""));
}

#[test]
fn test_export_bibliography_csl_json_url_only() {
    let dir = setup_test_dir();

    let note_path = dir.path().join(".qipu/notes/qp-aaaa-url-only-csl.md");
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
            "csl-json",
        ])
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);

    assert!(output.contains("\"type\": \"webpage\""));
    assert!(output.contains("\"URL\": \"https://example.com/url-only\""));
    assert!(output.contains("\"note\": \"From: URL Only Note\""));
    assert!(!output.contains("\"title\""));
    assert!(!output.contains("\"accessed\""));
}

#[test]
fn test_export_bibliography_csl_json_special_url_chars() {
    let dir = setup_test_dir();

    let note_path = dir.path().join(".qipu/notes/qp-aaaa-special-chars-csl.md");
    fs::write(
        &note_path,
        "---\nid: qp-aaaa\ntitle: Special URL Chars\nsources:\n  - url: https://example.com/path?query=value&param=test#fragment\n    title: URL with Query and Fragment\n    accessed: 2024-01-15\n---\nBody",
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
            "csl-json",
        ])
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);

    assert!(
        output.contains("\"URL\": \"https://example.com/path?query=value&param=test#fragment\"")
    );
    assert!(output.contains("\"title\": \"URL with Query and Fragment\""));
    assert!(output.contains("\"accessed\""));
}

#[test]
fn test_export_bibliography_csl_json_unicode_url() {
    let dir = setup_test_dir();

    let note_path = dir.path().join(".qipu/notes/qp-aaaa-unicode-csl.md");
    fs::write(
        &note_path,
        "---\nid: qp-aaaa\ntitle: Unicode URL\nsources:\n  - url: https://example.com/café?title=résumé\n    title: Article with Unicode\n    accessed: 2024-01-15\n---\nBody",
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
            "csl-json",
        ])
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);

    assert!(output.contains("\"URL\": \"https://example.com/café?title=résumé\""));
    assert!(output.contains("\"title\": \"Article with Unicode\""));
    assert!(output.contains("\"accessed\""));
}
