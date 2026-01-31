//! Tests for compaction apply command with file input

use crate::support::{run_and_get_stdout, run_assert_success, setup_test_dir};
use std::fs;
use std::io::Write;

fn create_note_file(dir: &tempfile::TempDir, id: &str, title: &str, content: &str) {
    let filename = format!("{}-{}.md", id, title.to_lowercase().replace(' ', "-"));
    let note_content = format!(
        r#"---
id: {}
title: {}
type: permanent
---
{}"#,
        id, title, content
    );
    fs::write(
        dir.path().join(format!(".qipu/notes/{}", filename)),
        note_content,
    )
    .unwrap();
}

fn write_notes_file(path: &std::path::Path, notes: &[&str]) {
    let mut file = fs::File::create(path).unwrap();
    for note in notes {
        writeln!(file, "{}", note).unwrap();
    }
}

fn write_notes_file_with_empty_lines(path: &std::path::Path, notes: &[&str]) {
    let mut file = fs::File::create(path).unwrap();
    for note in notes {
        writeln!(file, "{}", note).unwrap();
        writeln!(file).unwrap();
        writeln!(file, "  ").unwrap();
    }
}

fn setup_compact_test(digest_id: &str, digest_title: &str, note_ids: &[&str]) -> tempfile::TempDir {
    let dir = setup_test_dir();

    for note_id in note_ids {
        create_note_file(
            &dir,
            note_id,
            &format!("Source Note {}", note_id.trim_start_matches("qp-note")),
            "This is a source note.",
        );
    }

    create_note_file(&dir, digest_id, digest_title, "This is a digest.");
    run_assert_success(&dir, &["index"]);
    dir
}

#[test]
fn test_compact_apply_notes_file_basic() {
    let dir = setup_compact_test(
        "qp-digest",
        "Digest Note",
        &["qp-note1", "qp-note2", "qp-note3"],
    );

    let notes_file = dir.path().join("notes.txt");
    write_notes_file(&notes_file, &["qp-note1", "qp-note2", "qp-note3"]);

    let stdout = run_and_get_stdout(
        &dir,
        &[
            "compact",
            "apply",
            "qp-digest",
            "--notes-file",
            notes_file.to_str().unwrap(),
        ],
    );
    assert!(stdout.contains("Applied compaction:"));
    assert!(stdout.contains("Digest: qp-digest"));
    assert!(stdout.contains("Compacts 3 notes:"));
    assert!(stdout.contains("qp-note1"));
    assert!(stdout.contains("qp-note2"));
    assert!(stdout.contains("qp-note3"));

    let stdout = run_and_get_stdout(&dir, &["compact", "show", "qp-digest"]);
    assert!(stdout.contains("Direct compaction count: 3"));
}

#[test]
fn test_compact_apply_notes_file_json() {
    let dir = setup_compact_test("qp-digest2", "Digest Note 2", &["qp-note1", "qp-note2"]);

    let notes_file = dir.path().join("notes2.txt");
    write_notes_file(&notes_file, &["qp-note1", "qp-note2"]);

    let stdout = run_and_get_stdout(
        &dir,
        &[
            "--format",
            "json",
            "compact",
            "apply",
            "qp-digest2",
            "--notes-file",
            notes_file.to_str().unwrap(),
        ],
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["digest_id"], "qp-digest2");
    assert_eq!(json["count"], 2);
    assert!(json["compacts"].is_array());
    let compacts = json["compacts"].as_array().unwrap();
    assert_eq!(compacts.len(), 2);
    assert!(compacts.contains(&serde_json::json!("qp-note1")));
    assert!(compacts.contains(&serde_json::json!("qp-note2")));
}

#[test]
fn test_compact_apply_notes_file_records() {
    let dir = setup_compact_test("qp-digest3", "Digest Note 3", &["qp-note1"]);

    let notes_file = dir.path().join("notes3.txt");
    write_notes_file(&notes_file, &["qp-note1"]);

    let stdout = run_and_get_stdout(
        &dir,
        &[
            "--format",
            "records",
            "compact",
            "apply",
            "qp-digest3",
            "--notes-file",
            notes_file.to_str().unwrap(),
        ],
    );
    assert!(stdout.contains("H qipu=1 records=1 mode=compact.apply digest=qp-digest3 count=1"));
    assert!(stdout.contains("D compacted qp-note1"));
}

#[test]
fn test_compact_apply_notes_file_with_empty_lines() {
    let dir = setup_compact_test("qp-digest4", "Digest Note 4", &["qp-note1", "qp-note2"]);

    let notes_file = dir.path().join("notes4.txt");
    write_notes_file_with_empty_lines(&notes_file, &["qp-note1", "qp-note2"]);

    let stdout = run_and_get_stdout(
        &dir,
        &[
            "compact",
            "apply",
            "qp-digest4",
            "--notes-file",
            notes_file.to_str().unwrap(),
        ],
    );
    assert!(stdout.contains("Compacts 2 notes:"));
    assert!(stdout.contains("qp-note1"));
    assert!(stdout.contains("qp-note2"));
}
