use crate::support::{run_and_get_stdout, run_assert_success, setup_test_dir};
use std::fs;

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

fn setup_multi_level_compaction() -> tempfile::TempDir {
    let dir = setup_test_dir();

    create_note_file(&dir, "qp-note1", "Note 1", "This is note 1 content.");
    create_note_file(&dir, "qp-note2", "Note 2", "This is note 2 content.");
    create_note_file(&dir, "qp-note3", "Note 3", "This is note 3 content.");
    create_note_file(
        &dir,
        "qp-digest1",
        "Digest Level 1",
        "Level 1 digest summarizing notes 1 and 2.",
    );
    create_note_file(
        &dir,
        "qp-digest2",
        "Digest Level 2",
        "Level 2 digest summarizing digest1 and note3.",
    );

    run_assert_success(&dir, &["index"]);

    run_assert_success(
        &dir,
        &[
            "compact",
            "apply",
            "qp-digest1",
            "--note",
            "qp-note1",
            "--note",
            "qp-note2",
        ],
    );

    run_assert_success(
        &dir,
        &[
            "compact",
            "apply",
            "qp-digest2",
            "--note",
            "qp-digest1",
            "--note",
            "qp-note3",
        ],
    );

    run_assert_success(&dir, &["index"]);
    dir
}

#[test]
fn test_compact_multi_level_status_compacted_notes() {
    let dir = setup_multi_level_compaction();

    let stdout = run_and_get_stdout(&dir, &["compact", "status", "qp-note1"]);
    assert!(stdout.contains("Compacted by: Digest Level 1 (qp-digest1)"));
    assert!(stdout.contains("Canonical: Digest Level 2 (qp-digest2)"));

    let stdout = run_and_get_stdout(&dir, &["compact", "status", "qp-digest1"]);
    assert!(stdout.contains("Compacted by: Digest Level 2 (qp-digest2)"));
    assert!(stdout.contains("Canonical: Digest Level 2 (qp-digest2)"));
    assert!(stdout.contains("Compacts 2 notes:"));
    assert!(stdout.contains("Note 1 (qp-note1)"));
    assert!(stdout.contains("Note 2 (qp-note2)"));
}

#[test]
fn test_compact_multi_level_status_canonical_note() {
    let dir = setup_multi_level_compaction();

    let stdout = run_and_get_stdout(&dir, &["compact", "status", "qp-digest2"]);
    assert!(stdout.contains("Compacted by: (none)"));
    assert!(stdout.contains("Canonical: (self)"));
    assert!(stdout.contains("Compacts 2 notes:"));
    assert!(stdout.contains("Digest Level 1 (qp-digest1)"));
    assert!(stdout.contains("Note 3 (qp-note3)"));
}

#[test]
fn test_compact_multi_level_status_json() {
    let dir = setup_multi_level_compaction();

    let stdout = run_and_get_stdout(&dir, &["--format", "json", "compact", "status", "qp-note1"]);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["note_id"], "qp-note1");
    assert_eq!(json["compactor"], "qp-digest1");
    assert_eq!(json["canonical"], "qp-digest2");
}

#[test]
fn test_compact_multi_level_show_digest() {
    let dir = setup_multi_level_compaction();

    let stdout = run_and_get_stdout(&dir, &["compact", "show", "qp-digest1"]);
    assert!(stdout.contains("Digest: qp-digest1"));
    assert!(stdout.contains("Direct compaction count: 2"));

    let stdout = run_and_get_stdout(&dir, &["compact", "show", "qp-digest2"]);
    assert!(stdout.contains("Digest: qp-digest2"));
    assert!(stdout.contains("Direct compaction count: 2"));
    assert!(stdout.contains("Digest Level 1 (qp-digest1)"));
    assert!(stdout.contains("Note 3 (qp-note3)"));
}

#[test]
fn test_compact_multi_level_show_depth() {
    let dir = setup_multi_level_compaction();

    let stdout = run_and_get_stdout(
        &dir,
        &["compact", "show", "qp-digest2", "--compaction-depth", "3"],
    );
    assert!(stdout.contains("Nested compaction (depth 3):"));
    assert!(stdout.contains("Digest Level 1 (qp-digest1)"));
    assert!(stdout.contains("Note 3 (qp-note3)"));
    assert!(stdout.contains("Note 1 (qp-note1)"));
    assert!(stdout.contains("Note 2 (qp-note2)"));
}

#[test]
fn test_compact_multi_level_list_resolves_compaction() {
    let dir = setup_multi_level_compaction();

    let stdout = run_and_get_stdout(&dir, &["list"]);
    assert!(stdout.contains("qp-digest2 [P] Digest Level 2"));
    assert!(!stdout.contains("qp-note1"));
    assert!(!stdout.contains("qp-note2"));
    assert!(!stdout.contains("qp-note3"));
    assert!(!stdout.contains("qp-digest1"));
}

#[test]
fn test_compact_multi_level_list_without_resolution() {
    let dir = setup_multi_level_compaction();

    let stdout = run_and_get_stdout(&dir, &["list", "--no-resolve-compaction"]);
    assert!(stdout.contains("qp-digest2 [P] Digest Level 2"));
    assert!(stdout.contains("qp-digest1 [P] Digest Level 1"));
    assert!(stdout.contains("qp-note1 [P] Note 1"));
    assert!(stdout.contains("qp-note2 [P] Note 2"));
    assert!(stdout.contains("qp-note3 [P] Note 3"));
}

#[test]
fn test_compact_multi_level_show_resolves_compaction() {
    let dir = setup_multi_level_compaction();

    let stdout = run_and_get_stdout(&dir, &["show", "qp-note1"]);
    assert!(stdout.contains("id: qp-digest2"));
    assert!(stdout.contains("title: Digest Level 2"));

    let stdout = run_and_get_stdout(&dir, &["show", "qp-note1", "--no-resolve-compaction"]);
    assert!(stdout.contains("id: qp-note1"));
    assert!(stdout.contains("title: Note 1"));
}

#[test]
fn test_compact_multi_level_show_json() {
    let dir = setup_multi_level_compaction();

    let stdout = run_and_get_stdout(
        &dir,
        &[
            "--format",
            "json",
            "compact",
            "show",
            "qp-digest2",
            "--compaction-depth",
            "3",
        ],
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["digest_id"], "qp-digest2");
    assert_eq!(json["count"], 2);
    assert!(json["compacts"].is_array());
    let compacts = json["compacts"].as_array().unwrap();
    assert_eq!(compacts.len(), 2);
    assert!(compacts.contains(&serde_json::json!("qp-digest1")));
    assert!(compacts.contains(&serde_json::json!("qp-note3")));
    assert_eq!(json["depth"], 3);
    assert!(json["tree"].is_array());
    let tree = json["tree"].as_array().unwrap();
    assert!(!tree.is_empty());
}

#[test]
fn test_compact_multi_level_show_records() {
    let dir = setup_multi_level_compaction();

    let stdout = run_and_get_stdout(
        &dir,
        &[
            "--format",
            "records",
            "compact",
            "show",
            "qp-digest2",
            "--compaction-depth",
            "3",
        ],
    );
    assert!(stdout.contains("H qipu=1 records=1 mode=compact.show"));
    assert!(stdout.contains("digest=qp-digest2"));
    assert!(stdout.contains("count=2"));
    assert!(stdout.contains("D compacted qp-digest1"));
    assert!(stdout.contains("D compacted qp-note3"));
}
