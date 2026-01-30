use crate::support::{qipu, setup_test_dir};
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_export_link_mode_preserve() {
    let dir = setup_test_dir();

    // Create notes with wiki links
    let note_a_path = dir.path().join(".qipu/notes/qp-aaaa-note-a.md");
    fs::write(
        &note_a_path,
        "---\nid: qp-aaaa\ntitle: Note A\n---\nSee [[qp-bbbb]] and [[qp-cccc|Custom Label]]",
    )
    .unwrap();

    let note_b_path = dir.path().join(".qipu/notes/qp-bbbb-note-b.md");
    fs::write(&note_b_path, "---\nid: qp-bbbb\ntitle: Note B\n---\nBody B").unwrap();

    let note_c_path = dir.path().join(".qipu/notes/qp-cccc-note-c.md");
    fs::write(&note_c_path, "---\nid: qp-cccc\ntitle: Note C\n---\nBody C").unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Export with preserve mode should keep wiki links unchanged
    qipu()
        .current_dir(dir.path())
        .args([
            "export",
            "--note",
            "qp-aaaa",
            "--note",
            "qp-bbbb",
            "--note",
            "qp-cccc",
            "--link-mode",
            "preserve",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "See [[qp-bbbb]] and [[qp-cccc|Custom Label]]",
        ))
        .stdout(predicate::str::contains("## Note: Note A (qp-aaaa)"));
}

#[test]
fn test_export_link_mode_markdown_basic() {
    let dir = setup_test_dir();

    // Create notes with wiki links
    let note_a_path = dir.path().join(".qipu/notes/qp-aaaa-note-a.md");
    fs::write(
        &note_a_path,
        "---\nid: qp-aaaa\ntitle: Note A\n---\nSee [[qp-bbbb]] for details",
    )
    .unwrap();

    let note_b_path = dir.path().join(".qipu/notes/qp-bbbb-note-b.md");
    fs::write(&note_b_path, "---\nid: qp-bbbb\ntitle: Note B\n---\nBody B").unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Export with markdown mode should convert wiki links to markdown file links
    let result = qipu()
        .current_dir(dir.path())
        .args([
            "export",
            "--note",
            "qp-aaaa",
            "--note",
            "qp-bbbb",
            "--link-mode",
            "markdown",
        ])
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);

    // Verify wiki links are converted to markdown links with file paths
    assert!(output.contains("[qp-bbbb]("));
    assert!(output.contains(".qipu/notes/qp-bbbb-note-b.md)"));
    assert!(output.contains("for details"));
    assert!(!output.contains("[[qp-bbbb]]"));
}

#[test]
fn test_export_link_mode_markdown_with_labels() {
    let dir = setup_test_dir();

    // Create notes with labeled wiki links
    let note_a_path = dir.path().join(".qipu/notes/qp-aaaa-note-a.md");
    fs::write(
        &note_a_path,
        "---\nid: qp-aaaa\ntitle: Note A\n---\nRead [[qp-bbbb|Note B]] and [[qp-cccc|the third note]]",
    )
    .unwrap();

    let note_b_path = dir.path().join(".qipu/notes/qp-bbbb-note-b.md");
    fs::write(&note_b_path, "---\nid: qp-bbbb\ntitle: Note B\n---\nBody B").unwrap();

    let note_c_path = dir.path().join(".qipu/notes/qp-cccc-note-c.md");
    fs::write(&note_c_path, "---\nid: qp-cccc\ntitle: Note C\n---\nBody C").unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Export with markdown mode should preserve custom labels
    let result = qipu()
        .current_dir(dir.path())
        .args([
            "export",
            "--note",
            "qp-aaaa",
            "--note",
            "qp-bbbb",
            "--note",
            "qp-cccc",
            "--link-mode",
            "markdown",
        ])
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);

    // Check that labels are preserved and wiki links are converted to markdown
    assert!(output.contains("[Note B]("));
    assert!(output.contains(".qipu/notes/qp-bbbb-note-b.md)"));
    assert!(output.contains("[the third note]("));
    assert!(output.contains(".qipu/notes/qp-cccc-note-c.md)"));
    assert!(!output.contains("[[qp-bbbb"));
    assert!(!output.contains("[[qp-cccc"));
}

#[test]
fn test_export_link_mode_markdown_multiple_notes() {
    let dir = setup_test_dir();

    // Create notes with cross-references
    let note_a_path = dir.path().join(".qipu/notes/qp-aaaa-note-a.md");
    fs::write(
        &note_a_path,
        "---\nid: qp-aaaa\ntitle: Note A\n---\nLinks to [[qp-bbbb]]",
    )
    .unwrap();

    let note_b_path = dir.path().join(".qipu/notes/qp-bbbb-note-b.md");
    fs::write(
        &note_b_path,
        "---\nid: qp-bbbb\ntitle: Note B\n---\nLinks to [[qp-aaaa]] and [[qp-cccc]]",
    )
    .unwrap();

    let note_c_path = dir.path().join(".qipu/notes/qp-cccc-note-c.md");
    fs::write(
        &note_c_path,
        "---\nid: qp-cccc\ntitle: Note C\n---\nLinks back to [[qp-bbbb|B]]",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Export with markdown mode should convert all wiki links
    let result = qipu()
        .current_dir(dir.path())
        .args([
            "export",
            "--note",
            "qp-aaaa",
            "--note",
            "qp-bbbb",
            "--note",
            "qp-cccc",
            "--link-mode",
            "markdown",
        ])
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);

    // Verify all links are converted to markdown format with paths
    assert!(output.contains("[qp-bbbb]("));
    assert!(output.contains(".qipu/notes/qp-bbbb-note-b.md)"));
    assert!(output.contains("[qp-aaaa]("));
    assert!(output.contains(".qipu/notes/qp-aaaa-note-a.md)"));
    assert!(output.contains("[qp-cccc]("));
    assert!(output.contains(".qipu/notes/qp-cccc-note-c.md)"));
    assert!(output.contains("[B]("));
    // No wiki links should remain
    assert!(!output.contains("[[qp-aaaa"));
    assert!(!output.contains("[[qp-bbbb"));
    assert!(!output.contains("[[qp-cccc"));
}

#[test]
fn test_export_link_mode_preserve_with_moc() {
    let dir = setup_test_dir();

    // Create notes
    let note_a_path = dir.path().join(".qipu/notes/qp-aaaa-note-a.md");
    fs::write(
        &note_a_path,
        "---\nid: qp-aaaa\ntitle: Note A\n---\nContent with [[qp-bbbb]]",
    )
    .unwrap();

    let note_b_path = dir.path().join(".qipu/notes/qp-bbbb-note-b.md");
    fs::write(&note_b_path, "---\nid: qp-bbbb\ntitle: Note B\n---\nBody B").unwrap();

    // Create MOC
    let moc_path = dir.path().join(".qipu/mocs/qp-moc1-outline.md");
    fs::write(
        &moc_path,
        "---\nid: qp-moc1\ntitle: Outline\ntype: moc\n---\n[[qp-aaaa]]\n[[qp-bbbb]]\n",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Export MOC with preserve mode
    qipu()
        .current_dir(dir.path())
        .args([
            "export",
            "--moc",
            "qp-moc1",
            "--mode",
            "outline",
            "--link-mode",
            "preserve",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Content with [[qp-bbbb]]"))
        .stdout(predicate::str::contains("## Note A (qp-aaaa)"));
}

#[test]
fn test_export_link_mode_markdown_with_moc() {
    let dir = setup_test_dir();

    // Create notes
    let note_a_path = dir.path().join(".qipu/notes/qp-aaaa-note-a.md");
    fs::write(
        &note_a_path,
        "---\nid: qp-aaaa\ntitle: Note A\n---\nSee [[qp-bbbb|Note B]]",
    )
    .unwrap();

    let note_b_path = dir.path().join(".qipu/notes/qp-bbbb-note-b.md");
    fs::write(&note_b_path, "---\nid: qp-bbbb\ntitle: Note B\n---\nBody B").unwrap();

    // Create MOC
    let moc_path = dir.path().join(".qipu/mocs/qp-moc1-outline.md");
    fs::write(
        &moc_path,
        "---\nid: qp-moc1\ntitle: Outline\ntype: moc\n---\n[[qp-aaaa]]\n[[qp-bbbb]]\n",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Export MOC with markdown mode
    let result = qipu()
        .current_dir(dir.path())
        .args([
            "export",
            "--moc",
            "qp-moc1",
            "--mode",
            "outline",
            "--link-mode",
            "markdown",
        ])
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);

    // Verify wiki links are converted to markdown with paths
    assert!(output.contains("See [Note B]("));
    assert!(output.contains(".qipu/notes/qp-bbbb-note-b.md)"));
    assert!(!output.contains("See [[qp-bbbb"));
}
