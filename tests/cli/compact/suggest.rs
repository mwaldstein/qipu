use crate::support::qipu;
use tempfile::tempdir;

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
