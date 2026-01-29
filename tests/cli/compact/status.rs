use crate::cli::support::qipu;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_compact_status() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create source and digest notes
    let source_content = r#"---
id: qp-source
title: Source Note
type: permanent
---
This is a source note."#;

    let digest_content = r#"---
id: qp-digest
title: Digest Note
type: permanent
---
This digest summarizes the source note."#;

    fs::write(
        dir.path().join(".qipu/notes/qp-source-source-note.md"),
        source_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-digest-digest-note.md"),
        digest_content,
    )
    .unwrap();

    // Apply compaction
    qipu()
        .current_dir(dir.path())
        .args(["compact", "apply", "qp-digest", "--note", "qp-source"])
        .assert()
        .success();

    // Test compact status for source note (compacted by digest)
    let output = qipu()
        .current_dir(dir.path())
        .args(["compact", "status", "qp-source"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("Note: Source Note (qp-source)"));
    assert!(stdout.contains("Source Note"));
    assert!(stdout.contains("Compacted by: Digest Note"));
    assert!(stdout.contains("qp-digest"));
    assert!(stdout.contains("Compacts: (none)"));

    // Test compact status for digest note (compacts source)
    let output = qipu()
        .current_dir(dir.path())
        .args(["compact", "status", "qp-digest"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("Note: Digest Note (qp-digest)"));
    assert!(stdout.contains("Digest Note"));
    assert!(stdout.contains("Compacted by: (none)"));
    assert!(stdout.contains("Canonical: (self)"));
    assert!(stdout.contains("Compacts 1 notes:"));
    assert!(stdout.contains("Source Note (qp-source)"));

    // Test JSON format
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "compact", "status", "qp-digest"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["note_id"], "qp-digest");
    assert_eq!(json["compactor"], serde_json::Value::Null);
    assert_eq!(json["canonical"], "qp-digest");
    assert!(json["compacts"].is_array());
    let compacts = json["compacts"].as_array().unwrap();
    assert_eq!(compacts.len(), 1);
    assert_eq!(compacts[0], "qp-source");

    // Test Records format
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "compact", "status", "qp-source"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("H qipu=1 records=1 mode=compact.status"));
    assert!(stdout.contains("note=qp-source"));
    assert!(stdout.contains("compactor qp-digest"));
}
