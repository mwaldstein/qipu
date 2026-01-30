use assert_cmd::{cargo::cargo_bin_cmd, Command};
use predicates::prelude::*;
use tempfile::tempdir;

fn qipu() -> Command {
    cargo_bin_cmd!("qipu")
}

#[test]
fn test_load_strategy_merge_links() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test.pack.json");

    // Use unique IDs with timestamp to avoid any parallel test interference
    let unique_suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let target_id = format!("qp-{}", unique_suffix);
    let linked_id = format!("qp-{}", unique_suffix + 1);

    // 1. Initialize store 1 and create a note with links
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Target Note")
        .arg("--id")
        .arg(&target_id)
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Linked Note")
        .arg("--id")
        .arg(&linked_id)
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg(&target_id)
        .arg(&linked_id)
        .arg("--type")
        .arg("related")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Pack the notes
    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 3. Initialize store 2 and create a target note with same ID but different links
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // Create target note in store2 with same ID as in store1
    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Target Note")
        .arg("--id")
        .arg(&target_id)
        .arg("--tag")
        .arg("store2")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 4. Load with merge-links strategy
    let mut cmd = qipu();
    cmd.arg("load")
        .arg(&pack_file)
        .arg("--strategy")
        .arg("merge-links")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 5. Verify the target note now has the merged link from pack
    let output = qipu()
        .arg("show")
        .arg(&target_id)
        .arg("--links")
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();

    assert!(
        predicate::str::contains(linked_id.as_str()).eval(&String::from_utf8_lossy(&output.stdout))
    );
}

#[test]
fn test_load_strategy_merge_links_preserves_content() {
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test.pack.json");

    // Use unique IDs with timestamp to avoid any parallel test interference
    let unique_suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let target_id = format!("qp-{}", unique_suffix);
    let linked_id = format!("qp-{}", unique_suffix + 1);

    // 1. Initialize store 1 and create notes with links
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Target Note")
        .arg("--id")
        .arg(&target_id)
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Linked Note")
        .arg("--id")
        .arg(&linked_id)
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg(&target_id)
        .arg(&linked_id)
        .arg("--type")
        .arg("related")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Pack the notes
    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 3. Initialize store 2 and create a target note with DIFFERENT content
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // Create target note in store2 with same ID but different title and tags
    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Different Title")
        .arg("--id")
        .arg(&target_id)
        .arg("--tag")
        .arg("store2-tag")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 4. Load with merge-links strategy
    let mut cmd = qipu();
    cmd.arg("load")
        .arg(&pack_file)
        .arg("--strategy")
        .arg("merge-links")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 5. Verify the target note's ORIGINAL content is preserved
    let output = qipu()
        .arg("show")
        .arg(&target_id)
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();

    let output_str = String::from_utf8_lossy(&output.stdout);

    // Title should be from store2 (original), not from pack
    assert!(predicate::str::contains("Different Title").eval(&output_str));
    assert!(!predicate::str::contains("Target Note").eval(&output_str));

    // Tag from store2 should be preserved
    assert!(predicate::str::contains("store2-tag").eval(&output_str));

    // Link from pack should be added
    assert!(predicate::str::contains(linked_id.as_str()).eval(&output_str));
}

#[test]
fn test_merge_links_only_merges_to_newly_loaded_notes() {
    // This test verifies that merge-links strategy only merges links
    // when the TARGET note was newly loaded, not when it already existed
    let dir1 = tempdir().unwrap();
    let store1_path = dir1.path();
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();
    let pack_file = dir1.path().join("test.pack.json");

    // Use unique IDs to avoid parallel test interference
    let unique_suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let note_a_id = format!("qp-a-{}", unique_suffix);
    let note_b_id = format!("qp-b-{}", unique_suffix);
    let note_c_id = format!("qp-c-{}", unique_suffix);

    // 1. Initialize store 1 and create notes A, B, C with links
    qipu()
        .arg("init")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // Create note A
    qipu()
        .arg("create")
        .arg("Note A")
        .arg("--id")
        .arg(&note_a_id)
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // Create note B
    qipu()
        .arg("create")
        .arg("Note B")
        .arg("--id")
        .arg(&note_b_id)
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // Create note C
    qipu()
        .arg("create")
        .arg("Note C")
        .arg("--id")
        .arg(&note_c_id)
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // Add link from A to B (B will be pre-existing in store2)
    qipu()
        .arg("link")
        .arg("add")
        .arg(&note_a_id)
        .arg(&note_b_id)
        .arg("--type")
        .arg("supports")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // Add link from A to C (C will be newly loaded in store2)
    qipu()
        .arg("link")
        .arg("add")
        .arg(&note_a_id)
        .arg(&note_c_id)
        .arg("--type")
        .arg("related")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 2. Dump all notes to pack
    qipu()
        .arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store1_path)
        .assert()
        .success();

    // 3. Initialize store 2 with pre-existing note B (but not A or C)
    qipu()
        .arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // Create note B in store2 with same ID but different content
    qipu()
        .arg("create")
        .arg("Note B Pre-existing")
        .arg("--id")
        .arg(&note_b_id)
        .arg("--tag")
        .arg("pre-existing")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 4. Load pack with merge-links strategy
    qipu()
        .arg("load")
        .arg(&pack_file)
        .arg("--strategy")
        .arg("merge-links")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    // 5. Verify results:
    // - Note A should be created
    // - Note B should keep existing content (not overwritten)
    // - Note C should be created
    // - Note A's links:
    //   * Link to C should be PRESENT (C was newly loaded)
    //   * Link to B should NOT be present (B already existed)

    // Check note A - should have link to C but not to B
    let output_a = qipu()
        .arg("show")
        .arg(&note_a_id)
        .arg("--links")
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();
    let output_a_str = String::from_utf8_lossy(&output_a.stdout);

    // Should have link to newly loaded note C
    assert!(
        predicate::str::contains(&note_c_id).eval(&output_a_str),
        "Note A should have link to newly loaded note C\nOutput: {}",
        output_a_str
    );

    // Should NOT have link to pre-existing note B
    assert!(
        !predicate::str::contains(&note_b_id).eval(&output_a_str),
        "Note A should NOT have link to pre-existing note B\nOutput: {}",
        output_a_str
    );

    // Check note B - should preserve original content
    let output_b = qipu()
        .arg("show")
        .arg(&note_b_id)
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();
    let output_b_str = String::from_utf8_lossy(&output_b.stdout);

    assert!(
        predicate::str::contains("Note B Pre-existing").eval(&output_b_str),
        "Note B should preserve original title\nOutput: {}",
        output_b_str
    );
    assert!(
        predicate::str::contains("pre-existing").eval(&output_b_str),
        "Note B should preserve original tag\nOutput: {}",
        output_b_str
    );

    // Check note C - should be created
    let output_c = qipu()
        .arg("show")
        .arg(&note_c_id)
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();
    let output_c_str = String::from_utf8_lossy(&output_c.stdout);

    assert!(
        predicate::str::contains("Note C").eval(&output_c_str),
        "Note C should be created\nOutput: {}",
        output_c_str
    );
}
