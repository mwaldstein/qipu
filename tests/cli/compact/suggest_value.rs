use crate::support::{qipu, setup_test_dir};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_compact_suggest_prefers_low_value() {
    let dir = setup_test_dir();

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
    let dir = setup_test_dir();

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
    let dir = setup_test_dir();

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
