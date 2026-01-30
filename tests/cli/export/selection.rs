use crate::support::{qipu, setup_test_dir};
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_export_tag_selection_deterministic_ordering() {
    let dir = setup_test_dir();

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
    let dir = setup_test_dir();

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
    let dir = setup_test_dir();

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
    let dir = setup_test_dir();

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
    let dir = setup_test_dir();

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
