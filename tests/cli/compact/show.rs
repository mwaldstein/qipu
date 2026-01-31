//! Tests for compaction show command

use crate::support::{qipu, setup_test_dir};
use predicates::prelude::*;

#[test]
fn test_compact_show() {
    use std::fs;

    let dir = setup_test_dir();

    // Create notes to be compacted
    let note1_content = r#"---
id: qp-note1
title: Source Note 1
type: permanent
---
This is source note 1 content."#;

    let note2_content = r#"---
id: qp-note2
title: Source Note 2
type: permanent
---
This is source note 2 content."#;

    let digest_content = r#"---
id: qp-digest
title: Digest Note
type: permanent
---
## Summary
This digest summarizes notes 1 and 2.

### Note 1
Content from source note 1.

### Note 2
Content from source note 2."#;

    fs::write(
        dir.path().join(".qipu/notes/qp-note1-source-note-1.md"),
        note1_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-note2-source-note-2.md"),
        note2_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-digest-digest-note.md"),
        digest_content,
    )
    .unwrap();

    // Index the notes so they exist in the database
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Apply compaction
    qipu()
        .current_dir(dir.path())
        .args([
            "compact",
            "apply",
            "qp-digest",
            "--note",
            "qp-note1",
            "--note",
            "qp-note2",
        ])
        .assert()
        .success();

    // Test compact show command
    let output = qipu()
        .current_dir(dir.path())
        .args(["compact", "show", "qp-digest"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("Digest: qp-digest"));
    assert!(stdout.contains("Direct compaction count: 2"));
    assert!(stdout.contains("Compaction:"));
    assert!(stdout.contains("Compacted notes:"));
    assert!(stdout.contains("Source Note 1"));
    assert!(stdout.contains("Source Note 2"));

    // Test with depth parameter
    let output = qipu()
        .current_dir(dir.path())
        .args(["compact", "show", "qp-digest", "--compaction-depth", "3"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("Nested compaction"));

    // Test JSON format
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "compact", "show", "qp-digest"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["digest_id"], "qp-digest");
    assert_eq!(json["count"], 2);
    assert!(json["compacts"].is_array());
    let compacts = json["compacts"].as_array().unwrap();
    assert_eq!(compacts.len(), 2);
    assert!(compacts.contains(&serde_json::json!("qp-note1")));
    assert!(compacts.contains(&serde_json::json!("qp-note2")));
    assert!(json["compaction_pct"].is_string());

    // Test Records format
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "compact", "show", "qp-digest"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("H qipu=1 records=1 mode=compact.show"));
    assert!(stdout.contains("digest=qp-digest"));
    assert!(stdout.contains("count=2"));

    // Test error for non-digest note
    qipu()
        .current_dir(dir.path())
        .args(["compact", "show", "qp-note1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("does not compact any notes"));
}

#[test]
fn test_compact_show_truncation() {
    use std::fs;

    let dir = setup_test_dir();

    // Create 5 notes to be compacted
    for i in 1..=5 {
        let note_content = format!(
            r#"---
id: qp-note{}
title: Source Note {}
type: permanent
---
This is source note {} content."#,
            i, i, i
        );
        fs::write(
            dir.path()
                .join(format!(".qipu/notes/qp-note{}-source-note-{}.md", i, i)),
            note_content,
        )
        .unwrap();
    }

    let digest_content = r#"---
id: qp-digest
title: Digest Note
type: permanent
---
This digest summarizes notes 1-5."#;

    fs::write(
        dir.path().join(".qipu/notes/qp-digest-digest-note.md"),
        digest_content,
    )
    .unwrap();

    // Index the notes
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Apply compaction for all 5 notes
    qipu()
        .current_dir(dir.path())
        .args([
            "compact",
            "apply",
            "qp-digest",
            "--note",
            "qp-note1",
            "--note",
            "qp-note2",
            "--note",
            "qp-note3",
            "--note",
            "qp-note4",
            "--note",
            "qp-note5",
        ])
        .assert()
        .success();

    // Test with --compaction-max-nodes=3 (human format)
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "compact",
            "show",
            "qp-digest",
            "--compaction-max-nodes",
            "3",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("Digest: qp-digest"));
    assert!(stdout.contains("Direct compaction count: 3"));
    assert!(stdout.contains("truncated: showing 3 of 5 notes"));

    // Test with --compaction-max-nodes=3 (JSON format)
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "json",
            "compact",
            "show",
            "qp-digest",
            "--compaction-max-nodes",
            "3",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["digest_id"], "qp-digest");
    assert_eq!(json["count"], 3);
    assert_eq!(json["compacted_ids_truncated"], true);
    let compacts = json["compacts"].as_array().unwrap();
    assert_eq!(compacts.len(), 3);

    // Test with --compaction-max-nodes=3 (Records format)
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "records",
            "compact",
            "show",
            "qp-digest",
            "--compaction-max-nodes",
            "3",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("H qipu=1 records=1 mode=compact.show"));
    assert!(stdout.contains("digest=qp-digest"));
    assert!(stdout.contains("count=3"));
    assert!(stdout.contains("D compacted_truncated max=3 total=5"));

    // Verify without limit shows all 5
    let output = qipu()
        .current_dir(dir.path())
        .args(["compact", "show", "qp-digest"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("Direct compaction count: 5"));
    assert!(!stdout.contains("truncated"));
}
