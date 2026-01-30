use crate::support::{extract_id, qipu};
use tempfile::tempdir;

// ============================================================================
// Progressive disclosure workflow tests
// ============================================================================

#[test]
fn test_get_index_then_fetch_bodies_basic() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create multiple notes
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "First note with content"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Second note with different content"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    let output3 = qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "moc", "MOC note"])
        .output()
        .unwrap();
    let id3 = extract_id(&output3);

    // Step 1: Get index view from prime (records format)
    let prime_output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "prime"])
        .output()
        .unwrap();
    let prime_stdout = String::from_utf8_lossy(&prime_output.stdout);

    // Verify prime output contains metadata and summaries but NOT full bodies
    assert!(prime_stdout.contains("H qipu=1 records=1 store="));
    assert!(prime_stdout.contains("M ")); // MOC record
    assert!(prime_stdout.contains("N ")); // Note record
    assert!(
        !prime_stdout.contains("B qp-"),
        "Prime should not include full body lines"
    );
    assert!(
        !prime_stdout.contains("B-END"),
        "Prime should not include body end markers"
    );

    // Verify we can extract note IDs from the index
    assert!(prime_stdout.contains(&id1));
    assert!(prime_stdout.contains(&id2));
    assert!(prime_stdout.contains(&id3));

    // Step 2: Fetch full body for a specific note using context
    let context_output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "context", "--note", &id1])
        .output()
        .unwrap();
    let context_stdout = String::from_utf8_lossy(&context_output.stdout);

    // Verify context output includes full body
    assert!(context_stdout.contains("H qipu=1 records=1 store="));
    assert!(context_stdout.contains(&format!("N {}", id1)));
    assert!(context_stdout.contains("B "));
    assert!(context_stdout.contains("B-END"));
    assert!(context_stdout.contains("First note with content"));
}

#[test]
fn test_get_index_then_fetch_bodies_multiple_notes() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create multiple notes
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note Alpha"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note Beta"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    let output3 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note Gamma"])
        .output()
        .unwrap();
    let id3 = extract_id(&output3);

    // Step 1: Get index view
    let prime_output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "prime"])
        .output()
        .unwrap();
    let prime_stdout = String::from_utf8_lossy(&prime_output.stdout);

    // Verify all IDs are present
    assert!(prime_stdout.contains(&id1));
    assert!(prime_stdout.contains(&id2));
    assert!(prime_stdout.contains(&id3));

    // Step 2: Fetch bodies for multiple selected notes
    let context_output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "records",
            "context",
            "--note",
            &id1,
            "--note",
            &id3,
            "--related",
            "0", // Disable related-note expansion for deterministic test
        ])
        .output()
        .unwrap();
    let context_stdout = String::from_utf8_lossy(&context_output.stdout);

    // Verify both selected notes have bodies
    assert!(context_stdout.contains("Note Alpha"));
    assert!(context_stdout.contains("Note Gamma"));

    // Count B-END markers - should have 2 (one for each selected note)
    let body_end_count = context_stdout.matches("B-END").count();
    assert_eq!(
        body_end_count, 2,
        "Should have 2 bodies for 2 selected notes"
    );

    // Verify unselected note doesn't have a body in output
    let beta_body_lines: Vec<&str> = context_stdout
        .lines()
        .filter(|line| line.starts_with(&format!("B {}", id2)))
        .collect();
    assert_eq!(
        beta_body_lines.len(),
        0,
        "Unselected note should not have body content"
    );
}

#[test]
fn test_index_excludes_bodies_context_includes_bodies() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note with substantial content
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Add more content to make the body meaningful
    qipu()
        .current_dir(dir.path())
        .args(["update", &id])
        .write_stdin(
            "# Test Note\n\nThis is the first paragraph.\n\nThis is the second paragraph with more details.\n\nThird paragraph for completeness.",
        )
        .assert()
        .success();

    // Get index from prime
    let prime_output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "prime"])
        .output()
        .unwrap();
    let prime_stdout = String::from_utf8_lossy(&prime_output.stdout);

    // Prime should NOT include body
    assert!(!prime_stdout.contains("B qp-"));
    assert!(!prime_stdout.contains("B-END"));
    assert!(!prime_stdout.contains("This is the first paragraph"));

    // Get context with full body
    let context_output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "context", "--note", &id])
        .output()
        .unwrap();
    let context_stdout = String::from_utf8_lossy(&context_output.stdout);

    // Context SHOULD include body
    assert!(context_stdout.contains("B qp-"));
    assert!(context_stdout.contains("B-END"));
    assert!(context_stdout.contains("This is the first paragraph"));
    assert!(context_stdout.contains("This is the second paragraph"));
    assert!(context_stdout.contains("Third paragraph"));
}

#[test]
fn test_progressive_disclosure_with_link_tree() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create linked notes
    let output_root = qipu()
        .current_dir(dir.path())
        .args(["create", "Root Note"])
        .output()
        .unwrap();
    let root_id = extract_id(&output_root);

    let output_a = qipu()
        .current_dir(dir.path())
        .args(["create", "Child A"])
        .output()
        .unwrap();
    let id_a = extract_id(&output_a);

    let output_b = qipu()
        .current_dir(dir.path())
        .args(["create", "Child B"])
        .output()
        .unwrap();
    let id_b = extract_id(&output_b);

    let output_c = qipu()
        .current_dir(dir.path())
        .args(["create", "Grandchild"])
        .output()
        .unwrap();
    let id_c = extract_id(&output_c);

    // Create links
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &root_id, &id_a, "--type", "supports"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &root_id, &id_b, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_c, "--type", "derived-from"])
        .assert()
        .success();

    // Rebuild index to ensure links are indexed
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Step 1: Get graph neighborhood index (small traversal)
    let tree_output = qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "tree",
            &root_id,
            "--max-hops",
            "2",
            "--format",
            "records",
        ])
        .output()
        .unwrap();
    let tree_stdout = String::from_utf8_lossy(&tree_output.stdout);

    // Verify we get graph structure but not full bodies
    assert!(tree_stdout.contains("H qipu=1 records=1"));
    assert!(tree_stdout.contains(&format!("N {}", root_id)));
    assert!(tree_stdout.contains(&format!("N {}", id_a)));
    assert!(tree_stdout.contains(&format!("N {}", id_b)));
    assert!(tree_stdout.contains(&format!("N {}", id_c)));
    assert!(tree_stdout.contains("E ")); // Edge lines
    assert!(
        !tree_stdout.contains("B qp-"),
        "Link tree should not include bodies"
    );

    // Step 2: Fetch full body for a specific node from the tree
    let context_output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "context", "--note", &id_c])
        .output()
        .unwrap();
    let context_stdout = String::from_utf8_lossy(&context_output.stdout);

    // Verify we get the full body
    assert!(context_stdout.contains(&format!("N {}", id_c)));
    assert!(context_stdout.contains("B "));
    assert!(context_stdout.contains("B-END"));
    assert!(context_stdout.contains("Grandchild"));
}

#[test]
fn test_get_index_then_fetch_with_summary_only() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note with substantial content
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Summary Test"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Get index from prime (summaries only)
    let prime_output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "prime"])
        .output()
        .unwrap();
    let prime_stdout = String::from_utf8_lossy(&prime_output.stdout);

    // Prime should NOT have body, and may or may not have summary (prime format varies)
    assert!(!prime_stdout.contains("B qp-"));
    assert!(!prime_stdout.contains("B-END"));

    // Get context with summary-only
    let context_summary = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "records",
            "context",
            "--note",
            &id,
            "--summary-only",
        ])
        .output()
        .unwrap();
    let context_summary_stdout = String::from_utf8_lossy(&context_summary.stdout);

    // Context with --summary-only should NOT include body
    assert!(context_summary_stdout.contains("S "));
    assert!(!context_summary_stdout.contains("B qp-"));
    assert!(!context_summary_stdout.contains("B-END"));

    // Get context with full body (default)
    let context_body = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "context", "--note", &id])
        .output()
        .unwrap();
    let context_body_stdout = String::from_utf8_lossy(&context_body.stdout);

    // Context default should include body
    assert!(context_body_stdout.contains("B qp-"));
    assert!(context_body_stdout.contains("B-END"));
}
