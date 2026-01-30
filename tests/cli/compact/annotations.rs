use crate::support::{
    append_to_note, apply_compaction, create_link, create_note_with_tags, rebuild_index,
    run_and_get_stdout, setup_test_dir,
};

// ============================================================================
// Compaction annotations tests (per specs/compaction.md lines 115-125)
// ============================================================================

/// Setup test store with compacted notes for annotation testing
fn setup_compaction_test() -> (tempfile::TempDir, String, String, String, String) {
    let dir = setup_test_dir();

    // Create source notes
    let note1_id = create_note_with_tags(&dir, "Source Note 1", &["test"]);
    append_to_note(&dir, &note1_id, "\n\nunique-token-123");
    rebuild_index(&dir);

    let note2_id = create_note_with_tags(&dir, "Source Note 2", &["test"]);

    // Create digest note
    let digest_id = create_note_with_tags(&dir, "Digest Summary", &["summary"]);

    // Create linked note
    let note3_id = create_note_with_tags(&dir, "Linked Note", &[]);
    create_link(&dir, &note1_id, &note3_id, "related");

    // Apply compaction
    apply_compaction(&dir, &digest_id, &[&note1_id, &note2_id]);

    (dir, note1_id, note2_id, note3_id, digest_id)
}

#[test]
fn test_compaction_annotations_list_human() {
    let (dir, _note1_id, _note2_id, _note3_id, _digest_id) = setup_compaction_test();

    let output = run_and_get_stdout(&dir, &["list"]);

    assert!(output.contains("compacts=2"), "List should show compacts=2");
    assert!(
        output.contains("compaction="),
        "List should show compaction percentage"
    );
    assert!(
        !output.contains("Source Note 1"),
        "Source notes should be hidden"
    );
    assert!(
        !output.contains("Source Note 2"),
        "Source notes should be hidden"
    );
}

#[test]
fn test_compaction_annotations_list_json() {
    let (dir, _note1_id, _note2_id, _note3_id, _digest_id) = setup_compaction_test();

    let output = run_and_get_stdout(&dir, &["list", "--format", "json"]);

    assert!(
        output.contains("\"compacts\": 2"),
        "List JSON should show compacts"
    );
    assert!(
        output.contains("\"compaction_pct\""),
        "List JSON should show compaction_pct"
    );
}

#[test]
fn test_compaction_annotations_list_records() {
    let (dir, _note1_id, _note2_id, _note3_id, _digest_id) = setup_compaction_test();

    let output = run_and_get_stdout(&dir, &["list", "--format", "records"]);

    assert!(
        output.contains("compacts=2"),
        "List records should show compacts=2"
    );
    assert!(
        output.contains("compaction="),
        "List records should show compaction percentage"
    );
}

#[test]
fn test_compaction_annotations_show_digest() {
    let (dir, _note1_id, _note2_id, _note3_id, digest_id) = setup_compaction_test();

    let output = run_and_get_stdout(&dir, &["show", &digest_id, "--format", "json"]);

    assert!(
        output.contains("\"compacts\": 2"),
        "Show should show compacts"
    );
    assert!(
        output.contains("\"compaction_pct\""),
        "Show should show compaction_pct"
    );
}

#[test]
fn test_compaction_annotations_show_compacted_resolves() {
    let (dir, note1_id, _note2_id, _note3_id, digest_id) = setup_compaction_test();

    let output = run_and_get_stdout(&dir, &["show", &note1_id, "--format", "json"]);

    assert!(
        output.contains(&format!("\"id\": \"{}\"", digest_id)),
        "Show should resolve compacted note to digest"
    );
    assert!(
        output.contains(&format!("\"via\": \"{}\"", note1_id)),
        "Show should include via for compacted note"
    );
}

#[test]
fn test_compaction_annotations_show_no_resolve() {
    let (dir, note1_id, _note2_id, _note3_id, _digest_id) = setup_compaction_test();

    let output = run_and_get_stdout(
        &dir,
        &[
            "show",
            &note1_id,
            "--format",
            "json",
            "--no-resolve-compaction",
        ],
    );

    assert!(
        output.contains(&format!("\"id\": \"{}\"", note1_id)),
        "Show should return raw compacted note"
    );
    assert!(
        !output.contains("\"via\""),
        "Show should omit via when disabled"
    );
}

#[test]
fn test_compaction_annotations_show_with_links() {
    let (dir, note1_id, _note2_id, note3_id, digest_id) = setup_compaction_test();

    let output = run_and_get_stdout(&dir, &["show", &note1_id, "--links", "--format", "json"]);

    assert!(
        output.contains(&format!("\"id\": \"{}\"", digest_id)),
        "Show --links should resolve to digest"
    );
    assert!(
        output.contains(&note3_id),
        "Show --links should include edges from compacted notes"
    );
}

#[test]
fn test_compaction_annotations_context_by_note() {
    let (dir, _note1_id, _note2_id, _note3_id, digest_id) = setup_compaction_test();

    let output = run_and_get_stdout(&dir, &["context", "--note", &digest_id, "--format", "json"]);

    assert!(
        output.contains("\"compacts\": 2"),
        "Context should show compacts"
    );
    assert!(
        output.contains("\"compaction_pct\""),
        "Context should show compaction_pct"
    );
}

#[test]
fn test_compaction_annotations_context_by_query() {
    let (dir, note1_id, _note2_id, _note3_id, digest_id) = setup_compaction_test();

    let output = run_and_get_stdout(
        &dir,
        &["context", "--query", "unique-token-123", "--format", "json"],
    );

    assert!(
        output.contains(&format!("\"id\": \"{}\"", digest_id)),
        "Context query should resolve to digest"
    );
    assert!(
        output.contains(&format!("\"via\": \"{}\"", note1_id)),
        "Context query should include via"
    );
}

#[test]
fn test_compaction_annotations_export_human() {
    let (dir, _note1_id, _note2_id, _note3_id, _digest_id) = setup_compaction_test();

    let output = run_and_get_stdout(&dir, &["export", "--tag", "test"]);

    assert!(
        output.contains("compacts=2"),
        "Export should show compacts=2"
    );
    assert!(
        output.contains("compaction="),
        "Export should show compaction percentage"
    );
    assert!(
        !output.contains("Source Note 1"),
        "Export should hide compacted notes"
    );
}

#[test]
fn test_compaction_annotations_export_json() {
    let (dir, _note1_id, _note2_id, _note3_id, _digest_id) = setup_compaction_test();

    let output = run_and_get_stdout(&dir, &["export", "--tag", "test", "--format", "json"]);

    assert!(
        output.contains("\"compacts\": 2"),
        "Export JSON should show compacts"
    );
    assert!(
        output.contains("\"compaction_pct\""),
        "Export JSON should show compaction_pct"
    );
}

#[test]
fn test_compaction_annotations_export_records() {
    let (dir, _note1_id, _note2_id, _note3_id, _digest_id) = setup_compaction_test();

    let output = run_and_get_stdout(&dir, &["export", "--tag", "test", "--format", "records"]);

    assert!(
        output.contains("compacts=2"),
        "Export records should show compacts=2"
    );
    assert!(
        output.contains("compaction="),
        "Export records should show compaction"
    );
}

#[test]
fn test_compaction_annotations_search() {
    let (dir, _note1_id, _note2_id, _note3_id, _digest_id) = setup_compaction_test();

    let output = run_and_get_stdout(&dir, &["search", "Digest"]);

    assert!(
        output.contains("compacts=2"),
        "Search should show compacts=2"
    );
    assert!(
        output.contains("compaction="),
        "Search should show compaction percentage"
    );
}
