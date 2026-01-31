//! Tests for compaction suggest command with default value thresholds

use crate::support::{qipu, setup_test_dir};
use std::fs;

#[test]
fn test_compact_suggest_no_value_uses_default() {
    let dir = setup_test_dir();

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

    assert!(
        ids.contains(&"qp-low1") || ids.contains(&"qp-low2") || ids.contains(&"qp-low3"),
        "First candidate should be the low-value cluster, not the default-value cluster"
    );
}
