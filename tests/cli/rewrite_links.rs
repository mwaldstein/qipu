//! Tests for wiki-link rewriting feature

use tempfile::tempdir;

fn qipu() -> assert_cmd::Command {
    assert_cmd::cargo::cargo_bin_cmd!("qipu")
}

#[test]
fn test_index_rewrites_wiki_links() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    qipu().current_dir(root).arg("init").assert().success();

    qipu()
        .current_dir(root)
        .args(["capture", "--id", "qp-a1"])
        .write_stdin("# Note A\n\nSee [[qp-b2]] and [[qp-c3|Note C]]")
        .assert()
        .success();

    qipu()
        .current_dir(root)
        .args(["capture", "--id", "qp-b2"])
        .write_stdin("# Note B\n\nThis is note B")
        .assert()
        .success();

    qipu()
        .current_dir(root)
        .args(["capture", "--id", "qp-c3"])
        .write_stdin("# Note C\n\nThis is note C")
        .assert()
        .success();

    // Index without rewriting should not change wiki-links
    qipu().current_dir(root).args(["index"]).assert().success();

    let note_a_path = root.join(".qipu/notes/qp-a1-note-a.md");
    let note_a_content = std::fs::read_to_string(&note_a_path).unwrap();

    // Should still have wiki-links
    assert!(note_a_content.contains("[[qp-b2]]"));
    assert!(note_a_content.contains("[[qp-c3|Note C]]"));

    // Index with rewriting should convert wiki-links to markdown
    qipu()
        .current_dir(root)
        .args(["index", "--rewrite-wiki-links"])
        .assert()
        .success();

    let note_a_content = std::fs::read_to_string(&note_a_path).unwrap();

    // Should now have markdown links
    assert!(!note_a_content.contains("[[qp-b2]]"));
    assert!(!note_a_content.contains("[[qp-c3|Note C]]"));
    assert!(note_a_content.contains("[qp-b2](qp-b2.md)"));
    assert!(note_a_content.contains("[Note C](qp-c3.md)"));

    // Re-running should not modify anything
    qipu()
        .current_dir(root)
        .args(["index", "--rewrite-wiki-links"])
        .assert()
        .success();

    let note_a_content2 = std::fs::read_to_string(&note_a_path).unwrap();
    assert_eq!(note_a_content, note_a_content2);
}
