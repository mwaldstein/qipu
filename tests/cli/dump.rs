use crate::cli::support::qipu;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_dump_max_hops_limits_traversal_depth() {
    let dir = tempdir().unwrap();
    let store_path = dir.path();
    let pack_file = dir.path().join("test.pack");

    // Initialize store
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Create a chain of notes: A -> B -> C -> D
    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note A")
        .arg("--id")
        .arg("note-a")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note B")
        .arg("--id")
        .arg("note-b")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note C")
        .arg("--id")
        .arg("note-c")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note D")
        .arg("--id")
        .arg("note-d")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Create links: A -> B, B -> C, C -> D
    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg("note-a")
        .arg("note-b")
        .arg("--type")
        .arg("next")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg("note-b")
        .arg("note-c")
        .arg("--type")
        .arg("next")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg("note-c")
        .arg("note-d")
        .arg("--type")
        .arg("next")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Dump with max-hops=1 from note-a (should only get A and B)
    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--note")
        .arg("note-a")
        .arg("--output")
        .arg(&pack_file)
        .arg("--max-hops")
        .arg("1")
        .arg("--direction")
        .arg("out")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Load and verify only A and B are in the pack
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();

    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    let output = qipu()
        .arg("list")
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();

    let list: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let ids: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap())
        .collect();

    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&"note-a"));
    assert!(ids.contains(&"note-b"));
    assert!(!ids.contains(&"note-c"));
    assert!(!ids.contains(&"note-d"));
}

#[test]
fn test_dump_direction_filters_traversal() {
    let dir = tempdir().unwrap();
    let store_path = dir.path();
    let pack_file = dir.path().join("test.pack");

    // Initialize store
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Create notes
    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note A")
        .arg("--id")
        .arg("note-a")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note B")
        .arg("--id")
        .arg("note-b")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note C")
        .arg("--id")
        .arg("note-c")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Create links: A -> B, A -> C, and also C -> A (bidirectional)
    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg("note-a")
        .arg("note-b")
        .arg("--type")
        .arg("related")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg("note-a")
        .arg("note-c")
        .arg("--type")
        .arg("related")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg("note-c")
        .arg("note-a")
        .arg("--type")
        .arg("references")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Dump with direction=out from note-a (should only get A, B, C via outbound links)
    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--note")
        .arg("note-a")
        .arg("--output")
        .arg(&pack_file)
        .arg("--max-hops")
        .arg("1")
        .arg("--direction")
        .arg("out")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Load and verify
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();

    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    let output = qipu()
        .arg("list")
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();

    let list: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let ids: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap())
        .collect();

    assert_eq!(ids.len(), 3);
    assert!(ids.contains(&"note-a"));
    assert!(ids.contains(&"note-b"));
    assert!(ids.contains(&"note-c"));
}

#[test]
fn test_dump_type_filter_affects_reachability() {
    let dir = tempdir().unwrap();
    let store_path = dir.path();
    let pack_file = dir.path().join("test.pack");

    // Initialize store
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Create notes
    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note A")
        .arg("--id")
        .arg("note-a")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note B")
        .arg("--id")
        .arg("note-b")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note C")
        .arg("--id")
        .arg("note-c")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Create links with different types: A -> B (type: important), A -> C (type: reference)
    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg("note-a")
        .arg("note-b")
        .arg("--type")
        .arg("important")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg("note-a")
        .arg("note-c")
        .arg("--type")
        .arg("reference")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Dump with type filter for "important" only
    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--note")
        .arg("note-a")
        .arg("--output")
        .arg(&pack_file)
        .arg("--max-hops")
        .arg("1")
        .arg("--direction")
        .arg("out")
        .arg("--type")
        .arg("important")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Load and verify only A and B are in the pack (C is excluded due to type filter)
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();

    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    let output = qipu()
        .arg("list")
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();

    let list: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let ids: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap())
        .collect();

    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&"note-a"));
    assert!(ids.contains(&"note-b"));
    assert!(!ids.contains(&"note-c"));
}

#[test]
fn test_dump_typed_only_excludes_inline_links() {
    let dir = tempdir().unwrap();
    let store_path = dir.path();
    let pack_file = dir.path().join("test.pack");

    // Initialize store
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Create notes
    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note A")
        .arg("--id")
        .arg("note-a")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note B")
        .arg("--id")
        .arg("note-b")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note C")
        .arg("--id")
        .arg("note-c")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Create typed link: A -> B
    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg("note-a")
        .arg("note-b")
        .arg("--type")
        .arg("related")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Create inline link by editing note A to reference C
    let note_a_path = store_path.join("notes").join("note-a-note-a.md");
    let content = fs::read_to_string(&note_a_path).unwrap();
    let updated_content = format!("{}\n\nSee [[note-c]] for more info.", content);
    fs::write(&note_a_path, updated_content).unwrap();

    // Rebuild index so inline link is detected
    let mut cmd = qipu();
    cmd.arg("index")
        .arg("--rebuild")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Dump with --typed-only flag
    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--note")
        .arg("note-a")
        .arg("--output")
        .arg(&pack_file)
        .arg("--max-hops")
        .arg("1")
        .arg("--direction")
        .arg("out")
        .arg("--typed-only")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Load and verify only A and B are in the pack (C is excluded due to typed-only filter)
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();

    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    let output = qipu()
        .arg("list")
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();

    let list: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let ids: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap())
        .collect();

    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&"note-a"));
    assert!(ids.contains(&"note-b"));
    assert!(!ids.contains(&"note-c"));
}

#[test]
fn test_dump_inline_only_excludes_typed_links() {
    let dir = tempdir().unwrap();
    let store_path = dir.path();
    let pack_file = dir.path().join("test.pack");

    // Initialize store
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Create notes
    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note A")
        .arg("--id")
        .arg("note-a")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note B")
        .arg("--id")
        .arg("note-b")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note C")
        .arg("--id")
        .arg("note-c")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Create typed link: A -> B
    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg("note-a")
        .arg("note-b")
        .arg("--type")
        .arg("related")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Create inline link by editing note A to reference C
    let note_a_path = store_path.join("notes").join("note-a-note-a.md");
    let content = fs::read_to_string(&note_a_path).unwrap();
    let updated_content = format!("{}\n\nSee [[note-c]] for more info.", content);
    fs::write(&note_a_path, updated_content).unwrap();

    // Rebuild index so inline link is detected
    let mut cmd = qipu();
    cmd.arg("index")
        .arg("--rebuild")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Dump with --inline-only flag
    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--note")
        .arg("note-a")
        .arg("--output")
        .arg(&pack_file)
        .arg("--max-hops")
        .arg("1")
        .arg("--direction")
        .arg("out")
        .arg("--inline-only")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Load and verify only A and C are in the pack (B is excluded due to inline-only filter)
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();

    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    let output = qipu()
        .arg("list")
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();

    let list: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let ids: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap())
        .collect();

    eprintln!("Dumped notes: {:?}", ids);
    assert_eq!(ids.len(), 2, "Expected 2 notes, got: {:?}", ids);
    assert!(ids.contains(&"note-a"), "Should contain note-a");
    assert!(ids.contains(&"note-c"), "Should contain note-c");
    assert!(
        !ids.contains(&"note-b"),
        "Should not contain note-b (filtered by --inline-only)"
    );
}

#[test]
fn test_dump_without_filters_includes_all_reachable_notes() {
    let dir = tempdir().unwrap();
    let store_path = dir.path();
    let pack_file = dir.path().join("test.pack");

    // Initialize store
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Create a diamond graph structure: A -> B, A -> C, B -> D, C -> D
    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note A")
        .arg("--id")
        .arg("note-a")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note B")
        .arg("--id")
        .arg("note-b")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note C")
        .arg("--id")
        .arg("note-c")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note D")
        .arg("--id")
        .arg("note-d")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Create links
    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg("note-a")
        .arg("note-b")
        .arg("--type")
        .arg("next")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg("note-a")
        .arg("note-c")
        .arg("--type")
        .arg("next")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg("note-b")
        .arg("note-d")
        .arg("--type")
        .arg("next")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg("note-c")
        .arg("note-d")
        .arg("--type")
        .arg("next")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Dump with no type filters (all notes should be reachable)
    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--note")
        .arg("note-a")
        .arg("--output")
        .arg(&pack_file)
        .arg("--max-hops")
        .arg("2")
        .arg("--direction")
        .arg("out")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Load and verify all 4 notes are in the pack
    let dir2 = tempdir().unwrap();
    let store2_path = dir2.path();

    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("load")
        .arg(&pack_file)
        .env("QIPU_STORE", store2_path)
        .assert()
        .success();

    let output = qipu()
        .arg("list")
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();

    let list: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let ids: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap())
        .collect();

    assert_eq!(ids.len(), 4);
    assert!(ids.contains(&"note-a"));
    assert!(ids.contains(&"note-b"));
    assert!(ids.contains(&"note-c"));
    assert!(ids.contains(&"note-d"));
}
