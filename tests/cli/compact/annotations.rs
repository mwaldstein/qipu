use crate::cli::support::qipu;
use tempfile::tempdir;

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

    // Reindex to update database with the manually edited content
    // Use --rebuild to force re-indexing since file modification may be within same second as creation
    qipu()
        .args([
            "--store",
            store_path.to_str().unwrap(),
            "index",
            "--rebuild",
        ])
        .assert()
        .success();

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
