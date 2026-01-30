use crate::support::{qipu, setup_test_dir};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_export_max_hops_no_traversal() {
    let dir = setup_test_dir();

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

    // Index notes to populate the database edges
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
    let dir = setup_test_dir();

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

    // Index notes to populate the database edges
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
    let dir = setup_test_dir();

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

    // Index notes to populate the database edges
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
    let dir = setup_test_dir();

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

    // Index notes to populate the database edges
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
    let dir = setup_test_dir();

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

    // Index notes to populate the database edges
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
    let dir = setup_test_dir();

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

    // Index notes to populate the database edges
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
