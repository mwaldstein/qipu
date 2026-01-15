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

// ============================================================================
// Compaction annotations tests (per specs/compaction.md lines 115-125)
// ============================================================================

#[test]
fn test_compaction_annotations() {
    let tmp = tempdir().unwrap();
    let store_path = tmp.path();

    // Initialize store
    qipu()
        .args(["--store", store_path.to_str().unwrap(), "init"])
        .assert()
        .success();

    // Create source notes
    let note1_output = qipu()
        .args([
            "--store",
            store_path.to_str().unwrap(),
            "create",
            "Source Note 1",
            "--tag",
            "test",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let note1_id = String::from_utf8_lossy(&note1_output)
        .lines()
        .find(|l| l.starts_with("qp-"))
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .to_string();

    let notes_dir = store_path.join("notes");
    for entry in std::fs::read_dir(&notes_dir).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(&note1_id) {
            let mut content = std::fs::read_to_string(entry.path()).unwrap();
            content.push_str("\n\nunique-token-123");
            std::fs::write(entry.path(), content).unwrap();
            break;
        }
    }

    let note2_output = qipu()
        .args([
            "--store",
            store_path.to_str().unwrap(),
            "create",
            "Source Note 2",
            "--tag",
            "test",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let note2_id = String::from_utf8_lossy(&note2_output)
        .lines()
        .find(|l| l.starts_with("qp-"))
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .to_string();

    // Create digest note
    let digest_output = qipu()
        .args([
            "--store",
            store_path.to_str().unwrap(),
            "create",
            "Digest Summary",
            "--tag",
            "summary",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let digest_id = String::from_utf8_lossy(&digest_output)
        .lines()
        .find(|l| l.starts_with("qp-"))
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .to_string();

    let note3_output = qipu()
        .args([
            "--store",
            store_path.to_str().unwrap(),
            "create",
            "Linked Note",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let note3_id = String::from_utf8_lossy(&note3_output)
        .lines()
        .find(|l| l.starts_with("qp-"))
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .to_string();

    qipu()
        .args([
            "--store",
            store_path.to_str().unwrap(),
            "link",
            "add",
            &note1_id,
            &note3_id,
            "--type",
            "related",
        ])
        .assert()
        .success();

    // Apply compaction
    qipu()
        .args([
            "--store",
            store_path.to_str().unwrap(),
            "compact",
            "apply",
            &digest_id,
            "--note",
            &note1_id,
            "--note",
            &note2_id,
        ])
        .assert()
        .success();

    // Test list command - human format
    let list_human = qipu()
        .args(["--store", store_path.to_str().unwrap(), "list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let list_human_str = String::from_utf8_lossy(&list_human);

    // Verify digest appears with annotations
    assert!(
        list_human_str.contains("compacts=2"),
        "List human output should show compacts=2"
    );
    assert!(
        list_human_str.contains("compaction="),
        "List human output should show compaction percentage"
    );

    // Verify compacted notes are hidden (resolved view)
    assert!(
        !list_human_str.contains("Source Note 1"),
        "Source notes should be hidden in resolved view"
    );
    assert!(
        !list_human_str.contains("Source Note 2"),
        "Source notes should be hidden in resolved view"
    );

    // Test list command - JSON format
    let list_json = qipu()
        .args([
            "--store",
            store_path.to_str().unwrap(),
            "list",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let list_json_str = String::from_utf8_lossy(&list_json);
    assert!(
        list_json_str.contains("\"compacts\": 2"),
        "List JSON output should show compacts field"
    );
    assert!(
        list_json_str.contains("\"compaction_pct\""),
        "List JSON output should show compaction_pct field"
    );

    // Test list command - Records format
    let list_records = qipu()
        .args([
            "--store",
            store_path.to_str().unwrap(),
            "list",
            "--format",
            "records",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let list_records_str = String::from_utf8_lossy(&list_records);
    assert!(
        list_records_str.contains("compacts=2"),
        "List records output should show compacts=2"
    );
    assert!(
        list_records_str.contains("compaction="),
        "List records output should show compaction percentage"
    );

    // Test show command - JSON format
    let show_json = qipu()
        .args([
            "--store",
            store_path.to_str().unwrap(),
            "show",
            &digest_id,
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let show_json_str = String::from_utf8_lossy(&show_json);
    assert!(
        show_json_str.contains("\"compacts\": 2"),
        "Show JSON output should show compacts field"
    );
    assert!(
        show_json_str.contains("\"compaction_pct\""),
        "Show JSON output should show compaction_pct field"
    );

    // Show compacted note should resolve to digest (with via)
    let show_compacted = qipu()
        .args([
            "--store",
            store_path.to_str().unwrap(),
            "show",
            &note1_id,
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let show_compacted_str = String::from_utf8_lossy(&show_compacted);
    assert!(
        show_compacted_str.contains(&format!("\"id\": \"{}\"", digest_id)),
        "Show should resolve compacted note to digest"
    );
    assert!(
        show_compacted_str.contains(&format!("\"via\": \"{}\"", note1_id)),
        "Show should include via for compacted note"
    );

    let show_raw = qipu()
        .args([
            "--store",
            store_path.to_str().unwrap(),
            "show",
            &note1_id,
            "--format",
            "json",
            "--no-resolve-compaction",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let show_raw_str = String::from_utf8_lossy(&show_raw);
    assert!(
        show_raw_str.contains(&format!("\"id\": \"{}\"", note1_id)),
        "Show should return raw compacted note when resolution is disabled"
    );
    assert!(
        !show_raw_str.contains("\"via\""),
        "Show should omit via when compaction is disabled"
    );

    let show_links = qipu()
        .args([
            "--store",
            store_path.to_str().unwrap(),
            "show",
            &note1_id,
            "--links",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let show_links_str = String::from_utf8_lossy(&show_links);
    assert!(
        show_links_str.contains(&format!("\"id\": \"{}\"", digest_id)),
        "Show --links should resolve to digest"
    );
    assert!(
        show_links_str.contains(&note3_id),
        "Show --links should include edges from compacted notes"
    );

    // Test context command - JSON format
    let context_json = qipu()
        .args([
            "--store",
            store_path.to_str().unwrap(),
            "context",
            "--note",
            &digest_id,
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let context_json_str = String::from_utf8_lossy(&context_json);
    assert!(
        context_json_str.contains("\"compacts\": 2"),
        "Context JSON output should show compacts field"
    );
    assert!(
        context_json_str.contains("\"compaction_pct\""),
        "Context JSON output should show compaction_pct field"
    );

    let context_query = qipu()
        .args([
            "--store",
            store_path.to_str().unwrap(),
            "context",
            "--query",
            "unique-token-123",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let context_query_str = String::from_utf8_lossy(&context_query);
    assert!(
        context_query_str.contains(&format!("\"id\": \"{}\"", digest_id)),
        "Context query should resolve to digest"
    );
    assert!(
        context_query_str.contains(&format!("\"via\": \"{}\"", note1_id)),
        "Context query should include via for compacted match"
    );

    // Test export command - human format
    let export_human = qipu()
        .args([
            "--store",
            store_path.to_str().unwrap(),
            "export",
            "--tag",
            "test",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let export_human_str = String::from_utf8_lossy(&export_human);
    assert!(
        export_human_str.contains("compacts=2"),
        "Export human output should show compacts=2"
    );
    assert!(
        export_human_str.contains("compaction="),
        "Export human output should show compaction percentage"
    );
    assert!(
        !export_human_str.contains("Source Note 1"),
        "Export should hide compacted notes in resolved view"
    );

    // Test export command - JSON format
    let export_json = qipu()
        .args([
            "--store",
            store_path.to_str().unwrap(),
            "export",
            "--tag",
            "test",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let export_json_str = String::from_utf8_lossy(&export_json);
    assert!(
        export_json_str.contains("\"compacts\": 2"),
        "Export JSON output should show compacts field"
    );
    assert!(
        export_json_str.contains("\"compaction_pct\""),
        "Export JSON output should show compaction_pct field"
    );

    // Test export command - Records format
    let export_records = qipu()
        .args([
            "--store",
            store_path.to_str().unwrap(),
            "export",
            "--tag",
            "test",
            "--format",
            "records",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let export_records_str = String::from_utf8_lossy(&export_records);
    assert!(
        export_records_str.contains("compacts=2"),
        "Export records output should show compacts=2"
    );
    assert!(
        export_records_str.contains("compaction="),
        "Export records output should show compaction percentage"
    );

    // Test search command - human format
    let search_human = qipu()
        .args(["--store", store_path.to_str().unwrap(), "search", "Digest"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let search_human_str = String::from_utf8_lossy(&search_human);
    assert!(
        search_human_str.contains("compacts=2"),
        "Search human output should show compacts=2"
    );
    assert!(
        search_human_str.contains("compaction="),
        "Search human output should show compaction percentage"
    );
}
