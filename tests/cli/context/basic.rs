use crate::support::{qipu, setup_test_dir};
use tempfile::tempdir;

#[test]
fn test_context_records_with_body_and_sources() {
    use std::fs;

    let dir = setup_test_dir();

    // Create a note with sources manually
    let note_content = r#"---
id: qp-test1
title: Research Note
type: literature
tags:
  - research
  - testing
sources:
  - url: https://example.com/article
    title: Example Article
    accessed: 2026-01-13
  - url: https://example.com/paper
    title: Another Paper
---

This is the body of the note.

It has multiple paragraphs.
"#;

    let notes_dir = dir.path().join(".qipu/notes");
    fs::create_dir_all(&notes_dir).unwrap();
    let note_path = notes_dir.join("qp-test1-research-note.md");
    fs::write(&note_path, note_content).unwrap();

    // Rebuild index to pick up the manually created note
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Test records format with body
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "records",
            "context",
            "--note",
            "qp-test1",
            "--with-body",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify header
    assert!(stdout.contains("H qipu=1 records=1 store="));

    // Verify note metadata
    assert!(stdout.contains("N qp-test1 literature \"Research Note\""));
    assert!(stdout.contains("tags=research,testing"));

    // Verify sources (D lines)
    assert!(stdout.contains("D source url=https://example.com/article"));
    assert!(stdout.contains("title=\"Example Article\""));
    assert!(stdout.contains("accessed=2026-01-13"));
    assert!(stdout.contains("from=qp-test1"));
    assert!(stdout.contains("D source url=https://example.com/paper"));
    assert!(stdout.contains("title=\"Another Paper\""));

    // Verify body is included
    assert!(stdout.contains("B qp-test1"));
    assert!(stdout.contains("This is the body of the note."));
    assert!(stdout.contains("It has multiple paragraphs."));
    assert!(stdout.contains("B-END"));
}

#[test]
fn test_context_deterministic_ordering_with_budget() {
    use std::fs;

    let dir = setup_test_dir();

    let notes_dir = dir.path().join(".qipu/notes");
    fs::create_dir_all(&notes_dir).unwrap();

    let note_templates = vec![
        (
            r#"---
id: qp-zzz
title: Oldest Note
type: permanent
value: 95
created: 2024-01-01T00:00:00Z
---

Oldest content
"#,
            "note1.md",
        ),
        (
            r#"---
id: qp-aaa
title: Middle Note
type: permanent
value: 90
created: 2024-01-02T00:00:00Z
---

Middle content
"#,
            "note2.md",
        ),
        (
            r#"---
id: qp-mmm
title: Newest Note
type: permanent
value: 85
created: 2024-01-03T00:00:00Z
---

Newest content
"#,
            "note3.md",
        ),
    ];

    for (note_content, filename) in note_templates {
        fs::write(notes_dir.join(filename), note_content).unwrap();
    }

    // Index the notes
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let mut results = Vec::new();

    for _ in 0..3 {
        let output = qipu()
            .current_dir(dir.path())
            .args([
                "context",
                "--min-value",
                "80",
                "--max-chars",
                "2000",
                "--format",
                "json",
            ])
            .output()
            .unwrap();

        assert!(output.status.success());
        results.push(String::from_utf8_lossy(&output.stdout).to_string());
    }

    assert_eq!(results[0], results[1]);
    assert_eq!(results[1], results[2]);

    let json: serde_json::Value = serde_json::from_str(&results[0]).unwrap();
    let notes = json["notes"].as_array().unwrap();

    assert!(
        !notes.is_empty(),
        "Should include at least one note with budget"
    );

    let note_ids: Vec<&str> = notes.iter().map(|n| n["id"].as_str().unwrap()).collect();

    assert_eq!(note_ids, vec!["qp-zzz", "qp-aaa", "qp-mmm"]);
}
