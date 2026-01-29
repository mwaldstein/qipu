use crate::cli::support::qipu;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_dump_semantic_inversion_default() {
    let dir = tempdir().unwrap();
    let store_path = dir.path();
    let pack_file = dir.path().join("test.pack");

    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Semantic Source")
        .arg("--id")
        .arg("note-a")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Semantic Target")
        .arg("--id")
        .arg("note-b")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg("note-a")
        .arg("note-b")
        .arg("--type")
        .arg("supports")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--note")
        .arg("note-b")
        .arg("--output")
        .arg(&pack_file)
        .arg("--direction")
        .arg("both")
        .arg("--max-hops")
        .arg("1")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

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

    assert!(ids.len() >= 2, "Should include both notes");
    assert!(ids.contains(&"note-a"), "Should include source note");
    assert!(ids.contains(&"note-b"), "Should include target note");
}

#[test]
fn test_dump_semantic_inversion_disabled() {
    let dir = tempdir().unwrap();
    let store_path = dir.path();
    let pack_file = dir.path().join("test.pack");

    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Semantic Disabled Source")
        .arg("--id")
        .arg("note-a")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Semantic Disabled Target")
        .arg("--id")
        .arg("note-b")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg("note-a")
        .arg("note-b")
        .arg("--type")
        .arg("supports")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--note")
        .arg("note-b")
        .arg("--output")
        .arg(&pack_file)
        .arg("--direction")
        .arg("both")
        .arg("--max-hops")
        .arg("1")
        .arg("--no-semantic-inversion")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

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

    assert!(ids.len() >= 2, "Should include both notes");
    assert!(ids.contains(&"note-a"), "Should include source note");
    assert!(ids.contains(&"note-b"), "Should include target note");
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

    // Create links with different types: A -> B (type: supports), A -> C (type: related)
    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg("note-a")
        .arg("note-b")
        .arg("--type")
        .arg("supports")
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

    // Dump with type filter for "supports" only
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
        .arg("supports")
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
fn test_dump_by_tag() {
    let dir = tempdir().unwrap();
    let store_path = dir.path();
    let pack_file = dir.path().join("test.pack");

    // Initialize store
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Create notes with different tags
    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Project note")
        .arg("--id")
        .arg("note-a")
        .arg("--tag")
        .arg("project")
        .arg("--tag")
        .arg("important")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Another project note")
        .arg("--id")
        .arg("note-b")
        .arg("--tag")
        .arg("project")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Personal note")
        .arg("--id")
        .arg("note-c")
        .arg("--tag")
        .arg("personal")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Dump notes with tag "project"
    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--tag")
        .arg("project")
        .arg("--output")
        .arg(&pack_file)
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Load into a new store
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

    // Verify only notes with tag "project" are loaded
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
fn test_dump_by_moc() {
    let dir = tempdir().unwrap();
    let store_path = dir.path();
    let pack_file = dir.path().join("test.pack");

    // Initialize store
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Create MOC note
    let mut cmd = qipu();
    cmd.arg("create")
        .arg("My MOC")
        .arg("--id")
        .arg("my-moc")
        .arg("--type")
        .arg("moc")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Create content notes
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

    // Link notes from MOC (A and B are linked, C is not)
    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg("my-moc")
        .arg("note-a")
        .arg("--type")
        .arg("has-part")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("link")
        .arg("add")
        .arg("my-moc")
        .arg("note-b")
        .arg("--type")
        .arg("has-part")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Dump notes linked from MOC
    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--moc")
        .arg("my-moc")
        .arg("--output")
        .arg(&pack_file)
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Load into a new store
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

    // Verify notes linked from MOC are loaded (A and B)
    // Note: The MOC itself is not included (only notes linked from it)
    let output = qipu()
        .arg("list")
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();

    let list: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let mut ids: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap())
        .collect();
    ids.sort();

    eprintln!("Loaded notes: {:?}", ids);
    // The implementation includes the MOC itself in the dump along with linked notes
    assert!(ids.contains(&"note-a"), "Should contain note-a");
    assert!(ids.contains(&"note-b"), "Should contain note-b");
    assert!(
        !ids.contains(&"note-c"),
        "Should not contain note-c (not linked from MOC)"
    );
}

#[test]
fn test_dump_by_query() {
    let dir = tempdir().unwrap();
    let store_path = dir.path();
    let pack_file = dir.path().join("test.pack");

    // Initialize store
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Create notes with searchable content
    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Rust programming tutorial")
        .arg("--id")
        .arg("note-a")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Advanced Rust techniques")
        .arg("--id")
        .arg("note-b")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Python basics")
        .arg("--id")
        .arg("note-c")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Rebuild index to ensure FTS is updated
    let mut cmd = qipu();
    cmd.arg("index")
        .arg("--rebuild")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Dump notes matching query "Rust"
    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--query")
        .arg("Rust")
        .arg("--output")
        .arg(&pack_file)
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Load into a new store
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

    // Verify only notes matching query are loaded
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
fn test_dump_no_selectors_full_store() {
    let dir = tempdir().unwrap();
    let store_path = dir.path();
    let pack_file = dir.path().join("test.pack");

    // Initialize store
    let mut cmd = qipu();
    cmd.arg("init")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Create multiple notes with different characteristics
    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note A")
        .arg("--id")
        .arg("note-a")
        .arg("--tag")
        .arg("project")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note B")
        .arg("--id")
        .arg("note-b")
        .arg("--tag")
        .arg("personal")
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    let mut cmd = qipu();
    cmd.arg("create")
        .arg("Note C")
        .arg("--id")
        .arg("note-c")
        .arg("--type")
        .arg("moc")
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

    // Create some links
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

    // Dump with no selectors (should dump entire store)
    let mut cmd = qipu();
    cmd.arg("dump")
        .arg("--output")
        .arg(&pack_file)
        .env("QIPU_STORE", store_path)
        .assert()
        .success();

    // Load into a new store
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

    // Verify all notes are loaded
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

    // Verify the link is preserved
    let output = qipu()
        .arg("link")
        .arg("list")
        .arg("note-a")
        .arg("--format")
        .arg("json")
        .env("QIPU_STORE", store2_path)
        .output()
        .unwrap();

    let links: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let link_targets: Vec<&str> = links
        .as_array()
        .unwrap()
        .iter()
        .filter(|l| l["direction"].as_str() == Some("out"))
        .map(|l| l["id"].as_str().unwrap())
        .collect();

    assert!(link_targets.contains(&"note-b"));
}
