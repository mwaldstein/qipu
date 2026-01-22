use crate::cli::support::{extract_id, qipu};
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_link_tree_single_node() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a single note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Root Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Tree should show just the root
    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id))
        .stdout(predicate::str::contains("Root Note"));
}

#[test]
fn test_link_tree_with_links() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a chain of notes
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Root"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Child 1"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    let output3 = qipu()
        .current_dir(dir.path())
        .args(["create", "Child 2"])
        .output()
        .unwrap();
    let id3 = extract_id(&output3);

    // Link root -> child1 -> child2
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id2, &id3, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Tree from root should show all nodes
    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id1])
        .assert()
        .success()
        .stdout(predicate::str::contains("Root"))
        .stdout(predicate::str::contains("Child 1"))
        .stdout(predicate::str::contains("Child 2"));
}

#[test]
fn test_link_tree_json_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "JSON Root"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "JSON Child"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "link", "tree", &id1])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"root\":"))
        .stdout(predicate::str::contains("\"notes\":"))
        .stdout(predicate::str::contains("\"links\":"))
        .stdout(predicate::str::contains("\"spanning_tree\":"));
}

#[test]
fn test_link_tree_records_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Records Root"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "link", "tree", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("H qipu=1 records=1 store="))
        .stdout(predicate::str::contains("mode=link.tree"))
        .stdout(predicate::str::contains("N "));
}

#[test]
fn test_link_tree_max_hops() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a chain of 5 notes
    let mut ids = Vec::new();
    for i in 0..5 {
        let output = qipu()
            .current_dir(dir.path())
            .args(["create", &format!("Node {}", i)])
            .output()
            .unwrap();
        ids.push(extract_id(&output));
    }

    // Link them in a chain
    for i in 0..4 {
        qipu()
            .current_dir(dir.path())
            .args(["link", "add", &ids[i], &ids[i + 1], "--type", "related"])
            .assert()
            .success();
    }

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // With max-hops=2, should only see nodes 0, 1, 2
    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &ids[0], "--max-hops", "2"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Node 0"))
        .stdout(predicate::str::contains("Node 1"))
        .stdout(predicate::str::contains("Node 2"))
        .stdout(predicate::str::contains("Node 3").not());
}

#[test]
fn test_link_tree_direction_out() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create 3 notes: A -> B, C -> A
    let output_a = qipu()
        .current_dir(dir.path())
        .args(["create", "Node A"])
        .output()
        .unwrap();
    let id_a = extract_id(&output_a);

    let output_b = qipu()
        .current_dir(dir.path())
        .args(["create", "Node B"])
        .output()
        .unwrap();
    let id_b = extract_id(&output_b);

    let output_c = qipu()
        .current_dir(dir.path())
        .args(["create", "Node C"])
        .output()
        .unwrap();
    let id_c = extract_id(&output_c);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_b, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_c, &id_a, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Tree from A with direction=out should show A -> B but not C
    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id_a, "--direction", "out"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Node A"))
        .stdout(predicate::str::contains("Node B"))
        .stdout(predicate::str::contains("Node C").not());
}

#[test]
fn test_link_tree_cycle_shows_seen() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create 3 notes: A -> B -> C -> A (cycle)
    let output_a = qipu()
        .current_dir(dir.path())
        .args(["create", "Node A"])
        .output()
        .unwrap();
    let id_a = extract_id(&output_a);

    let output_b = qipu()
        .current_dir(dir.path())
        .args(["create", "Node B"])
        .output()
        .unwrap();
    let id_b = extract_id(&output_b);

    let output_c = qipu()
        .current_dir(dir.path())
        .args(["create", "Node C"])
        .output()
        .unwrap();
    let id_c = extract_id(&output_c);

    // Create a cycle: A -> B -> C -> A
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_b, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_b, &id_c, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_c, &id_a, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Tree from A should show the cycle with (seen) marker
    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id_a])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain all three nodes
    assert!(stdout.contains("Node A"));
    assert!(stdout.contains("Node B"));
    assert!(stdout.contains("Node C"));

    // Should show "(seen)" for the back-edge to Node A
    assert!(stdout.contains("(seen)"));
}

#[test]
fn test_link_tree_max_hops_reports_truncation() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a chain of 5 notes: 0 -> 1 -> 2 -> 3 -> 4
    let mut ids = Vec::new();
    for i in 0..5 {
        let output = qipu()
            .current_dir(dir.path())
            .args(["create", &format!("Node {}", i)])
            .output()
            .unwrap();
        ids.push(extract_id(&output));
    }

    // Link them in a chain
    for i in 0..4 {
        qipu()
            .current_dir(dir.path())
            .args(["link", "add", &ids[i], &ids[i + 1], "--type", "related"])
            .assert()
            .success();
    }

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // With max-hops=2, traversal should be truncated because node 2 has outbound links
    // that cannot be explored
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "json",
            "link",
            "tree",
            &ids[0],
            "--max-hops",
            "2",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Should report truncation
    assert_eq!(json["truncated"], true);
    assert_eq!(json["truncation_reason"], "max_hops");

    // Should include nodes 0, 1, 2 but not 3, 4
    let note_ids: Vec<String> = json["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap().to_string())
        .collect();
    assert!(note_ids.contains(&ids[0]));
    assert!(note_ids.contains(&ids[1]));
    assert!(note_ids.contains(&ids[2]));
    assert!(!note_ids.contains(&ids[3]));
    assert!(!note_ids.contains(&ids[4]));
}

#[test]
fn test_link_tree_spanning_tree_ordering() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create root note and multiple children with different link types
    let output_root = qipu()
        .current_dir(dir.path())
        .args(["create", "Root"])
        .output()
        .unwrap();
    let id_root = extract_id(&output_root);

    // Create children with IDs that would sort differently by ID vs type
    let output_a = qipu()
        .current_dir(dir.path())
        .args(["create", "Node A"])
        .output()
        .unwrap();
    let id_a = extract_id(&output_a);

    let output_b = qipu()
        .current_dir(dir.path())
        .args(["create", "Node B"])
        .output()
        .unwrap();
    let id_b = extract_id(&output_b);

    let output_c = qipu()
        .current_dir(dir.path())
        .args(["create", "Node C"])
        .output()
        .unwrap();
    let id_c = extract_id(&output_c);

    // Add links with types that have different lexical ordering
    // "derived-from" < "related" < "supports" (alphabetically: d < r < s)
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_root, &id_c, "--type", "supports"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_root, &id_a, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_root, &id_b, "--type", "derived-from"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Get tree output in JSON format to check spanning tree ordering
    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id_root, "--format", "json"])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    // Spanning tree should be ordered by (hop, link_type, target_id)
    // Expected order: "derived-from" < "related" < "supports" (alphabetically)
    let spanning_tree = json["spanning_tree"].as_array().unwrap();
    assert_eq!(spanning_tree.len(), 3);

    // Extract types in order
    let types: Vec<&str> = spanning_tree
        .iter()
        .map(|e| e["type"].as_str().unwrap())
        .collect();

    // Types should be ordered: derived-from, related, supports
    assert_eq!(types[0], "derived-from");
    assert_eq!(types[1], "related");
    assert_eq!(types[2], "supports");

    // Verify target IDs
    assert_eq!(spanning_tree[0]["to"].as_str().unwrap(), id_b);
    assert_eq!(spanning_tree[1]["to"].as_str().unwrap(), id_a);
    assert_eq!(spanning_tree[2]["to"].as_str().unwrap(), id_c);
}

#[test]
fn test_link_tree_type_filter() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create root note and children with different link types
    let output_root = qipu()
        .current_dir(dir.path())
        .args(["create", "Root"])
        .output()
        .unwrap();
    let id_root = extract_id(&output_root);

    let output_child1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Child 1"])
        .output()
        .unwrap();
    let id_child1 = extract_id(&output_child1);

    let output_child2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Child 2"])
        .output()
        .unwrap();
    let id_child2 = extract_id(&output_child2);

    // Add typed links with different types
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_root, &id_child1, "--type", "supports"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_root, &id_child2, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // With --type supports, should only show Child 1
    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id_root, "--type", "supports"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Child 1"))
        .stdout(predicate::str::contains("Child 2").not());
}

#[test]
fn test_link_tree_exclude_type_filter() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output_root = qipu()
        .current_dir(dir.path())
        .args(["create", "Root"])
        .output()
        .unwrap();
    let id_root = extract_id(&output_root);

    let output_child1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Child 1"])
        .output()
        .unwrap();
    let id_child1 = extract_id(&output_child1);

    let output_child2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Child 2"])
        .output()
        .unwrap();
    let id_child2 = extract_id(&output_child2);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_root, &id_child1, "--type", "supports"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_root, &id_child2, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // With --exclude-type supports, should only show Child 2
    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id_root, "--exclude-type", "supports"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Child 2"))
        .stdout(predicate::str::contains("Child 1").not());
}

#[test]
fn test_link_tree_typed_only() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output_child1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Typed Child"])
        .output()
        .unwrap();
    let id_child1 = extract_id(&output_child1);

    let output_child2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Inline Child"])
        .output()
        .unwrap();
    let id_child2 = extract_id(&output_child2);

    let output_root = qipu()
        .current_dir(dir.path())
        .args(["capture", "--title", "Root"])
        .write_stdin(format!("See [[{}]]", id_child2))
        .output()
        .unwrap();
    let id_root = extract_id(&output_root);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_root, &id_child1, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id_root, "--typed-only"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Typed Child"))
        .stdout(predicate::str::contains("Inline Child").not());
}

#[test]
fn test_link_tree_inline_only() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output_child1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Typed Child"])
        .output()
        .unwrap();
    let id_child1 = extract_id(&output_child1);

    let output_child2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Inline Child"])
        .output()
        .unwrap();
    let id_child2 = extract_id(&output_child2);

    let output_root = qipu()
        .current_dir(dir.path())
        .args(["capture", "--title", "Root"])
        .write_stdin(format!("See [[{}]]", id_child2))
        .output()
        .unwrap();
    let id_root = extract_id(&output_root);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_root, &id_child1, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id_root, "--inline-only"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Inline Child"))
        .stdout(predicate::str::contains("Typed Child").not());
}

#[test]
fn test_link_tree_direction_in() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output_a = qipu()
        .current_dir(dir.path())
        .args(["create", "Node A"])
        .output()
        .unwrap();
    let id_a = extract_id(&output_a);

    let output_b = qipu()
        .current_dir(dir.path())
        .args(["create", "Node B"])
        .output()
        .unwrap();
    let id_b = extract_id(&output_b);

    let output_c = qipu()
        .current_dir(dir.path())
        .args(["create", "Node C"])
        .output()
        .unwrap();
    let id_c = extract_id(&output_c);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_b, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_b, &id_c, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id_b, "--direction", "in"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Node B"))
        .stdout(predicate::str::contains("Node A"))
        .stdout(predicate::str::contains("Node C").not());
}

#[test]
fn test_link_tree_direction_both() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output_a = qipu()
        .current_dir(dir.path())
        .args(["create", "Node A"])
        .output()
        .unwrap();
    let id_a = extract_id(&output_a);

    let output_b = qipu()
        .current_dir(dir.path())
        .args(["create", "Node B"])
        .output()
        .unwrap();
    let id_b = extract_id(&output_b);

    let output_c = qipu()
        .current_dir(dir.path())
        .args(["create", "Node C"])
        .output()
        .unwrap();
    let id_c = extract_id(&output_c);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_b, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_b, &id_c, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id_b, "--direction", "both"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Node B"))
        .stdout(predicate::str::contains("Node A"))
        .stdout(predicate::str::contains("Node C"));
}

#[test]
fn test_link_tree_min_value_filter_all_match() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create notes with different values
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Root Note"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "High Value Note"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    // Set values
    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id1, "90"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id2, "95"])
        .assert()
        .success();

    // Link root -> child
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Both notes should appear with --min-value 80
    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id1, "--min-value", "80"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Root Note"))
        .stdout(predicate::str::contains("High Value Note"));
}

#[test]
fn test_link_tree_min_value_filter_some_match() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create notes with different values
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Root Note"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Low Value Note"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    let output3 = qipu()
        .current_dir(dir.path())
        .args(["create", "High Value Note"])
        .output()
        .unwrap();
    let id3 = extract_id(&output3);

    // Set values
    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id1, "90"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id2, "30"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id3, "95"])
        .assert()
        .success();

    // Link root -> child1, root -> child2
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id3, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Only root and high value note should appear with --min-value 80
    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id1, "--min-value", "80"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Root Note"))
        .stdout(predicate::str::contains("High Value Note"))
        .stdout(predicate::str::contains("Low Value Note").not());
}

#[test]
fn test_link_tree_min_value_filter_with_defaults() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create notes with default and explicit values
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Root Note"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Default Value Note"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    // Set only root value, leave child as default (treated as 50)
    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id1, "90"])
        .assert()
        .success();

    // Link root -> child
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Both notes should appear with --min-value 50 (default is treated as 50)
    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id1, "--min-value", "50"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Root Note"))
        .stdout(predicate::str::contains("Default Value Note"));
}

#[test]
fn test_link_tree_min_value_filter_excludes_root() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note with low value
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Low Value Root"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "High Value Child"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    // Set values
    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id1, "20"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id2, "90"])
        .assert()
        .success();

    // Link root -> child
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // No notes should appear if root is filtered out
    qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id1, "--min-value", "80"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Low Value Root").not())
        .stdout(predicate::str::contains("High Value Child").not())
        .stdout(predicate::str::contains("No notes found"));
}

#[test]
fn test_link_tree_semantic_inversion_default() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create A -> B with "supports" link
    let output_a = qipu()
        .current_dir(dir.path())
        .args(["create", "Note A"])
        .output()
        .unwrap();
    let id_a = extract_id(&output_a);

    let output_b = qipu()
        .current_dir(dir.path())
        .args(["create", "Note B"])
        .output()
        .unwrap();
    let id_b = extract_id(&output_b);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_b, "--type", "supports"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Tree from B with direction=in should show virtual "supported-by" edge (semantic inversion enabled by default)
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "tree",
            &id_b,
            "--direction",
            "in",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    // Should have a link with type "supported-by" and source "virtual"
    let links = json["links"].as_array().unwrap();
    let inverted_link = links
        .iter()
        .find(|l| l["from"].as_str() == Some(&id_b) && l["to"].as_str() == Some(&id_a));

    assert!(inverted_link.is_some(), "Should have virtual inverted link");
    let link = inverted_link.unwrap();
    assert_eq!(link["type"].as_str().unwrap(), "supported-by");
    assert_eq!(link["source"].as_str().unwrap(), "virtual");
}

#[test]
fn test_link_tree_semantic_inversion_disabled() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create A -> B with "supports" link
    let output_a = qipu()
        .current_dir(dir.path())
        .args(["create", "Note A"])
        .output()
        .unwrap();
    let id_a = extract_id(&output_a);

    let output_b = qipu()
        .current_dir(dir.path())
        .args(["create", "Note B"])
        .output()
        .unwrap();
    let id_b = extract_id(&output_b);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_b, "--type", "supports"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Tree from B with direction=in and --no-semantic-inversion should show raw backlink with original "supports" type
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "tree",
            &id_b,
            "--direction",
            "in",
            "--no-semantic-inversion",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    // Should have the original link A -> B with type "supports"
    let links = json["links"].as_array().unwrap();
    let raw_link = links
        .iter()
        .find(|l| l["from"].as_str() == Some(&id_a) && l["to"].as_str() == Some(&id_b));

    assert!(raw_link.is_some(), "Should have raw backlink");
    let link = raw_link.unwrap();
    assert_eq!(link["type"].as_str().unwrap(), "supports");
    assert_eq!(link["source"].as_str().unwrap(), "typed");
}

#[test]
fn test_link_tree_semantic_inversion_type_filter() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create A -> B -> C with "supports" links (same structure as test_link_tree_direction_in)
    let output_a = qipu()
        .current_dir(dir.path())
        .args(["create", "Note A"])
        .output()
        .unwrap();
    let id_a = extract_id(&output_a);

    let output_b = qipu()
        .current_dir(dir.path())
        .args(["create", "Note B"])
        .output()
        .unwrap();
    let id_b = extract_id(&output_b);

    let output_c = qipu()
        .current_dir(dir.path())
        .args(["create", "Note C"])
        .output()
        .unwrap();
    let id_c = extract_id(&output_c);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_b, "--type", "supports"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_b, &id_c, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // With semantic inversion (default), filtering by --type supported-by should match the virtual inverted edge
    qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "tree",
            &id_b,
            "--direction",
            "in",
            "--type",
            "supported-by",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Note A"))
        .stdout(predicate::str::contains("Note C").not());

    // When filtering by --type supports (the original link type) with semantic inversion,
    // should NOT match because the inverted edge has type "supported-by"
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "tree",
            &id_b,
            "--direction",
            "in",
            "--type",
            "supports",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("Note A"),
        "Should not find Note A when filtering by original type with semantic inversion"
    );

    // Without semantic inversion and direction=in, the raw backlink should still be traversable
    // (check JSON output since human output formatter has a known limitation)
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "tree",
            &id_b,
            "--direction",
            "in",
            "--no-semantic-inversion",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let note_ids: Vec<&str> = json["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap())
        .collect();
    assert!(note_ids.contains(&id_a.as_str()), "Should find Note A");
    assert!(!note_ids.contains(&id_c.as_str()), "Should not find Note C");

    // Check that the link retains its original type "supports" (not inverted)
    let links = json["links"].as_array().unwrap();
    let link = links
        .iter()
        .find(|l| l["from"].as_str() == Some(&id_a) && l["to"].as_str() == Some(&id_b));
    assert!(link.is_some(), "Should have raw link A -> B");
    assert_eq!(link.unwrap()["type"].as_str().unwrap(), "supports");

    // When filtering by --type supports (the original link type) without semantic inversion, should match
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "tree",
            &id_b,
            "--direction",
            "in",
            "--no-semantic-inversion",
            "--type",
            "supports",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let note_ids: Vec<&str> = json["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap())
        .collect();
    assert!(
        note_ids.contains(&id_a.as_str()),
        "Should find Note A when filtering by original type"
    );

    // When filtering by --type supported-by without semantic inversion, should NOT match
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "tree",
            &id_b,
            "--direction",
            "in",
            "--no-semantic-inversion",
            "--type",
            "supported-by",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let note_ids: Vec<&str> = json["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap())
        .collect();
    assert!(
        !note_ids.contains(&id_a.as_str()),
        "Should not find Note A when filtering by inverted type without semantic inversion"
    );
}

#[test]
fn test_link_tree_max_nodes_truncation() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a larger graph with multiple branches
    // Root -> A -> A1
    //      -> B -> B1
    //      -> C -> C1
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Root"])
        .output()
        .unwrap();
    let root = extract_id(&output);

    let mut child_ids = Vec::new();
    for label in ["A", "B", "C"] {
        let output = qipu()
            .current_dir(dir.path())
            .args(["create", label])
            .output()
            .unwrap();
        let child_id = extract_id(&output);
        child_ids.push(child_id.clone());

        // Link root to child
        qipu()
            .current_dir(dir.path())
            .args(["link", "add", &root, &child_id, "--type", "related"])
            .assert()
            .success();

        // Create grandchild
        let output = qipu()
            .current_dir(dir.path())
            .args(["create", &format!("{} Child", label)])
            .output()
            .unwrap();
        let grandchild_id = extract_id(&output);

        // Link child to grandchild
        qipu()
            .current_dir(dir.path())
            .args([
                "link",
                "add",
                &child_id,
                &grandchild_id,
                "--type",
                "related",
            ])
            .assert()
            .success();
    }

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // With max-nodes=3, we should get root + 2 children (truncated before exploring all nodes)
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "json",
            "link",
            "tree",
            &root,
            "--max-nodes",
            "3",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Should report truncation
    assert_eq!(json["truncated"], true);
    assert_eq!(json["truncation_reason"], "max_nodes");

    // Should have exactly 3 nodes (root + 2 children)
    let note_count = json["notes"].as_array().unwrap().len();
    assert_eq!(note_count, 3);
}

#[test]
fn test_link_tree_max_edges_truncation() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a graph with multiple edges
    // Root -> A, B, C, D
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Root"])
        .output()
        .unwrap();
    let root = extract_id(&output);

    let mut child_ids = Vec::new();
    for label in ["A", "B", "C", "D"] {
        let output = qipu()
            .current_dir(dir.path())
            .args(["create", label])
            .output()
            .unwrap();
        let child_id = extract_id(&output);
        child_ids.push(child_id.clone());

        // Link root to child
        qipu()
            .current_dir(dir.path())
            .args(["link", "add", &root, &child_id, "--type", "related"])
            .assert()
            .success();
    }

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // With max-edges=2, we should only get 2 edges
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "json",
            "link",
            "tree",
            &root,
            "--max-edges",
            "2",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Should report truncation
    assert_eq!(json["truncated"], true);
    assert_eq!(json["truncation_reason"], "max_edges");

    // Should have exactly 2 edges
    let edge_count = json["links"].as_array().unwrap().len();
    assert_eq!(edge_count, 2);
}

#[test]
fn test_link_tree_max_fanout_truncation() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a graph where root has many children
    // Root -> A, B, C, D, E
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Root"])
        .output()
        .unwrap();
    let root = extract_id(&output);

    let mut child_ids = Vec::new();
    for label in ["A", "B", "C", "D", "E"] {
        let output = qipu()
            .current_dir(dir.path())
            .args(["create", label])
            .output()
            .unwrap();
        let child_id = extract_id(&output);
        child_ids.push(child_id.clone());

        // Link root to child
        qipu()
            .current_dir(dir.path())
            .args(["link", "add", &root, &child_id, "--type", "related"])
            .assert()
            .success();
    }

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // With max-fanout=2, root should only expand 2 children
    // Use --direction out to avoid counting inverted edges
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "json",
            "link",
            "tree",
            &root,
            "--max-fanout",
            "2",
            "--direction",
            "out",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Should report truncation
    assert_eq!(json["truncated"], true);
    assert_eq!(json["truncation_reason"], "max_fanout");

    // Should have root + 2 children = 3 nodes
    let note_count = json["notes"].as_array().unwrap().len();
    assert_eq!(note_count, 3);

    // Should have exactly 2 edges (root to 2 children)
    let edge_count = json["links"].as_array().unwrap().len();
    assert_eq!(edge_count, 2);
}

#[test]
fn test_link_tree_records_max_chars_no_truncation() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a small graph
    let output_root = qipu()
        .current_dir(dir.path())
        .args(["create", "Root"])
        .output()
        .unwrap();
    let id_root = extract_id(&output_root);

    let output_child = qipu()
        .current_dir(dir.path())
        .args(["create", "Child"])
        .output()
        .unwrap();
    let id_child = extract_id(&output_child);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_root, &id_child, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Use a large max-chars that won't truncate
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "records",
            "link",
            "tree",
            &id_root,
            "--max-chars",
            "10000",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Header should show truncated=false
    assert!(
        stdout.contains("truncated=false"),
        "Expected truncated=false in header"
    );

    // Should include all notes
    assert!(stdout.contains(&format!("N {}", id_root)));
    assert!(stdout.contains(&format!("N {}", id_child)));

    // Should include the edge
    assert!(stdout.contains(&format!("E {} related {}", id_root, id_child)));
}

#[test]
fn test_link_tree_records_max_chars_truncation() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a graph with several notes
    let output_root = qipu()
        .current_dir(dir.path())
        .args(["create", "Root"])
        .output()
        .unwrap();
    let id_root = extract_id(&output_root);

    let mut child_ids = Vec::new();
    for i in 1..=5 {
        let output = qipu()
            .current_dir(dir.path())
            .args(["create", &format!("Child {}", i)])
            .output()
            .unwrap();
        let id = extract_id(&output);
        child_ids.push(id.clone());

        qipu()
            .current_dir(dir.path())
            .args(["link", "add", &id_root, &id, "--type", "related"])
            .assert()
            .success();
    }

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Use a small max-chars that will truncate
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "records",
            "link",
            "tree",
            &id_root,
            "--max-chars",
            "200",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Header should show truncated=true
    assert!(
        stdout.contains("truncated=true"),
        "Expected truncated=true in header"
    );

    // Should include the header line
    assert!(stdout.contains("H qipu=1 records=1"));

    // Count total characters in output (including newlines)
    let total_chars = stdout.len();
    assert!(
        total_chars <= 200,
        "Output exceeded max-chars budget: {} > 200",
        total_chars
    );
}

#[test]
fn test_link_tree_records_max_chars_header_only() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output_root = qipu()
        .current_dir(dir.path())
        .args(["create", "Root"])
        .output()
        .unwrap();
    let id_root = extract_id(&output_root);

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Use a max-chars that can only fit the header (header is ~115 chars)
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "records",
            "link",
            "tree",
            &id_root,
            "--max-chars",
            "120",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should have truncated=true
    assert!(
        stdout.contains("truncated=true"),
        "Expected truncated=true in header"
    );

    // Should only have the header line (no N or E records)
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(
        lines.len(),
        1,
        "Expected only header line, got {} lines",
        lines.len()
    );
    assert!(lines[0].starts_with("H "));

    // Verify total length is within budget
    assert!(stdout.len() <= 120);
}
