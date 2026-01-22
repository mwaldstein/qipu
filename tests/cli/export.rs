use crate::cli::support::qipu;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_export_basic() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note with known ID
    let note_path = dir.path().join(".qipu/notes/qp-1111-test-note.md");
    fs::write(&note_path, "---\nid: qp-1111\ntitle: Test Note\n---\nBody").unwrap();

    qipu()
        .current_dir(dir.path())
        .args(["export", "--note", "qp-1111"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Test Note"))
        .stdout(predicate::str::contains("qp-1111"));
}

#[test]
fn test_export_with_attachments() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create an attachment
    let attachments_dir = dir.path().join(".qipu/attachments");
    fs::create_dir_all(&attachments_dir).unwrap();
    fs::write(attachments_dir.join("test.png"), "image data").unwrap();

    // Create a note referencing the attachment
    let note_content = "See ![diagram](../attachments/test.png)";
    let note_path = dir.path().join(".qipu/notes/qp-1234-attachment-note.md");
    fs::write(
        &note_path,
        format!(
            "---\nid: qp-1234\ntitle: Attachment Note\n---\n{}",
            note_content
        ),
    )
    .unwrap();

    // Export with attachments
    qipu()
        .current_dir(dir.path())
        .args([
            "export",
            "--note",
            "qp-1234",
            "--output",
            "export.md",
            "--with-attachments",
        ])
        .assert()
        .success();

    // Verify attachment was copied
    assert!(dir.path().join("attachments/test.png").exists());
    assert_eq!(
        fs::read_to_string(dir.path().join("attachments/test.png")).unwrap(),
        "image data"
    );

    // Verify exported content has rewritten links (../attachments/ -> ./attachments/)
    let export_content = fs::read_to_string(dir.path().join("export.md")).unwrap();
    assert!(
        export_content.contains("./attachments/test.png"),
        "exported content should have rewritten attachment links to ./attachments/"
    );
    assert!(
        !export_content.contains("../attachments/test.png"),
        "exported content should not contain original ../attachments/ links"
    );
}

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
            "## Note: Note B (qp-bbbb)\n\n**Type:** fleeting\n\n**Path:",
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
fn test_export_tag_selection_deterministic_ordering() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create notes with specific timestamps and IDs to test ordering
    // Note C: oldest created_at
    let note_c_path = dir.path().join(".qipu/notes/qp-cccc-note-c.md");
    fs::write(
        &note_c_path,
        "---\nid: qp-cccc\ntitle: Note C\ncreated: 2020-01-01T00:00:00Z\ntags:\n  - test-tag\n---\nBody C",
    )
    .unwrap();

    // Note A: newest created_at
    let note_a_path = dir.path().join(".qipu/notes/qp-aaaa-note-a.md");
    fs::write(
        &note_a_path,
        "---\nid: qp-aaaa\ntitle: Note A\ncreated: 2022-01-01T00:00:00Z\ntags:\n  - test-tag\n---\nBody A",
    )
    .unwrap();

    // Note B: middle created_at
    let note_b_path = dir.path().join(".qipu/notes/qp-bbbb-note-b.md");
    fs::write(
        &note_b_path,
        "---\nid: qp-bbbb\ntitle: Note B\ncreated: 2021-01-01T00:00:00Z\ntags:\n  - test-tag\n---\nBody B",
    )
    .unwrap();

    // Index the notes
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Export by tag should sort by (created_at, id)
    // Expected order: Note C (2020), Note B (2021), Note A (2022)
    qipu()
        .current_dir(dir.path())
        .args(["export", "--tag", "test-tag"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Note: Note C (qp-cccc)"))
        .stdout(predicate::str::contains("Body C\n\n---\n\n## Note: Note B"))
        .stdout(predicate::str::contains("Body B\n\n---\n\n## Note: Note A"));
}

#[test]
fn test_export_tag_selection_with_same_created_at() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create notes with same created_at to test ID-based tiebreaking
    let note_c_path = dir.path().join(".qipu/notes/qp-cccc-note-c.md");
    fs::write(
        &note_c_path,
        "---\nid: qp-cccc\ntitle: Note C\ncreated: 2021-01-01T00:00:00Z\ntags:\n  - same-time\n---\nBody C",
    )
    .unwrap();

    let note_a_path = dir.path().join(".qipu/notes/qp-aaaa-note-a.md");
    fs::write(
        &note_a_path,
        "---\nid: qp-aaaa\ntitle: Note A\ncreated: 2021-01-01T00:00:00Z\ntags:\n  - same-time\n---\nBody A",
    )
    .unwrap();

    let note_b_path = dir.path().join(".qipu/notes/qp-bbbb-note-b.md");
    fs::write(
        &note_b_path,
        "---\nid: qp-bbbb\ntitle: Note B\ncreated: 2021-01-01T00:00:00Z\ntags:\n  - same-time\n---\nBody B",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // With same created_at, should sort by ID: qp-aaaa, qp-bbbb, qp-cccc
    qipu()
        .current_dir(dir.path())
        .args(["export", "--tag", "same-time"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Note: Note A (qp-aaaa)"))
        .stdout(predicate::str::contains("Body A\n\n---\n\n## Note: Note B"))
        .stdout(predicate::str::contains("Body B\n\n---\n\n## Note: Note C"));
}

#[test]
fn test_export_query_selection_deterministic_ordering() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create notes with specific timestamps containing a search term
    let note_c_path = dir.path().join(".qipu/notes/qp-cccc-note-c.md");
    fs::write(
        &note_c_path,
        "---\nid: qp-cccc\ntitle: Search Term C\ncreated: 2020-03-01T00:00:00Z\n---\nThis contains searchable content",
    )
    .unwrap();

    let note_a_path = dir.path().join(".qipu/notes/qp-aaaa-note-a.md");
    fs::write(
        &note_a_path,
        "---\nid: qp-aaaa\ntitle: Search Term A\ncreated: 2022-03-01T00:00:00Z\n---\nThis contains searchable content",
    )
    .unwrap();

    let note_b_path = dir.path().join(".qipu/notes/qp-bbbb-note-b.md");
    fs::write(
        &note_b_path,
        "---\nid: qp-bbbb\ntitle: Search Term B\ncreated: 2021-03-01T00:00:00Z\n---\nThis contains searchable content",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Export by query should sort by (created_at, id)
    // Expected order: Note C (2020), Note B (2021), Note A (2022)
    qipu()
        .current_dir(dir.path())
        .args(["export", "--query", "searchable"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Note: Search Term C (qp-cccc)"))
        .stdout(predicate::str::contains(
            "This contains searchable content\n\n---\n\n## Note: Search Term B",
        ))
        .stdout(predicate::str::contains(
            "This contains searchable content\n\n---\n\n## Note: Search Term A",
        ));
}

#[test]
fn test_export_query_selection_with_missing_created_at() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create notes with and without created_at to test sorting behavior
    // Notes with created_at should come first, then notes without (sorted by ID)
    let note_with_date = dir.path().join(".qipu/notes/qp-cccc-with-date.md");
    fs::write(
        &note_with_date,
        "---\nid: qp-cccc\ntitle: With Date\ncreated: 2021-01-01T00:00:00Z\n---\nContent with date keyword",
    )
    .unwrap();

    let note_without_date_a = dir.path().join(".qipu/notes/qp-aaaa-no-date.md");
    fs::write(
        &note_without_date_a,
        "---\nid: qp-aaaa\ntitle: No Date A\n---\nContent with keyword",
    )
    .unwrap();

    let note_without_date_b = dir.path().join(".qipu/notes/qp-bbbb-no-date.md");
    fs::write(
        &note_without_date_b,
        "---\nid: qp-bbbb\ntitle: No Date B\n---\nContent with keyword",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Notes with created_at come first, then notes without (sorted by ID)
    // Expected order: qp-cccc (has date), qp-aaaa (no date), qp-bbbb (no date)
    qipu()
        .current_dir(dir.path())
        .args(["export", "--query", "keyword"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Note: With Date (qp-cccc)"))
        .stdout(predicate::str::contains(
            "Content with date keyword\n\n---\n\n## Note: No Date A",
        ))
        .stdout(predicate::str::contains(
            "Content with keyword\n\n---\n\n## Note: No Date B",
        ));
}

#[test]
fn test_export_moc_selection_preserves_moc_order() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create notes with created_at that would sort differently
    let note_a_path = dir.path().join(".qipu/notes/qp-aaaa-note-a.md");
    fs::write(
        &note_a_path,
        "---\nid: qp-aaaa\ntitle: Note A\ncreated: 2020-01-01T00:00:00Z\n---\nBody A",
    )
    .unwrap();

    let note_b_path = dir.path().join(".qipu/notes/qp-bbbb-note-b.md");
    fs::write(
        &note_b_path,
        "---\nid: qp-bbbb\ntitle: Note B\ncreated: 2021-01-01T00:00:00Z\n---\nBody B",
    )
    .unwrap();

    let note_c_path = dir.path().join(".qipu/notes/qp-cccc-note-c.md");
    fs::write(
        &note_c_path,
        "---\nid: qp-cccc\ntitle: Note C\ncreated: 2022-01-01T00:00:00Z\n---\nBody C",
    )
    .unwrap();

    // MOC links in reverse chronological order (C -> B -> A)
    let moc_path = dir.path().join(".qipu/mocs/qp-moc1-order.md");
    fs::write(
        &moc_path,
        "---\nid: qp-moc1\ntitle: Order Test\ntype: moc\n---\n[[qp-cccc]]\n[[qp-bbbb]]\n[[qp-aaaa]]\n",
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // MOC-driven export should preserve MOC order, NOT sort by created_at
    // Expected order: C -> B -> A (as linked in MOC, not by created_at)
    qipu()
        .current_dir(dir.path())
        .args(["export", "--moc", "qp-moc1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Note: Note C (qp-cccc)"))
        .stdout(predicate::str::contains("Body C\n\n---\n\n## Note: Note B"))
        .stdout(predicate::str::contains("Body B\n\n---\n\n## Note: Note A"));
}

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
fn test_export_link_mode_preserve() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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

#[test]
fn test_export_max_hops_no_traversal() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a chain of linked notes: A -> B -> C
    let note_a = dir.path().join(".qipu/notes/qp-aaaa-note-a.md");
    fs::write(
        &note_a,
        "---\nid: qp-aaaa\ntitle: Note A\ntags:\n  - tag1\nlinks:\n  - id: qp-bbbb\n    type: related\n---\nFirst note with [[qp-bbbb]]",
    )
    .unwrap();

    let note_b = dir.path().join(".qipu/notes/qp-bbbb-note-b.md");
    fs::write(
        &note_b,
        "---\nid: qp-bbbb\ntitle: Note B\ntags:\n  - tag2\nlinks:\n  - id: qp-cccc\n    type: related\n---\nSecond note with [[qp-cccc]]",
    )
    .unwrap();

    let note_c = dir.path().join(".qipu/notes/qp-cccc-note-c.md");
    fs::write(
        &note_c,
        "---\nid: qp-cccc\ntitle: Note C\ntags:\n  - tag3\n---\nThird note",
    )
    .unwrap();

    // Index the notes to populate the database edges
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Export starting from Note A with max-hops=0 (no traversal)
    let result = qipu()
        .current_dir(dir.path())
        .args(["export", "--note", "qp-aaaa", "--max-hops", "0"])
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);

    // Should only include Note A
    assert!(output.contains("Note A"));
    assert!(!output.contains("Note B"));
    assert!(!output.contains("Note C"));
}

#[test]
fn test_export_max_hops_one_hop() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a chain of linked notes: A -> B -> C
    let note_a = dir.path().join(".qipu/notes/qp-aaaa-note-a.md");
    fs::write(
        &note_a,
        "---\nid: qp-aaaa\ntitle: Note A\ntags:\n  - tag1\nlinks:\n  - id: qp-bbbb\n    type: related\n---\nFirst note with [[qp-bbbb]]",
    )
    .unwrap();

    let note_b = dir.path().join(".qipu/notes/qp-bbbb-note-b.md");
    fs::write(
        &note_b,
        "---\nid: qp-bbbb\ntitle: Note B\ntags:\n  - tag2\nlinks:\n  - id: qp-cccc\n    type: related\n---\nSecond note with [[qp-cccc]]",
    )
    .unwrap();

    let note_c = dir.path().join(".qipu/notes/qp-cccc-note-c.md");
    fs::write(
        &note_c,
        "---\nid: qp-cccc\ntitle: Note C\ntags:\n  - tag3\n---\nThird note",
    )
    .unwrap();

    // Index the notes to populate the database edges
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Export starting from Note A with max-hops=1
    let result = qipu()
        .current_dir(dir.path())
        .args(["export", "--note", "qp-aaaa", "--max-hops", "1"])
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);

    // Should include Note A and B, but not C
    assert!(output.contains("Note A"));
    assert!(output.contains("Note B"));
    assert!(!output.contains("Note C"));
}

#[test]
fn test_export_max_hops_two_hops() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a chain of linked notes: A -> B -> C
    let note_a = dir.path().join(".qipu/notes/qp-aaaa-note-a.md");
    fs::write(
        &note_a,
        "---\nid: qp-aaaa\ntitle: Note A\ntags:\n  - tag1\nlinks:\n  - id: qp-bbbb\n    type: related\n---\nFirst note with [[qp-bbbb]]",
    )
    .unwrap();

    let note_b = dir.path().join(".qipu/notes/qp-bbbb-note-b.md");
    fs::write(
        &note_b,
        "---\nid: qp-bbbb\ntitle: Note B\ntags:\n  - tag2\nlinks:\n  - id: qp-cccc\n    type: related\n---\nSecond note with [[qp-cccc]]",
    )
    .unwrap();

    let note_c = dir.path().join(".qipu/notes/qp-cccc-note-c.md");
    fs::write(
        &note_c,
        "---\nid: qp-cccc\ntitle: Note C\ntags:\n  - tag3\n---\nThird note",
    )
    .unwrap();

    // Index the notes to populate the database edges
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Export starting from Note A with max-hops=2
    let result = qipu()
        .current_dir(dir.path())
        .args(["export", "--note", "qp-aaaa", "--max-hops", "2"])
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);

    // Should include all three notes
    assert!(output.contains("Note A"));
    assert!(output.contains("Note B"));
    assert!(output.contains("Note C"));
}

#[test]
fn test_export_max_hops_with_tag_selection() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create notes: A (with tag1) -> B -> C
    let note_a = dir.path().join(".qipu/notes/qp-aaaa-note-a.md");
    fs::write(
        &note_a,
        "---\nid: qp-aaaa\ntitle: Note A\ntags:\n  - tag1\nlinks:\n  - id: qp-bbbb\n    type: related\n---\nFirst note with [[qp-bbbb]]",
    )
    .unwrap();

    let note_b = dir.path().join(".qipu/notes/qp-bbbb-note-b.md");
    fs::write(
        &note_b,
        "---\nid: qp-bbbb\ntitle: Note B\ntags:\n  - tag2\nlinks:\n  - id: qp-cccc\n    type: related\n---\nSecond note with [[qp-cccc]]",
    )
    .unwrap();

    let note_c = dir.path().join(".qipu/notes/qp-cccc-note-c.md");
    fs::write(
        &note_c,
        "---\nid: qp-cccc\ntitle: Note C\ntags:\n  - tag3\n---\nThird note",
    )
    .unwrap();

    // Index the notes to populate the database edges
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Export notes with tag1 and expand with max-hops=1
    let result = qipu()
        .current_dir(dir.path())
        .args(["export", "--tag", "tag1", "--max-hops", "1"])
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);

    // Should include Note A (selected by tag) and B (1 hop away), but not C
    assert!(output.contains("Note A"));
    assert!(output.contains("Note B"));
    assert!(!output.contains("Note C"));
}

#[test]
fn test_export_max_hops_bidirectional_traversal() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a network: A -> B, C -> B (B is in the middle)
    let note_a = dir.path().join(".qipu/notes/qp-aaaa-note-a.md");
    fs::write(
        &note_a,
        "---\nid: qp-aaaa\ntitle: Note A\ntags:\n  - tag1\nlinks:\n  - id: qp-bbbb\n    type: related\n---\nFirst note with [[qp-bbbb]]",
    )
    .unwrap();

    let note_b = dir.path().join(".qipu/notes/qp-bbbb-note-b.md");
    fs::write(
        &note_b,
        "---\nid: qp-bbbb\ntitle: Note B\ntags:\n  - tag2\n---\nMiddle note",
    )
    .unwrap();

    let note_c = dir.path().join(".qipu/notes/qp-cccc-note-c.md");
    fs::write(
        &note_c,
        "---\nid: qp-cccc\ntitle: Note C\ntags:\n  - tag3\nlinks:\n  - id: qp-bbbb\n    type: related\n---\nThird note with [[qp-bbbb]]",
    )
    .unwrap();

    // Index the notes to populate the database edges
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Export starting from Note B with max-hops=1 (should find both A and C via backlinks)
    let result = qipu()
        .current_dir(dir.path())
        .args(["export", "--note", "qp-bbbb", "--max-hops", "1"])
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);

    // Should include all three notes (B is selected, A and C are 1 hop away via backlinks)
    assert!(output.contains("Note A"));
    assert!(output.contains("Note B"));
    assert!(output.contains("Note C"));
}

#[test]
fn test_export_max_hops_json_format() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a chain of linked notes: A -> B
    let note_a = dir.path().join(".qipu/notes/qp-aaaa-note-a.md");
    fs::write(
        &note_a,
        "---\nid: qp-aaaa\ntitle: Note A\ntags:\n  - tag1\nlinks:\n  - id: qp-bbbb\n    type: related\n---\nFirst note with [[qp-bbbb]]",
    )
    .unwrap();

    let note_b = dir.path().join(".qipu/notes/qp-bbbb-note-b.md");
    fs::write(
        &note_b,
        "---\nid: qp-bbbb\ntitle: Note B\ntags:\n  - tag2\n---\nSecond note",
    )
    .unwrap();

    // Index the notes to populate the database edges
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Export with JSON format and max-hops=1
    let result = qipu()
        .current_dir(dir.path())
        .args([
            "export",
            "--note",
            "qp-aaaa",
            "--max-hops",
            "1",
            "--format",
            "json",
        ])
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    // Verify JSON output contains both notes
    let notes = json["notes"].as_array().unwrap();
    assert_eq!(notes.len(), 2);

    // Verify both notes are present
    let note_titles: Vec<_> = notes.iter().filter_map(|n| n["title"].as_str()).collect();
    assert!(note_titles.contains(&"Note A"));
    assert!(note_titles.contains(&"Note B"));
}
