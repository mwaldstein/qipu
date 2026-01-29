use crate::cli::support::qipu;
use std::fs;
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
