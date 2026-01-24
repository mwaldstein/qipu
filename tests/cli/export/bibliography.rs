use crate::cli::support::qipu;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_export_bibliography_basic() {
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

    // Export in bibliography mode
    qipu()
        .current_dir(dir.path())
        .args(["export", "--note", "qp-aaaa", "--mode", "bibliography"])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Bibliography"))
        .stdout(predicate::str::contains(
            "[Example Article](https://example.com/article)",
        ))
        .stdout(predicate::str::contains("(accessed 2024-01-15)"))
        .stdout(predicate::str::contains("— from: Research Note"));
}

#[test]
fn test_export_bibliography_no_sources() {
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

    // Export in bibliography mode should show "no sources" message
    qipu()
        .current_dir(dir.path())
        .args(["export", "--note", "qp-aaaa", "--mode", "bibliography"])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Bibliography"))
        .stdout(predicate::str::contains(
            "*No sources found in selected notes.*",
        ));
}

#[test]
fn test_export_bibliography_multiple_notes() {
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

    // Export both notes in bibliography mode
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
        ])
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);

    // Verify all sources are present
    assert!(output.contains("# Bibliography"));
    assert!(output.contains("[Alpha Article](https://example.com/alpha)"));
    assert!(output.contains("[Beta Article](https://example.com/beta)"));
    assert!(output.contains("(accessed 2024-02-01)"));
    assert!(output.contains("https://example.com/gamma"));
    assert!(output.contains("— from: Note A"));
    assert!(output.contains("— from: Note B"));
}

#[test]
fn test_export_bibliography_deterministic_ordering() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note with multiple sources in non-alphabetical order
    let note_path = dir.path().join(".qipu/notes/qp-aaaa-ordered.md");
    fs::write(
        &note_path,
        "---\nid: qp-aaaa\ntitle: Ordered Sources\nsources:\n  - url: https://zzz.com/last\n    title: Last Source\n  - url: https://aaa.com/first\n    title: First Source\n  - url: https://mmm.com/middle\n    title: Middle Source\n---\nBody",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let result = qipu()
        .current_dir(dir.path())
        .args(["export", "--note", "qp-aaaa", "--mode", "bibliography"])
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    let lines: Vec<&str> = output.lines().collect();

    // Find the positions of each source URL in the output
    let first_pos = lines
        .iter()
        .position(|l| l.contains("https://aaa.com/first"))
        .expect("First source not found");
    let middle_pos = lines
        .iter()
        .position(|l| l.contains("https://mmm.com/middle"))
        .expect("Middle source not found");
    let last_pos = lines
        .iter()
        .position(|l| l.contains("https://zzz.com/last"))
        .expect("Last source not found");

    // Verify they appear in alphabetical order by URL
    assert!(
        first_pos < middle_pos && middle_pos < last_pos,
        "Sources should be sorted alphabetically by URL. Got positions: first={}, middle={}, last={}",
        first_pos,
        middle_pos,
        last_pos
    );
}

#[test]
fn test_export_bibliography_source_format_variations() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note with various source formats
    let note_path = dir.path().join(".qipu/notes/qp-aaaa-formats.md");
    fs::write(
        &note_path,
        "---\nid: qp-aaaa\ntitle: Format Test\nsources:\n  - url: https://example.com/full\n    title: Full Citation\n    accessed: 2024-03-01\n  - url: https://example.com/title-only\n    title: Title Only\n  - url: https://example.com/url-only\n---\nBody",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let result = qipu()
        .current_dir(dir.path())
        .args(["export", "--note", "qp-aaaa", "--mode", "bibliography"])
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);

    // Full citation with title and accessed date
    assert!(output.contains("[Full Citation](https://example.com/full)"));
    assert!(output.contains("(accessed 2024-03-01)"));

    // Title only (no accessed date)
    assert!(output.contains("[Title Only](https://example.com/title-only)"));
    assert!(!output.contains("title-only) (accessed"));

    // URL only (no title, shown as plain URL)
    assert!(output.contains("https://example.com/url-only"));
    assert!(!output.contains("[url-only]"));

    // All should reference the note
    let format_test_count = output.matches("— from: Format Test").count();
    assert_eq!(
        format_test_count, 3,
        "All three sources should reference Format Test"
    );
}

#[test]
fn test_export_bibliography_with_tag_selection() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create notes with the same tag
    let note_a_path = dir.path().join(".qipu/notes/qp-aaaa-tagged.md");
    fs::write(
        &note_a_path,
        "---\nid: qp-aaaa\ntitle: Tagged A\ntags:\n  - research\nsources:\n  - url: https://example.com/a\n    title: Source A\n---\nBody A",
    )
    .unwrap();

    let note_b_path = dir.path().join(".qipu/notes/qp-bbbb-tagged.md");
    fs::write(
        &note_b_path,
        "---\nid: qp-bbbb\ntitle: Tagged B\ntags:\n  - research\nsources:\n  - url: https://example.com/b\n    title: Source B\n---\nBody B",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Export by tag in bibliography mode
    qipu()
        .current_dir(dir.path())
        .args(["export", "--tag", "research", "--mode", "bibliography"])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Bibliography"))
        .stdout(predicate::str::contains(
            "[Source A](https://example.com/a)",
        ))
        .stdout(predicate::str::contains(
            "[Source B](https://example.com/b)",
        ))
        .stdout(predicate::str::contains("— from: Tagged A"))
        .stdout(predicate::str::contains("— from: Tagged B"));
}

#[test]
fn test_export_bibliography_with_bib_alias() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note with sources
    let note_path = dir.path().join(".qipu/notes/qp-aaaa-source.md");
    fs::write(
        &note_path,
        "---\nid: qp-aaaa\ntitle: Test\nsources:\n  - url: https://example.com/test\n    title: Test Source\n---\nBody",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Test that "bib" alias works for bibliography mode
    qipu()
        .current_dir(dir.path())
        .args(["export", "--note", "qp-aaaa", "--mode", "bib"])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Bibliography"))
        .stdout(predicate::str::contains(
            "[Test Source](https://example.com/test)",
        ));
}

#[test]
fn test_export_bibliography_singular_source_field() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note with singular source field (as created by qipu capture --source)
    let note_path = dir.path().join(".qipu/notes/qp-aaaa-singular-source.md");
    fs::write(
        &note_path,
        "---\nid: qp-aaaa\ntitle: Captured Note\nsource: https://example.com/captured\n---\nBody captured from web",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Export should include the singular source field
    qipu()
        .current_dir(dir.path())
        .args(["export", "--note", "qp-aaaa", "--mode", "bibliography"])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Bibliography"))
        .stdout(predicate::str::contains("https://example.com/captured"))
        .stdout(predicate::str::contains("— from: Captured Note"));
}

#[test]
fn test_export_bibliography_both_source_fields() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note with both singular source and sources array
    let note_path = dir.path().join(".qipu/notes/qp-aaaa-both-sources.md");
    fs::write(
        &note_path,
        "---\nid: qp-aaaa\ntitle: Mixed Sources\nsource: https://example.com/singular\nsources:\n  - url: https://example.com/array1\n    title: Array Source 1\n  - url: https://example.com/array2\n    title: Array Source 2\n---\nBody",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let result = qipu()
        .current_dir(dir.path())
        .args(["export", "--note", "qp-aaaa", "--mode", "bibliography"])
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);

    // Verify all sources are present (singular + array)
    assert!(output.contains("https://example.com/singular"));
    assert!(output.contains("[Array Source 1](https://example.com/array1)"));
    assert!(output.contains("[Array Source 2](https://example.com/array2)"));

    // All should reference the same note
    let source_count = output.matches("— from: Mixed Sources").count();
    assert_eq!(
        source_count, 3,
        "All three sources (singular + 2 array) should reference Mixed Sources"
    );
}
