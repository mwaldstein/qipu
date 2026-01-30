use crate::support::qipu;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_export_outline_preserves_moc_order() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let note_a_path = dir.path().join(".qipu/notes/qp-aaaa-note-a.md");
    fs::write(&note_a_path, "---\nid: qp-aaaa\ntitle: Note A\n---\nBody A").unwrap();

    let note_b_path = dir.path().join(".qipu/notes/qp-bbbb-note-b.md");
    fs::write(&note_b_path, "---\nid: qp-bbbb\ntitle: Note B\n---\nBody B").unwrap();

    let moc_path = dir.path().join(".qipu/mocs/qp-moc1-outline.md");
    fs::write(
        &moc_path,
        "---\nid: qp-moc1\ntitle: Outline\ntype: moc\n---\n[[qp-bbbb]]\n[[qp-aaaa]]\n",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["export", "--moc", "qp-moc1", "--mode", "outline"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Note B (qp-bbbb)"))
        .stdout(predicate::str::contains("## Note A (qp-aaaa)"))
        .stdout(predicate::str::contains(
            "## Note B (qp-bbbb)\n\nBody B\n\n---\n\n## Note A (qp-aaaa)",
        ));
}

#[test]
fn test_export_outline_anchors() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let note_a_path = dir.path().join(".qipu/notes/qp-aaaa-note-a.md");
    fs::write(
        &note_a_path,
        "---\nid: qp-aaaa\ntitle: Note A\n---\nSee [[qp-bbbb]]",
    )
    .unwrap();

    let note_b_path = dir.path().join(".qipu/notes/qp-bbbb-note-b.md");
    fs::write(&note_b_path, "---\nid: qp-bbbb\ntitle: Note B\n---\nBody B").unwrap();

    let moc_path = dir.path().join(".qipu/mocs/qp-moc1-outline.md");
    fs::write(
        &moc_path,
        "---\nid: qp-moc1\ntitle: Outline\ntype: moc\n---\n[[qp-aaaa]]\n[[qp-bbbb]]\n",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .args([
            "export",
            "--moc",
            "qp-moc1",
            "--mode",
            "outline",
            "--link-mode",
            "anchors",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#"<a id="note-qp-aaaa"></a>"#))
        .stdout(predicate::str::contains(r#"<a id="note-qp-bbbb"></a>"#))
        .stdout(predicate::str::contains("See [qp-bbbb](#note-qp-bbbb)"));
}

#[test]
fn test_export_bundle_rewrites_links_to_anchors() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let note_a_path = dir.path().join(".qipu/notes/qp-aaaa-note-a.md");
    fs::write(&note_a_path, "---\nid: qp-aaaa\ntitle: Note A\n---\nBody A").unwrap();

    let note_b_path = dir.path().join(".qipu/notes/qp-bbbb-note-b.md");
    fs::write(
        &note_b_path,
        "---\nid: qp-bbbb\ntitle: Note B\n---\nSee [[qp-aaaa|Note A]] and [ref](qp-aaaa)",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .args([
            "export",
            "--note",
            "qp-bbbb",
            "--note",
            "qp-aaaa",
            "--link-mode",
            "anchors",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "See [Note A](#note-qp-aaaa) and [ref](#note-qp-aaaa)",
        ))
        .stdout(predicate::str::contains(r#"<a id="note-qp-aaaa"></a>"#))
        .stdout(predicate::str::contains(r#"<a id="note-qp-bbbb"></a>"#));
}

#[test]
fn test_export_bundle_preserves_moc_order() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let note_a_path = dir.path().join(".qipu/notes/qp-aaaa-note-a.md");
    fs::write(&note_a_path, "---\nid: qp-aaaa\ntitle: Note A\n---\nBody A").unwrap();

    let note_b_path = dir.path().join(".qipu/notes/qp-bbbb-note-b.md");
    fs::write(&note_b_path, "---\nid: qp-bbbb\ntitle: Note B\n---\nBody B").unwrap();

    let note_c_path = dir.path().join(".qipu/notes/qp-cccc-note-c.md");
    fs::write(&note_c_path, "---\nid: qp-cccc\ntitle: Note C\n---\nBody C").unwrap();

    let moc_path = dir.path().join(".qipu/mocs/qp-moc1-bundle.md");
    fs::write(
        &moc_path,
        "---\nid: qp-moc1\ntitle: Bundle\ntype: moc\n---\n[[qp-bbbb]]\n[[qp-cccc]]\n[[qp-aaaa]]\n",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["export", "--moc", "qp-moc1", "--mode", "bundle"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Note: Note B (qp-bbbb)"))
        .stdout(predicate::str::contains("## Note: Note C (qp-cccc)"))
        .stdout(predicate::str::contains("## Note: Note A (qp-aaaa)"))
        .stdout(predicate::str::contains(
            "## Note: Note B (qp-bbbb)\n\n**Type:** fleeting",
        ))
        .stdout(predicate::str::contains("Body B\n\n---\n\n## Note: Note C"))
        .stdout(predicate::str::contains("Body C\n\n---\n\n## Note: Note A"));
}

#[test]
fn test_export_anchor_links_point_to_existing_anchors() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let note_a_path = dir.path().join(".qipu/notes/qp-aaaa-note-a.md");
    fs::write(&note_a_path, "---\nid: qp-aaaa\ntitle: Note A\n---\nBody A").unwrap();

    let note_b_path = dir.path().join(".qipu/notes/qp-bbbb-note-b.md");
    fs::write(
        &note_b_path,
        "---\nid: qp-bbbb\ntitle: Note B\n---\nSee [[qp-aaaa]] and [[qp-cccc]]",
    )
    .unwrap();

    let note_c_path = dir.path().join(".qipu/notes/qp-cccc-note-c.md");
    fs::write(
        &note_c_path,
        "---\nid: qp-cccc\ntitle: Note C\n---\nBody C with link to [[qp-bbbb]]",
    )
    .unwrap();

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
            "anchors",
        ])
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);

    let mut anchor_ids: Vec<String> = Vec::new();
    for line in output.lines() {
        if line.contains("<a id=") && line.contains("note-") {
            let anchor_start = line.find("id=\"").unwrap();
            let id_start = anchor_start + 4;
            if let Some(id_end) = line[id_start..].find('"') {
                let anchor_id = &line[id_start..id_start + id_end];
                anchor_ids.push(anchor_id.to_string());
            }
        }
    }

    assert_eq!(
        anchor_ids.len(),
        3,
        "Should have 3 anchors for 3 notes, found: {:?}",
        anchor_ids
    );

    let mut anchor_links: Vec<String> = Vec::new();
    for line in output.lines() {
        if line.contains("](#note-") {
            let rest = &line[line.find("](#note-").unwrap()..];
            let start = rest.find("(#note-").unwrap();
            let end = rest[start..].find(')').unwrap();
            let link_target = &rest[start + 2..start + end];
            anchor_links.push(link_target.to_string());

            let remaining = &rest[start + end + 1..];
            if remaining.contains("](#note-") {
                let start2 = remaining.find("(#note-").unwrap();
                let end2 = remaining[start2..].find(')').unwrap();
                let link_target2 = &remaining[start2 + 2..start2 + end2];
                anchor_links.push(link_target2.to_string());
            }
        }
    }

    assert_eq!(
        anchor_links.len(),
        3,
        "Should have 3 rewritten links, found: {:?}",
        anchor_links
    );

    for link_target in &anchor_links {
        assert!(
            anchor_ids.contains(link_target),
            "Link points to {} but anchor doesn't exist. Anchors: {:?}",
            link_target,
            anchor_ids
        );
    }
}

#[test]
fn test_export_outline_fallback_to_bundle_without_moc() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let note_a_path = dir.path().join(".qipu/notes/qp-aaaa-note-a.md");
    fs::write(&note_a_path, "---\nid: qp-aaaa\ntitle: Note A\n---\nBody A").unwrap();

    let note_b_path = dir.path().join(".qipu/notes/qp-bbbb-note-b.md");
    fs::write(&note_b_path, "---\nid: qp-bbbb\ntitle: Note B\n---\nBody B").unwrap();

    // Export with outline mode but without --moc flag should fallback to bundle mode
    qipu()
        .current_dir(dir.path())
        .args([
            "export", "--note", "qp-aaaa", "--note", "qp-bbbb", "--mode", "outline",
        ])
        .assert()
        .success()
        // Should produce bundle-style output (with "## Note:" prefix)
        .stdout(predicate::str::contains("## Note: Note A (qp-aaaa)"))
        .stdout(predicate::str::contains("## Note: Note B (qp-bbbb)"))
        .stdout(predicate::str::contains("Body A"))
        .stdout(predicate::str::contains("Body B"));
}

#[test]
fn test_export_outline_with_typed_frontmatter_links() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let note_a_path = dir.path().join(".qipu/notes/qp-aaaa-note-a.md");
    fs::write(&note_a_path, "---\nid: qp-aaaa\ntitle: Note A\n---\nBody A").unwrap();

    let note_b_path = dir.path().join(".qipu/notes/qp-bbbb-note-b.md");
    fs::write(&note_b_path, "---\nid: qp-bbbb\ntitle: Note B\n---\nBody B").unwrap();

    let note_c_path = dir.path().join(".qipu/notes/qp-cccc-note-c.md");
    fs::write(&note_c_path, "---\nid: qp-cccc\ntitle: Note C\n---\nBody C").unwrap();

    let moc_path = dir.path().join(".qipu/mocs/qp-moc1-outline.md");
    fs::write(
        &moc_path,
        "---\nid: qp-moc1\ntitle: Outline\ntype: moc\nlinks:\n  - type: derived-from\n    id: qp-bbbb\n  - type: supports\n    id: qp-aaaa\n  - type: part-of\n    id: qp-cccc\n---\n",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["export", "--moc", "qp-moc1", "--mode", "outline"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Note B (qp-bbbb)"))
        .stdout(predicate::str::contains("## Note A (qp-aaaa)"))
        .stdout(predicate::str::contains("## Note C (qp-cccc)"))
        .stdout(predicate::str::contains(
            "## Note B (qp-bbbb)\n\nBody B\n\n---\n\n## Note A (qp-aaaa)",
        ))
        .stdout(predicate::str::contains(
            "## Note A (qp-aaaa)\n\nBody A\n\n---\n\n## Note C (qp-cccc)",
        ));
}

#[test]
fn test_export_outline_preserves_markdown_links() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let note_a_path = dir.path().join(".qipu/notes/qp-aaaa-note-a.md");
    fs::write(
        &note_a_path,
        "---\nid: qp-aaaa\ntitle: Note A\n---\nSee [custom link](.qipu/notes/qp-bbbb-note-b.md) and [[qp-cccc]]",
    )
    .unwrap();

    let note_b_path = dir.path().join(".qipu/notes/qp-bbbb-note-b.md");
    fs::write(&note_b_path, "---\nid: qp-bbbb\ntitle: Note B\n---\nBody B").unwrap();

    let note_c_path = dir.path().join(".qipu/notes/qp-cccc-note-c.md");
    fs::write(
        &note_c_path,
        "---\nid: qp-cccc\ntitle: Note C\n---\nBody C with [external](https://example.com)",
    )
    .unwrap();

    let moc_path = dir.path().join(".qipu/mocs/qp-moc1-outline.md");
    fs::write(
        &moc_path,
        "---\nid: qp-moc1\ntitle: Outline\ntype: moc\n---\n[[qp-aaaa]]\n[[qp-bbbb]]\n[[qp-cccc]]\n",
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
    assert!(output.contains("See [custom link](.qipu/notes/qp-bbbb-note-b.md)"));
    assert!(output.contains("and [qp-cccc]("));
    assert!(output.contains(".qipu/notes/qp-cccc-note-c.md)"));
    assert!(output.contains("Body C with [external](https://example.com)"));
}
