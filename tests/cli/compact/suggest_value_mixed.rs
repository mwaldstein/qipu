use crate::support::{qipu, setup_test_dir};
use std::fs;

#[test]
fn test_compact_suggest_mixed_value() {
    let dir = setup_test_dir();

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

    assert!(
        ids.contains(&"qp-mixed1") || ids.contains(&"qp-mixed2") || ids.contains(&"qp-mixed3"),
        "First candidate should be lower-average-value cluster"
    );
}
