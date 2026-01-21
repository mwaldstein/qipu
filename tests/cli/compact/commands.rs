use crate::cli::support::qipu;
use predicates::prelude::*;
use tempfile::tempdir;

// ============================================================================
// Compaction command tests
// ============================================================================

#[test]
fn test_compact_report() {
    use std::fs;
    use std::thread;
    use std::time::Duration;

    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create several notes with links
    let note1_content = r#"---
id: qp-note1
title: Note 1
type: permanent
---
This is note 1 content."#;

    let note2_content = r#"---
id: qp-note2
title: Note 2
type: permanent
links:
  - id: qp-note3
    type: related
---
This is note 2 content."#;

    let note3_content = r#"---
id: qp-note3
title: Note 3
type: permanent
---
This is note 3 content."#;

    let note4_content = r#"---
id: qp-note4
title: Note 4
type: permanent
links:
  - id: qp-note1
    type: related
---
This is note 4 content."#;

    fs::write(
        dir.path().join(".qipu/notes/qp-note1-note-1.md"),
        note1_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-note2-note-2.md"),
        note2_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-note3-note-3.md"),
        note3_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-note4-note-4.md"),
        note4_content,
    )
    .unwrap();

    // Build index to populate edges
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Create a digest note
    let digest_content = r#"---
id: qp-digest
title: Digest of Notes
type: permanent
---
## Summary
This digest summarizes notes 1 and 2."#;

    fs::write(
        dir.path().join(".qipu/notes/qp-digest-digest-of-notes.md"),
        digest_content,
    )
    .unwrap();

    // Index the digest note so it exists in the database
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

    // Rebuild index after compaction
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Test human format
    let output = qipu()
        .current_dir(dir.path())
        .args(["compact", "report", "qp-digest"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("Compaction Report: qp-digest"));
    assert!(stdout.contains("Direct count: 2"));
    assert!(stdout.contains("Compaction:"));
    assert!(stdout.contains("Internal edges:"));
    assert!(stdout.contains("Boundary edges:"));
    assert!(stdout.contains("Boundary ratio:"));
    assert!(stdout.contains("Staleness:"));
    assert!(stdout.contains("Invariants:"));
    assert!(stdout.contains("VALID"));

    // Test JSON format
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "compact", "report", "qp-digest"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["digest_id"], "qp-digest");
    assert_eq!(json["compacts_direct_count"], 2);
    assert!(json["edges"]["internal"].is_number());
    assert!(json["edges"]["boundary"].is_number());
    assert!(json["edges"]["boundary_ratio"].is_string());
    assert_eq!(json["staleness"]["is_stale"], false);
    assert_eq!(json["invariants"]["valid"], true);

    // Test records format
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "compact", "report", "qp-digest"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("H qipu=1 records=1 mode=compact.report"));
    assert!(stdout.contains("digest=qp-digest"));
    assert!(stdout.contains("count=2"));
    assert!(stdout.contains("valid=true"));

    // Test staleness detection by updating a source note
    // We need to add an updated timestamp that's later than the digest
    thread::sleep(Duration::from_millis(100)); // Ensure timestamp difference

    let now = chrono::Utc::now().to_rfc3339();
    let note1_updated = format!(
        r#"---
id: qp-note1
title: Note 1
type: permanent
updated: {}
---
This is UPDATED note 1 content."#,
        now
    );

    fs::write(
        dir.path().join(".qipu/notes/qp-note1-note-1.md"),
        note1_updated,
    )
    .unwrap();

    // Report should now detect staleness
    let output = qipu()
        .current_dir(dir.path())
        .args(["compact", "report", "qp-digest"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("STALE"));

    // Test error for non-digest note
    qipu()
        .current_dir(dir.path())
        .args(["compact", "report", "qp-note4"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not compact any notes"));
}

#[test]
fn test_compact_suggest() {
    use std::fs;

    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a cluster of interconnected notes
    // Cluster 1: notes 1-3 (tightly connected)
    let note1_content = r#"---
id: qp-note1
title: Note 1
type: permanent
links:
  - id: qp-note2
    type: related
  - id: qp-note3
    type: related
---
This is note 1 content."#;

    let note2_content = r#"---
id: qp-note2
title: Note 2
type: permanent
links:
  - id: qp-note1
    type: related
  - id: qp-note3
    type: related
---
This is note 2 content."#;

    let note3_content = r#"---
id: qp-note3
title: Note 3
type: permanent
links:
  - id: qp-note1
    type: related
  - id: qp-note2
    type: related
---
This is note 3 content."#;

    // Cluster 2: notes 4-6 (tightly connected)
    let note4_content = r#"---
id: qp-note4
title: Note 4
type: permanent
links:
  - id: qp-note5
    type: related
  - id: qp-note6
    type: related
---
This is note 4 content."#;

    let note5_content = r#"---
id: qp-note5
title: Note 5
type: permanent
links:
  - id: qp-note4
    type: related
  - id: qp-note6
    type: related
---
This is note 5 content."#;

    let note6_content = r#"---
id: qp-note6
title: Note 6
type: permanent
links:
  - id: qp-note4
    type: related
  - id: qp-note5
    type: related
---
This is note 6 content."#;

    // Isolated note (should not appear in suggestions)
    let note7_content = r#"---
id: qp-note7
title: Note 7
type: permanent
---
This is an isolated note."#;

    // Write all notes
    fs::write(
        dir.path().join(".qipu/notes/qp-note1-note-1.md"),
        note1_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-note2-note-2.md"),
        note2_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-note3-note-3.md"),
        note3_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-note4-note-4.md"),
        note4_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-note5-note-5.md"),
        note5_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-note6-note-6.md"),
        note6_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-note7-note-7.md"),
        note7_content,
    )
    .unwrap();

    // Build index to populate edges
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Test human format
    let output = qipu()
        .current_dir(dir.path())
        .args(["compact", "suggest"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("Compaction Candidates"));
    assert!(stdout.contains("Candidate 1"));
    assert!(stdout.contains("score:"));
    assert!(stdout.contains("Notes:"));
    assert!(stdout.contains("Cohesion:"));
    assert!(stdout.contains("Next step:"));
    assert!(stdout.contains("qipu compact apply"));

    // Test JSON format
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "compact", "suggest"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Should have at least one candidate
    assert!(json.is_array());
    let candidates = json.as_array().unwrap();
    assert!(!candidates.is_empty());

    // Check first candidate structure
    let first = &candidates[0];
    assert!(first["ids"].is_array());
    assert!(first["node_count"].is_number());
    assert!(first["internal_edges"].is_number());
    assert!(first["boundary_edges"].is_number());
    assert!(first["cohesion"].is_string());
    assert!(first["score"].is_string());
    assert!(first["suggested_command"].is_string());

    // Test records format
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "compact", "suggest"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("H qipu=1 records=1 mode=compact.suggest"));
    assert!(stdout.contains("D candidate"));

    // Test empty store (no candidates)
    let empty_dir = tempdir().unwrap();
    qipu()
        .current_dir(empty_dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(empty_dir.path())
        .args(["compact", "suggest"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("No compaction candidates found"));
}

#[test]
fn test_compact_show() {
    use std::fs;

    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
fn test_compact_status() {
    use std::fs;

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
