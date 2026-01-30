use crate::cli::support::qipu;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_export_bibliography_bibtex_special_url_chars() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let note_path = dir.path().join(".qipu/notes/qp-aaaa-special-chars.md");
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
            "bibtex",
        ])
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);

    assert!(output.contains("@misc{"));
    assert!(output.contains("title = {URL with Query and Fragment}"));
    assert!(output.contains("url = {https://example.com/path?query=value&param=test#fragment}"));
    assert!(output.contains("note = {Accessed: 2024-01-15}"));
}

#[test]
fn test_export_bibliography_bibtex_non_http_url() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let note_path = dir.path().join(".qipu/notes/qp-aaaa-non-http.md");
    fs::write(
        &note_path,
        "---\nid: qp-aaaa\ntitle: Non-HTTP URL\nsources:\n  - url: ftp://ftp.example.com/file.txt\n    title: FTP Source\n    accessed: 2024-01-15\n---\nBody",
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
    assert!(output.contains("title = {FTP Source}"));
    assert!(output.contains("url = {ftp://ftp.example.com/file.txt}"));
    assert!(output.contains("note = {Accessed: 2024-01-15}"));
}

#[test]
fn test_export_bibliography_bibtex_unicode_url() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let note_path = dir.path().join(".qipu/notes/qp-aaaa-unicode.md");
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
            "bibtex",
        ])
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);

    assert!(output.contains("@misc{"));
    assert!(output.contains("title = {Article with Unicode}"));
    assert!(output.contains("url = {https://example.com/café?title=résumé}"));
    assert!(output.contains("note = {Accessed: 2024-01-15}"));
}

#[test]
fn test_export_bibliography_bibtex_url_with_auth() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let note_path = dir.path().join(".qipu/notes/qp-aaaa-url-auth.md");
    fs::write(
        &note_path,
        "---\nid: qp-aaaa\ntitle: URL with Auth\nsources:\n  - url: https://user:pass@example.com/resource\n    title: Protected Resource\n    accessed: 2024-01-15\n---\nBody",
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
    assert!(output.contains("title = {Protected Resource}"));
    assert!(output.contains("url = {https://user:pass@example.com/resource}"));
    assert!(output.contains("note = {Accessed: 2024-01-15}"));
}
