//! Tests for context command custom filtering
use crate::support::{qipu, setup_test_dir};

#[test]
fn test_context_standalone_custom_filter() {
    use std::fs;

    let dir = setup_test_dir();

    let note1 = r#"---
id: qp-note1
title: Review Note
type: permanent
tags:
  - important
custom:
  workflow_state: review
---

This note is in review.
"#;

    let note2 = r#"---
id: qp-note2
title: Approved Note
type: permanent
tags:
  - important
custom:
  workflow_state: approved
---

This note is approved.
"#;

    let note3 = r#"---
id: qp-note3
title: No Custom Metadata Note
type: literature
tags:
  - research
---

This note has no custom metadata.
"#;

    let notes_dir = dir.path().join(".qipu/notes");
    fs::create_dir_all(&notes_dir).unwrap();
    fs::write(notes_dir.join("qp-note1-review-note.md"), note1).unwrap();
    fs::write(notes_dir.join("qp-note2-approved-note.md"), note2).unwrap();
    fs::write(notes_dir.join("qp-note3-no-custom-metadata-note.md"), note3).unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--custom-filter",
            "workflow_state=review",
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
        "Should include only review note when using standalone --custom-filter workflow_state=review, got {}",
        notes.len()
    );
    assert_eq!(notes[0]["id"].as_str().unwrap(), "qp-note1");
}

#[test]
fn test_context_custom_filter_numeric_comparisons() {
    use std::fs;

    let dir = setup_test_dir();

    let note1 = r#"---
id: qp-note1
title: Note 1
type: permanent
tags:
  - test
custom:
  count: 10
  score: 85.5
---

Note 1 content.
"#;

    let note2 = r#"---
id: qp-note2
title: Note 2
type: permanent
tags:
  - test
custom:
  count: 20
  score: 75.0
---

Note 2 content.
"#;

    let note3 = r#"---
id: qp-note3
title: Note 3
type: permanent
tags:
  - test
custom:
  count: 5
  score: 90.0
---

Note 3 content.
"#;

    let notes_dir = dir.path().join(".qipu/notes");
    fs::create_dir_all(&notes_dir).unwrap();
    fs::write(notes_dir.join("qp-note1-note-1.md"), note1).unwrap();
    fs::write(notes_dir.join("qp-note2-note-2.md"), note2).unwrap();
    fs::write(notes_dir.join("qp-note3-note-3.md"), note3).unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["context", "--custom-filter", "count>10", "--format", "json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let notes = json["notes"].as_array().unwrap();

    assert_eq!(
        notes.len(),
        1,
        "Should include only note with count > 10, got {}",
        notes.len()
    );
    assert_eq!(notes[0]["id"].as_str().unwrap(), "qp-note2");

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--custom-filter",
            "score>=80",
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
        "Should include only notes with score >= 80, got {}",
        notes.len()
    );
    let note_ids: Vec<&str> = notes.iter().map(|n| n["id"].as_str().unwrap()).collect();

    assert!(note_ids.contains(&"qp-note1"));
    assert!(note_ids.contains(&"qp-note3"));

    let output = qipu()
        .current_dir(dir.path())
        .args(["context", "--custom-filter", "count<10", "--format", "json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let notes = json["notes"].as_array().unwrap();

    assert_eq!(
        notes.len(),
        1,
        "Should include only note with count < 10, got {}",
        notes.len()
    );
    assert_eq!(notes[0]["id"].as_str().unwrap(), "qp-note3");

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--custom-filter",
            "score<=80",
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
        "Should include only note with score <= 80, got {}",
        notes.len()
    );
    assert_eq!(notes[0]["id"].as_str().unwrap(), "qp-note2");
}

#[test]
fn test_context_custom_filter_multiple_filters() {
    use std::fs;

    let dir = setup_test_dir();

    let note1 = r#"---
id: qp-note1
title: Note 1
type: permanent
tags:
  - test
custom:
  priority: high
  score: 90
  category: research
---

Note 1 content.
"#;

    let note2 = r#"---
id: qp-note2
title: Note 2
type: permanent
tags:
  - test
custom:
  priority: high
  score: 75
  category: research
---

Note 2 content.
"#;

    let note3 = r#"---
id: qp-note3
title: Note 3
type: permanent
tags:
  - test
custom:
  priority: low
  score: 90
  category: research
---

Note 3 content.
"#;

    let note4 = r#"---
id: qp-note4
title: Note 4
type: permanent
tags:
  - test
custom:
  priority: high
  score: 90
  category: implementation
---

Note 4 content.
"#;

    let notes_dir = dir.path().join(".qipu/notes");
    fs::create_dir_all(&notes_dir).unwrap();
    fs::write(notes_dir.join("qp-note1-note-1.md"), note1).unwrap();
    fs::write(notes_dir.join("qp-note2-note-2.md"), note2).unwrap();
    fs::write(notes_dir.join("qp-note3-note-3.md"), note3).unwrap();
    fs::write(notes_dir.join("qp-note4-note-4.md"), note4).unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--custom-filter",
            "priority=high",
            "--custom-filter",
            "score>=85",
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
        "Should include only notes with priority=high AND score>=85, got {}",
        notes.len()
    );
    let note_ids: Vec<&str> = notes.iter().map(|n| n["id"].as_str().unwrap()).collect();

    assert!(note_ids.contains(&"qp-note1"));
    assert!(note_ids.contains(&"qp-note4"));
    assert!(!note_ids.contains(&"qp-note2"));
    assert!(!note_ids.contains(&"qp-note3"));

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--custom-filter",
            "priority=high",
            "--custom-filter",
            "score>=85",
            "--custom-filter",
            "category=research",
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
        "Should include only note with priority=high AND score>=85 AND category=research, got {}",
        notes.len()
    );
    assert_eq!(notes[0]["id"].as_str().unwrap(), "qp-note1");
}

#[test]
fn test_context_custom_filter_date_comparisons() {
    use std::fs;

    let dir = setup_test_dir();

    let note1 = r#"---
id: qp-note1
title: January Article
type: literature
custom:
  publication_date: "2024-01-15"
---

Published in January.
"#;

    let note2 = r#"---
id: qp-note2
title: June Article
type: literature
custom:
  publication_date: "2024-06-20"
---

Published in June.
"#;

    let note3 = r#"---
id: qp-note3
title: December Article
type: literature
custom:
  publication_date: "2024-12-01"
---

Published in December.
"#;

    let notes_dir = dir.path().join(".qipu/notes");
    fs::create_dir_all(&notes_dir).unwrap();
    fs::write(notes_dir.join("qp-note1-january-article.md"), note1).unwrap();
    fs::write(notes_dir.join("qp-note2-june-article.md"), note2).unwrap();
    fs::write(notes_dir.join("qp-note3-december-article.md"), note3).unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Test: publication_date >= 2024-06-01 (should match note2 and note3)
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--custom-filter",
            "publication_date>=2024-06-01",
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
        "Should include notes with publication_date >= 2024-06-01, got {}",
        notes.len()
    );
    let note_ids: Vec<&str> = notes.iter().map(|n| n["id"].as_str().unwrap()).collect();
    assert!(note_ids.contains(&"qp-note2"));
    assert!(note_ids.contains(&"qp-note3"));

    // Test: publication_date < 2024-06-01 (should match only note1)
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--custom-filter",
            "publication_date<2024-06-01",
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
        "Should include only note with publication_date < 2024-06-01, got {}",
        notes.len()
    );
    assert_eq!(notes[0]["id"].as_str().unwrap(), "qp-note1");

    // Test: date range (>= 2024-03-01 AND <= 2024-09-01) - should match only note2
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--custom-filter",
            "publication_date>=2024-03-01",
            "--custom-filter",
            "publication_date<=2024-09-01",
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
        "Should include only note with publication_date in range 2024-03-01 to 2024-09-01, got {}",
        notes.len()
    );
    assert_eq!(notes[0]["id"].as_str().unwrap(), "qp-note2");
}
