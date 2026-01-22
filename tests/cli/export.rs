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
