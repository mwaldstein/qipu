use crate::support::{qipu, setup_test_dir};

#[test]
fn test_context_filter_by_min_value() {
    use std::fs;

    let dir = setup_test_dir();

    let high_value_note = r#"---
id: qp-high
title: High Value Note
type: permanent
value: 90
tags:
  - important
---

This is a high-value note.
"#;

    let low_value_note = r#"---
id: qp-low
title: Low Value Note
type: fleeting
value: 30
tags:
  - testing
---

This is a low-value note.
"#;

    let default_value_note = r#"---
id: qp-default
title: Default Value Note
type: literature
tags:
  - research
---

A note with default value (50).
"#;

    let notes_dir = dir.path().join(".qipu/notes");
    fs::create_dir_all(&notes_dir).unwrap();
    fs::write(
        notes_dir.join("qp-high-high-value-note.md"),
        high_value_note,
    )
    .unwrap();
    fs::write(notes_dir.join("qp-low-low-value-note.md"), low_value_note).unwrap();
    fs::write(
        notes_dir.join("qp-default-default-value-note.md"),
        default_value_note,
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--note",
            "qp-high",
            "--note",
            "qp-low",
            "--note",
            "qp-default",
            "--min-value",
            "80",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let notes = json["notes"].as_array().unwrap();

    assert_eq!(
        notes.len(),
        1,
        "Should include only high-value note, got {}",
        notes.len()
    );
    assert_eq!(notes[0]["id"].as_str().unwrap(), "qp-high");

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--note",
            "qp-high",
            "--note",
            "qp-low",
            "--note",
            "qp-default",
            "--min-value",
            "50",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let notes = json["notes"].as_array().unwrap();

    assert_eq!(
        notes.len(),
        2,
        "Should include high-value and default-value notes, got {}",
        notes.len()
    );

    let note_ids: Vec<&str> = notes.iter().map(|n| n["id"].as_str().unwrap()).collect();

    assert!(note_ids.contains(&"qp-high"));
    assert!(note_ids.contains(&"qp-default"));
    assert!(!note_ids.contains(&"qp-low"));
}

#[test]
fn test_context_standalone_min_value() {
    use std::fs;

    let dir = setup_test_dir();

    let high_value_note = r#"---
id: qp-high
title: High Value Note
type: permanent
value: 90
tags:
  - important
---

This is a high-value note.
"#;

    let low_value_note = r#"---
id: qp-low
title: Low Value Note
type: fleeting
value: 30
tags:
  - testing
---

This is a low-value note.
"#;

    let default_value_note = r#"---
id: qp-default
title: Default Value Note
type: literature
tags:
  - research
---

A note with default value (50).
"#;

    let notes_dir = dir.path().join(".qipu/notes");
    fs::create_dir_all(&notes_dir).unwrap();
    fs::write(
        notes_dir.join("qp-high-high-value-note.md"),
        high_value_note,
    )
    .unwrap();
    fs::write(notes_dir.join("qp-low-low-value-note.md"), low_value_note).unwrap();
    fs::write(
        notes_dir.join("qp-default-default-value-note.md"),
        default_value_note,
    )
    .unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["context", "--min-value", "80", "--format", "json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let notes = json["notes"].as_array().unwrap();

    assert_eq!(
        notes.len(),
        1,
        "Should include only high-value note when using standalone --min-value 80, got {}",
        notes.len()
    );
    assert_eq!(notes[0]["id"].as_str().unwrap(), "qp-high");

    let output = qipu()
        .current_dir(dir.path())
        .args(["context", "--min-value", "50", "--format", "json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let notes = json["notes"].as_array().unwrap();

    assert_eq!(
        notes.len(),
        2,
        "Should include high-value and default-value notes when using standalone --min-value 50, got {}",
        notes.len()
    );

    let note_ids: Vec<&str> = notes.iter().map(|n| n["id"].as_str().unwrap()).collect();

    assert!(note_ids.contains(&"qp-high"));
    assert!(note_ids.contains(&"qp-default"));
    assert!(!note_ids.contains(&"qp-low"));
}
