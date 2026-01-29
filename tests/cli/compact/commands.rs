use crate::cli::support::qipu;
use predicates::prelude::*;
use std::fs;
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

    // Reindex to update database with the modified note
    qipu()
        .current_dir(dir.path())
        // Use --rebuild to force re-indexing since file mtime may not advance
        // within the same second (incremental indexing uses second granularity).
        .args(["index", "--rebuild"])
        .assert()
        .success();

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
fn test_compact_show_truncation() {
    use std::fs;

    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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

#[test]
fn test_compact_apply_from_stdin() {
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

    let note3_content = r#"---
id: qp-note3
title: Source Note 3
type: permanent
---
This is source note 3 content."#;

    let digest_content = r#"---
id: qp-digest
title: Digest Note
type: permanent
---
This digest summarizes notes 1-3."#;

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
        dir.path().join(".qipu/notes/qp-note3-source-note-3.md"),
        note3_content,
    )
    .unwrap();
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

    // Test reading from stdin - human format
    let stdin_input = "qp-note1\nqp-note2\nqp-note3\n";
    let output = qipu()
        .current_dir(dir.path())
        .args(["compact", "apply", "qp-digest", "--from-stdin"])
        .write_stdin(stdin_input)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("Applied compaction:"));
    assert!(stdout.contains("Digest: qp-digest"));
    assert!(stdout.contains("Compacts 3 notes:"));
    assert!(stdout.contains("qp-note1"));
    assert!(stdout.contains("qp-note2"));
    assert!(stdout.contains("qp-note3"));

    // Verify compaction was applied by checking the digest note
    let output = qipu()
        .current_dir(dir.path())
        .args(["compact", "show", "qp-digest"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("Direct compaction count: 3"));

    // Test JSON format with stdin
    let dir2 = tempdir().unwrap();
    qipu()
        .current_dir(dir2.path())
        .arg("init")
        .assert()
        .success();

    // Copy the same notes to dir2
    fs::write(
        dir2.path().join(".qipu/notes/qp-note1-source-note-1.md"),
        note1_content,
    )
    .unwrap();
    fs::write(
        dir2.path().join(".qipu/notes/qp-note2-source-note-2.md"),
        note2_content,
    )
    .unwrap();
    fs::write(
        dir2.path().join(".qipu/notes/qp-digest2-digest-note-2.md"),
        r#"---
id: qp-digest2
title: Digest Note 2
type: permanent
---
Another digest."#,
    )
    .unwrap();

    qipu()
        .current_dir(dir2.path())
        .arg("index")
        .assert()
        .success();

    let stdin_input2 = "qp-note1\nqp-note2\n";
    let output = qipu()
        .current_dir(dir2.path())
        .args([
            "--format",
            "json",
            "compact",
            "apply",
            "qp-digest2",
            "--from-stdin",
        ])
        .write_stdin(stdin_input2)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["digest_id"], "qp-digest2");
    assert_eq!(json["count"], 2);
    assert!(json["compacts"].is_array());
    let compacts = json["compacts"].as_array().unwrap();
    assert_eq!(compacts.len(), 2);
    assert!(compacts.contains(&serde_json::json!("qp-note1")));
    assert!(compacts.contains(&serde_json::json!("qp-note2")));

    // Test records format with stdin
    let dir3 = tempdir().unwrap();
    qipu()
        .current_dir(dir3.path())
        .arg("init")
        .assert()
        .success();

    fs::write(
        dir3.path().join(".qipu/notes/qp-note1-source-note-1.md"),
        note1_content,
    )
    .unwrap();
    fs::write(
        dir3.path().join(".qipu/notes/qp-digest3-digest-note-3.md"),
        r#"---
id: qp-digest3
title: Digest Note 3
type: permanent
---
Third digest."#,
    )
    .unwrap();

    qipu()
        .current_dir(dir3.path())
        .arg("index")
        .assert()
        .success();

    let stdin_input3 = "qp-note1\n";
    let output = qipu()
        .current_dir(dir3.path())
        .args([
            "--format",
            "records",
            "compact",
            "apply",
            "qp-digest3",
            "--from-stdin",
        ])
        .write_stdin(stdin_input3)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("H qipu=1 records=1 mode=compact.apply digest=qp-digest3 count=1"));
    assert!(stdout.contains("D compacted qp-note1"));
}

#[test]
fn test_compact_apply_notes_file() {
    use std::fs;
    use std::io::Write;

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

    let note3_content = r#"---
id: qp-note3
title: Source Note 3
type: permanent
---
This is source note 3 content."#;

    let digest_content = r#"---
id: qp-digest
title: Digest Note
type: permanent
---
This digest summarizes notes 1-3."#;

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
        dir.path().join(".qipu/notes/qp-note3-source-note-3.md"),
        note3_content,
    )
    .unwrap();
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

    // Create a notes file with IDs (one per line)
    let notes_file = dir.path().join("notes.txt");
    let mut file = fs::File::create(&notes_file).unwrap();
    writeln!(file, "qp-note1").unwrap();
    writeln!(file, "qp-note2").unwrap();
    writeln!(file, "qp-note3").unwrap();

    // Test reading from file - human format
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "compact",
            "apply",
            "qp-digest",
            "--notes-file",
            notes_file.to_str().unwrap(),
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("Applied compaction:"));
    assert!(stdout.contains("Digest: qp-digest"));
    assert!(stdout.contains("Compacts 3 notes:"));
    assert!(stdout.contains("qp-note1"));
    assert!(stdout.contains("qp-note2"));
    assert!(stdout.contains("qp-note3"));

    // Verify compaction was applied
    let output = qipu()
        .current_dir(dir.path())
        .args(["compact", "show", "qp-digest"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("Direct compaction count: 3"));

    // Test JSON format with notes-file
    let dir2 = tempdir().unwrap();
    qipu()
        .current_dir(dir2.path())
        .arg("init")
        .assert()
        .success();

    fs::write(
        dir2.path().join(".qipu/notes/qp-note1-source-note-1.md"),
        note1_content,
    )
    .unwrap();
    fs::write(
        dir2.path().join(".qipu/notes/qp-note2-source-note-2.md"),
        note2_content,
    )
    .unwrap();
    fs::write(
        dir2.path().join(".qipu/notes/qp-digest2-digest-note-2.md"),
        r#"---
id: qp-digest2
title: Digest Note 2
type: permanent
---
Another digest."#,
    )
    .unwrap();

    qipu()
        .current_dir(dir2.path())
        .arg("index")
        .assert()
        .success();

    let notes_file2 = dir2.path().join("notes2.txt");
    let mut file2 = fs::File::create(&notes_file2).unwrap();
    writeln!(file2, "qp-note1").unwrap();
    writeln!(file2, "qp-note2").unwrap();

    let output = qipu()
        .current_dir(dir2.path())
        .args([
            "--format",
            "json",
            "compact",
            "apply",
            "qp-digest2",
            "--notes-file",
            notes_file2.to_str().unwrap(),
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["digest_id"], "qp-digest2");
    assert_eq!(json["count"], 2);
    assert!(json["compacts"].is_array());
    let compacts = json["compacts"].as_array().unwrap();
    assert_eq!(compacts.len(), 2);
    assert!(compacts.contains(&serde_json::json!("qp-note1")));
    assert!(compacts.contains(&serde_json::json!("qp-note2")));

    // Test records format with notes-file
    let dir3 = tempdir().unwrap();
    qipu()
        .current_dir(dir3.path())
        .arg("init")
        .assert()
        .success();

    fs::write(
        dir3.path().join(".qipu/notes/qp-note1-source-note-1.md"),
        note1_content,
    )
    .unwrap();
    fs::write(
        dir3.path().join(".qipu/notes/qp-digest3-digest-note-3.md"),
        r#"---
id: qp-digest3
title: Digest Note 3
type: permanent
---
Third digest."#,
    )
    .unwrap();

    qipu()
        .current_dir(dir3.path())
        .arg("index")
        .assert()
        .success();

    let notes_file3 = dir3.path().join("notes3.txt");
    let mut file3 = fs::File::create(&notes_file3).unwrap();
    writeln!(file3, "qp-note1").unwrap();

    let output = qipu()
        .current_dir(dir3.path())
        .args([
            "--format",
            "records",
            "compact",
            "apply",
            "qp-digest3",
            "--notes-file",
            notes_file3.to_str().unwrap(),
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("H qipu=1 records=1 mode=compact.apply digest=qp-digest3 count=1"));
    assert!(stdout.contains("D compacted qp-note1"));

    // Test with empty lines and whitespace in file (should be ignored)
    let dir4 = tempdir().unwrap();
    qipu()
        .current_dir(dir4.path())
        .arg("init")
        .assert()
        .success();

    fs::write(
        dir4.path().join(".qipu/notes/qp-note1-source-note-1.md"),
        note1_content,
    )
    .unwrap();
    fs::write(
        dir4.path().join(".qipu/notes/qp-note2-source-note-2.md"),
        note2_content,
    )
    .unwrap();
    fs::write(
        dir4.path().join(".qipu/notes/qp-digest4-digest-note-4.md"),
        r#"---
id: qp-digest4
title: Digest Note 4
type: permanent
---
Fourth digest."#,
    )
    .unwrap();

    qipu()
        .current_dir(dir4.path())
        .arg("index")
        .assert()
        .success();

    let notes_file4 = dir4.path().join("notes4.txt");
    let mut file4 = fs::File::create(&notes_file4).unwrap();
    writeln!(file4, "qp-note1").unwrap();
    writeln!(file4).unwrap(); // Empty line
    writeln!(file4, "  ").unwrap(); // Whitespace only
    writeln!(file4, "qp-note2").unwrap();

    let output = qipu()
        .current_dir(dir4.path())
        .args([
            "compact",
            "apply",
            "qp-digest4",
            "--notes-file",
            notes_file4.to_str().unwrap(),
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("Compacts 2 notes:"));
    assert!(stdout.contains("qp-note1"));
    assert!(stdout.contains("qp-note2"));
}

#[test]
fn test_compact_apply_mixed_sources() {
    use std::fs;
    use std::io::Write;

    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create notes
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

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Create a notes file with some IDs
    let notes_file = dir.path().join("notes.txt");
    let mut file = fs::File::create(&notes_file).unwrap();
    writeln!(file, "qp-note3").unwrap();
    writeln!(file, "qp-note4").unwrap();

    // Test combining --note flags, --notes-file, and --from-stdin
    // Should get all 5 notes from different sources, deduplicated
    let stdin_input = "qp-note1\nqp-note5\nqp-note5\n"; // note5 twice to test dedup
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "compact",
            "apply",
            "qp-digest",
            "--note",
            "qp-note1", // Will be deduplicated with stdin
            "--note",
            "qp-note2",
            "--notes-file",
            notes_file.to_str().unwrap(),
            "--from-stdin",
        ])
        .write_stdin(stdin_input)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("Compacts 5 notes:"));
    assert!(stdout.contains("qp-note1"));
    assert!(stdout.contains("qp-note2"));
    assert!(stdout.contains("qp-note3"));
    assert!(stdout.contains("qp-note4"));
    assert!(stdout.contains("qp-note5"));

    // Verify all 5 were compacted (no duplicates)
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "compact", "show", "qp-digest"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["count"], 5);
    let compacts = json["compacts"].as_array().unwrap();
    assert_eq!(compacts.len(), 5);
}

#[test]
fn test_compact_apply_no_sources_error() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create digest note
    let digest_content = r#"---
id: qp-digest
title: Digest Note
type: permanent
---
A digest."#;

    std::fs::write(
        dir.path().join(".qipu/notes/qp-digest-digest-note.md"),
        digest_content,
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Test that providing no sources results in an error
    qipu()
        .current_dir(dir.path())
        .args(["compact", "apply", "qp-digest"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "no source note IDs provided (use --note, --from-stdin, or --notes-file)",
        ));
}

#[test]
fn test_compact_suggest_prefers_low_value() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Cluster 1: Low-value notes (strong compaction candidates)
    let note1_content = r#"---
id: qp-low1
title: Low Value Note 1
type: permanent
value: 10
links:
  - id: qp-low2
    type: related
  - id: qp-low3
    type: related
---
This is low value note 1 content."#;

    let note2_content = r#"---
id: qp-low2
title: Low Value Note 2
type: permanent
value: 15
links:
  - id: qp-low1
    type: related
  - id: qp-low3
    type: related
---
This is low value note 2 content."#;

    let note3_content = r#"---
id: qp-low3
title: Low Value Note 3
type: permanent
value: 5
links:
  - id: qp-low1
    type: related
  - id: qp-low2
    type: related
---
This is low value note 3 content."#;

    // Cluster 2: High-value notes (poor compaction candidates)
    let note4_content = r#"---
id: qp-high1
title: High Value Note 1
type: permanent
value: 90
links:
  - id: qp-high2
    type: related
  - id: qp-high3
    type: related
---
This is high value note 1 content."#;

    let note5_content = r#"---
id: qp-high2
title: High Value Note 2
type: permanent
value: 85
links:
  - id: qp-high1
    type: related
  - id: qp-high3
    type: related
---
This is high value note 2 content."#;

    let note6_content = r#"---
id: qp-high3
title: High Value Note 3
type: permanent
value: 95
links:
  - id: qp-high1
    type: related
  - id: qp-high2
    type: related
---
This is high value note 3 content."#;

    fs::write(
        dir.path().join(".qipu/notes/qp-low1-low-value-note-1.md"),
        note1_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-low2-low-value-note-2.md"),
        note2_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-low3-low-value-note-3.md"),
        note3_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-high1-high-value-note-1.md"),
        note4_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-high2-high-value-note-2.md"),
        note5_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-high3-high-value-note-3.md"),
        note6_content,
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "compact", "suggest"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let candidates = json.as_array().unwrap();

    assert!(!candidates.is_empty(), "Should have at least one candidate");

    let first = &candidates[0];
    let ids: Vec<&str> = first["ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();

    // Low-value cluster should be suggested first
    assert!(
        ids.contains(&"qp-low1") || ids.contains(&"qp-low2") || ids.contains(&"qp-low3"),
        "First candidate should be low-value cluster"
    );
}

#[test]
fn test_compact_suggest_mixed_value() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Cluster with mixed values (some low, some high)
    let note1_content = r#"---
id: qp-mixed1
title: Mixed Value Note 1
type: permanent
value: 30
links:
  - id: qp-mixed2
    type: related
  - id: qp-mixed3
    type: related
---
This is mixed value note 1 content."#;

    let note2_content = r#"---
id: qp-mixed2
title: Mixed Value Note 2
type: permanent
value: 35
links:
  - id: qp-mixed1
    type: related
  - id: qp-mixed3
    type: related
---
This is mixed value note 2 content."#;

    let note3_content = r#"---
id: qp-mixed3
title: Mixed Value Note 3
type: permanent
value: 25
links:
  - id: qp-mixed1
    type: related
  - id: qp-mixed2
    type: related
---
This is mixed value note 3 content."#;

    // Another cluster with moderate values
    let note4_content = r#"---
id: qp-mod1
title: Moderate Value Note 1
type: permanent
value: 50
links:
  - id: qp-mod2
    type: related
  - id: qp-mod3
    type: related
---
This is moderate value note 1 content."#;

    let note5_content = r#"---
id: qp-mod2
title: Moderate Value Note 2
type: permanent
value: 55
links:
  - id: qp-mod1
    type: related
  - id: qp-mod3
    type: related
---
This is moderate value note 2 content."#;

    let note6_content = r#"---
id: qp-mod3
title: Moderate Value Note 3
type: permanent
value: 48
links:
  - id: qp-mod1
    type: related
  - id: qp-mod2
    type: related
---
This is moderate value note 3 content."#;

    fs::write(
        dir.path()
            .join(".qipu/notes/qp-mixed1-mixed-value-note-1.md"),
        note1_content,
    )
    .unwrap();
    fs::write(
        dir.path()
            .join(".qipu/notes/qp-mixed2-mixed-value-note-2.md"),
        note2_content,
    )
    .unwrap();
    fs::write(
        dir.path()
            .join(".qipu/notes/qp-mixed3-mixed-value-note-3.md"),
        note3_content,
    )
    .unwrap();
    fs::write(
        dir.path()
            .join(".qipu/notes/qp-mod1-moderate-value-note-1.md"),
        note4_content,
    )
    .unwrap();
    fs::write(
        dir.path()
            .join(".qipu/notes/qp-mod2-moderate-value-note-2.md"),
        note5_content,
    )
    .unwrap();
    fs::write(
        dir.path()
            .join(".qipu/notes/qp-mod3-moderate-value-note-3.md"),
        note6_content,
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "compact", "suggest"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let candidates = json.as_array().unwrap();

    assert!(!candidates.is_empty(), "Should have at least one candidate");

    let first = &candidates[0];
    let ids: Vec<&str> = first["ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();

    // Mixed-value cluster with average ~30 should be suggested over moderate (~51)
    assert!(
        ids.contains(&"qp-mixed1") || ids.contains(&"qp-mixed2") || ids.contains(&"qp-mixed3"),
        "First candidate should be lower-average-value cluster"
    );
}

#[test]
fn test_compact_suggest_no_value_uses_default() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Cluster 1: Notes without value (should default to 50)
    let note1_content = r#"---
id: qp-default1
title: Default Value Note 1
type: permanent
links:
  - id: qp-default2
    type: related
  - id: qp-default3
    type: related
---
This is default value note 1 content."#;

    let note2_content = r#"---
id: qp-default2
title: Default Value Note 2
type: permanent
links:
  - id: qp-default1
    type: related
  - id: qp-default3
    type: related
---
This is default value note 2 content."#;

    let note3_content = r#"---
id: qp-default3
title: Default Value Note 3
type: permanent
links:
  - id: qp-default1
    type: related
  - id: qp-default2
    type: related
---
This is default value note 3 content."#;

    // Cluster 2: Low-value notes
    let note4_content = r#"---
id: qp-low1
title: Low Value Note 1
type: permanent
value: 10
links:
  - id: qp-low2
    type: related
  - id: qp-low3
    type: related
---
This is low value note 1 content."#;

    let note5_content = r#"---
id: qp-low2
title: Low Value Note 2
type: permanent
value: 15
links:
  - id: qp-low1
    type: related
  - id: qp-low3
    type: related
---
This is low value note 2 content."#;

    let note6_content = r#"---
id: qp-low3
title: Low Value Note 3
type: permanent
value: 5
links:
  - id: qp-low1
    type: related
  - id: qp-low2
    type: related
---
This is low value note 3 content."#;

    fs::write(
        dir.path()
            .join(".qipu/notes/qp-default1-default-value-note-1.md"),
        note1_content,
    )
    .unwrap();
    fs::write(
        dir.path()
            .join(".qipu/notes/qp-default2-default-value-note-2.md"),
        note2_content,
    )
    .unwrap();
    fs::write(
        dir.path()
            .join(".qipu/notes/qp-default3-default-value-note-3.md"),
        note3_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-low1-low-value-note-1.md"),
        note4_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-low2-low-value-note-2.md"),
        note5_content,
    )
    .unwrap();
    fs::write(
        dir.path().join(".qipu/notes/qp-low3-low-value-note-3.md"),
        note6_content,
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "compact", "suggest"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let candidates = json.as_array().unwrap();

    assert!(!candidates.is_empty(), "Should have at least one candidate");

    let first = &candidates[0];
    let ids: Vec<&str> = first["ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();

    // Low-value cluster should be suggested first (default 50 vs average 10)
    assert!(
        ids.contains(&"qp-low1") || ids.contains(&"qp-low2") || ids.contains(&"qp-low3"),
        "First candidate should be the low-value cluster, not the default-value cluster"
    );
}
