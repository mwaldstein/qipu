use crate::cli::support::qipu;
use predicates::prelude::*;
use tempfile::tempdir;

// ============================================================================
// Link command tests
// ============================================================================

#[test]
fn test_link_list_empty() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note without links
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Isolated Note"])
        .output()
        .unwrap();

    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // First build index
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // List links should show no links
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("No links found"));
}

#[test]
fn test_link_add_and_list() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Source Note"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Target Note"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    // Add a link
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "supports"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added link"));

    // Build index
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // List links from source should show outbound link
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id1])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id2))
        .stdout(predicate::str::contains("supports"));

    // List links from target should show inbound link as virtual inverted edge by default
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id2])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id1))
        .stdout(predicate::str::contains("supported-by"))
        .stdout(predicate::str::contains("(virtual)"));

    // List links from target with --no-semantic-inversion should show raw inbound link
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id2, "--no-semantic-inversion"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id1))
        .stdout(predicate::str::contains("supports"))
        .stdout(predicate::str::contains("<-"));
}

#[test]
fn test_link_add_idempotent() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note A"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note B"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    // Add a link
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added link"));

    // Adding the same link again should report unchanged
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success()
        .stdout(predicate::str::contains("already exists"));
}

#[test]
fn test_link_remove() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note A"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note B"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    // Add a link
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "derived-from"])
        .assert()
        .success();

    // Remove the link
    qipu()
        .current_dir(dir.path())
        .args(["link", "remove", &id1, &id2, "--type", "derived-from"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed link"));

    // Build index
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // List links should show no links
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id1])
        .assert()
        .success()
        .stdout(predicate::str::contains("No links found"));
}

#[test]
fn test_link_list_json_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes and link them
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "JSON Source"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "JSON Target"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

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

    // List in JSON format
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "link", "list", &id1])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"direction\": \"out\""))
        .stdout(predicate::str::contains("\"source\": \"typed\""));
}

#[test]
fn test_link_list_direction_filter() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes and link them
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Direction Source"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Direction Target"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

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

    // List only outbound from source
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id1, "--direction", "out"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id2));

    // List only inbound to source should be empty
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id1, "--direction", "in"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No links found"));

    // List only inbound to target should show the link
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id2, "--direction", "in"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id1));
}

#[test]
fn test_link_list_records_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes and link them
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Records Source"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Records Target"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

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

    // List in records format
    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "link", "list", &id1])
        .assert()
        .success()
        .stdout(predicate::str::contains("H qipu=1 records=1 store="))
        .stdout(predicate::str::contains("mode=link.list"))
        .stdout(predicate::str::contains("E "));
}

// ============================================================================
// Link tree command tests
// ============================================================================

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
    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();

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
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Child 1"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    let output3 = qipu()
        .current_dir(dir.path())
        .args(["create", "Child 2"])
        .output()
        .unwrap();
    let id3 = String::from_utf8_lossy(&output3.stdout).trim().to_string();

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
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "JSON Child"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

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
    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();

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
        ids.push(String::from_utf8_lossy(&output.stdout).trim().to_string());
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
    let id_a = String::from_utf8_lossy(&output_a.stdout).trim().to_string();

    let output_b = qipu()
        .current_dir(dir.path())
        .args(["create", "Node B"])
        .output()
        .unwrap();
    let id_b = String::from_utf8_lossy(&output_b.stdout).trim().to_string();

    let output_c = qipu()
        .current_dir(dir.path())
        .args(["create", "Node C"])
        .output()
        .unwrap();
    let id_c = String::from_utf8_lossy(&output_c.stdout).trim().to_string();

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

// ============================================================================
// Link path command tests
// ============================================================================

#[test]
fn test_link_path_direct() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Start"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "End"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

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

    // Find path from start to end
    qipu()
        .current_dir(dir.path())
        .args(["link", "path", &id1, &id2])
        .assert()
        .success()
        .stdout(predicate::str::contains("Start"))
        .stdout(predicate::str::contains("End"))
        .stdout(predicate::str::contains("Path length: 1 hop"));
}

#[test]
fn test_link_path_multi_hop() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create A -> B -> C
    let output_a = qipu()
        .current_dir(dir.path())
        .args(["create", "Node A"])
        .output()
        .unwrap();
    let id_a = String::from_utf8_lossy(&output_a.stdout).trim().to_string();

    let output_b = qipu()
        .current_dir(dir.path())
        .args(["create", "Node B"])
        .output()
        .unwrap();
    let id_b = String::from_utf8_lossy(&output_b.stdout).trim().to_string();

    let output_c = qipu()
        .current_dir(dir.path())
        .args(["create", "Node C"])
        .output()
        .unwrap();
    let id_c = String::from_utf8_lossy(&output_c.stdout).trim().to_string();

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

    // Find path from A to C (2 hops)
    qipu()
        .current_dir(dir.path())
        .args(["link", "path", &id_a, &id_c])
        .assert()
        .success()
        .stdout(predicate::str::contains("Node A"))
        .stdout(predicate::str::contains("Node B"))
        .stdout(predicate::str::contains("Node C"))
        .stdout(predicate::str::contains("Path length: 2 hop"));
}

#[test]
fn test_link_path_not_found() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two unconnected notes
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Isolated 1"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Isolated 2"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Path should not be found
    qipu()
        .current_dir(dir.path())
        .args(["link", "path", &id1, &id2])
        .assert()
        .success()
        .stdout(predicate::str::contains("No path found"));
}

#[test]
fn test_link_path_json_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "JSON Start"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "JSON End"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

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
        .args(["--format", "json", "link", "path", &id1, &id2])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"from\":"))
        .stdout(predicate::str::contains("\"to\":"))
        .stdout(predicate::str::contains("\"found\": true"))
        .stdout(predicate::str::contains("\"notes\":"))
        .stdout(predicate::str::contains("\"links\":"));
}

#[test]
fn test_link_path_records_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Records Start"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Records End"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

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
        .args(["--format", "records", "link", "path", &id1, &id2])
        .assert()
        .success()
        .stdout(predicate::str::contains("H qipu=1 records=1 store="))
        .stdout(predicate::str::contains("mode=link.path"))
        .stdout(predicate::str::contains("found=true"));
}

// ============================================================================
// Compaction visibility tests for link commands
// ============================================================================

#[test]
fn test_link_list_with_compaction() {
    use std::fs;

    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create three notes: note1 -> note2, and digest that compacts note2
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 1"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 2"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    let output3 = qipu()
        .current_dir(dir.path())
        .args(["create", "Digest Note"])
        .output()
        .unwrap();
    let digest_id = String::from_utf8_lossy(&output3.stdout).trim().to_string();

    // Add link from note1 to note2
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    // Modify digest note to compact note2
    let notes_dir = dir.path().join(".qipu/notes");
    for entry in fs::read_dir(&notes_dir).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        if name.to_string_lossy().starts_with(&digest_id) {
            let note_content = fs::read_to_string(entry.path()).unwrap();
            // Insert compacts field in frontmatter
            let new_content = note_content.replace(
                &format!("id: {}", digest_id),
                &format!("id: {}\ncompacts:\n  - {}", digest_id, id2),
            );
            fs::write(entry.path(), new_content).unwrap();
            break;
        }
    }

    // Rebuild index to pick up compaction
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // link list on note1 should show link to digest_id (canonical), not note2
    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id1])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show canonical ID (digest)
    assert!(stdout.contains(&digest_id));
    // Should NOT show compacted note
    assert!(!stdout.contains(&id2));

    // link list on digest should show inbound link from note1
    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "list", &digest_id])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains(&id1));
}

#[test]
fn test_link_list_records_max_chars() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Link List Budget A"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Link List Budget B"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    let output3 = qipu()
        .current_dir(dir.path())
        .args(["create", "Link List Budget C"])
        .output()
        .unwrap();
    let id3 = String::from_utf8_lossy(&output3.stdout).trim().to_string();

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
        .args([
            "--format",
            "records",
            "link",
            "list",
            &id1,
            "--max-chars",
            "120",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("mode=link.list"))
        .stdout(predicate::str::contains("truncated=true"))
        .stdout(predicate::str::contains("N ").not());
}

#[test]
fn test_link_tree_with_compaction() {
    use std::fs;

    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a scenario that would create a self-loop without compaction:
    // note1 -> note2, note2 -> note3, then compact all into digest
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 1"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 2"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    let output3 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 3"])
        .output()
        .unwrap();
    let id3 = String::from_utf8_lossy(&output3.stdout).trim().to_string();

    let output_digest = qipu()
        .current_dir(dir.path())
        .args(["create", "Digest Note"])
        .output()
        .unwrap();
    let digest_id = String::from_utf8_lossy(&output_digest.stdout)
        .trim()
        .to_string();

    // Add links: note1 -> note2 -> note3
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

    // Modify digest to compact note1 and note2
    let notes_dir = dir.path().join(".qipu/notes");
    for entry in fs::read_dir(&notes_dir).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        if name.to_string_lossy().starts_with(&digest_id) {
            let note_content = fs::read_to_string(entry.path()).unwrap();
            let new_content = note_content.replace(
                &format!("id: {}", digest_id),
                &format!("id: {}\ncompacts:\n  - {}\n  - {}", digest_id, id1, id2),
            );
            fs::write(entry.path(), new_content).unwrap();
            break;
        }
    }

    // Rebuild index
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Tree from digest should show contracted graph (no self-loop)
    // It should show: digest -> note3
    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &digest_id])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show digest and note3
    assert!(stdout.contains(&digest_id));
    assert!(stdout.contains(&id3));
    // Should NOT show compacted notes
    assert!(!stdout.contains(&id1));
    assert!(!stdout.contains(&id2));

    // Tree from note3 going inbound should also use canonical IDs
    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id3, "--direction", "in"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains(&digest_id));
    assert!(!stdout.contains(&id1));
    assert!(!stdout.contains(&id2));
}

#[test]
fn test_link_path_with_compaction() {
    use std::fs;

    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a path: start -> middle -> end
    // Then compact middle into digest
    let output_start = qipu()
        .current_dir(dir.path())
        .args(["create", "Start Note"])
        .output()
        .unwrap();
    let start_id = String::from_utf8_lossy(&output_start.stdout)
        .trim()
        .to_string();

    let output_middle = qipu()
        .current_dir(dir.path())
        .args(["create", "Middle Note"])
        .output()
        .unwrap();
    let middle_id = String::from_utf8_lossy(&output_middle.stdout)
        .trim()
        .to_string();

    let output_end = qipu()
        .current_dir(dir.path())
        .args(["create", "End Note"])
        .output()
        .unwrap();
    let end_id = String::from_utf8_lossy(&output_end.stdout)
        .trim()
        .to_string();

    let output_digest = qipu()
        .current_dir(dir.path())
        .args(["create", "Digest Note"])
        .output()
        .unwrap();
    let digest_id = String::from_utf8_lossy(&output_digest.stdout)
        .trim()
        .to_string();

    // Add links: start -> middle -> end
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &start_id, &middle_id, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &middle_id, &end_id, "--type", "related"])
        .assert()
        .success();

    // Modify digest to compact middle
    let notes_dir = dir.path().join(".qipu/notes");
    for entry in fs::read_dir(&notes_dir).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        if name.to_string_lossy().starts_with(&digest_id) {
            let note_content = fs::read_to_string(entry.path()).unwrap();
            let new_content = note_content.replace(
                &format!("id: {}", digest_id),
                &format!("id: {}\ncompacts:\n  - {}", digest_id, middle_id),
            );
            fs::write(entry.path(), new_content).unwrap();
            break;
        }
    }

    // Rebuild index
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Path from start to end should go through digest (canonical), not middle
    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "path", &start_id, &end_id])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show start -> digest -> end
    assert!(stdout.contains(&start_id));
    assert!(stdout.contains(&digest_id));
    assert!(stdout.contains(&end_id));
    // Should NOT show compacted middle note
    assert!(!stdout.contains(&middle_id));
    assert!(stdout.contains("Path length: 2 hop"));
}

#[test]
fn test_link_no_resolve_compaction_flag() {
    use std::fs;

    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create note1 -> note2, and digest that compacts note2
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 1"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 2"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    let output3 = qipu()
        .current_dir(dir.path())
        .args(["create", "Digest Note"])
        .output()
        .unwrap();
    let digest_id = String::from_utf8_lossy(&output3.stdout).trim().to_string();

    // Add link
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    // Modify digest to compact note2
    let notes_dir = dir.path().join(".qipu/notes");
    for entry in fs::read_dir(&notes_dir).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        if name.to_string_lossy().starts_with(&digest_id) {
            let note_content = fs::read_to_string(entry.path()).unwrap();
            let new_content = note_content.replace(
                &format!("id: {}", digest_id),
                &format!("id: {}\ncompacts:\n  - {}", digest_id, id2),
            );
            fs::write(entry.path(), new_content).unwrap();
            break;
        }
    }

    // Rebuild index
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Test link list WITH --no-resolve-compaction flag
    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id1, "--no-resolve-compaction"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show the raw compacted note (note2), NOT the digest
    assert!(stdout.contains(&id2));
    assert!(!stdout.contains(&digest_id));

    // Test link tree WITH --no-resolve-compaction flag
    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id1, "--no-resolve-compaction"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show raw structure (note1 -> note2)
    assert!(stdout.contains(&id1));
    assert!(stdout.contains(&id2));
    // Digest shouldn't appear since we're showing raw links
    assert!(!stdout.contains(&digest_id));

    // Test link path WITH --no-resolve-compaction flag
    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "path", &id1, &id2, "--no-resolve-compaction"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show raw path (note1 -> note2)
    assert!(stdout.contains(&id1));
    assert!(stdout.contains(&id2));
    assert!(stdout.contains("Path length: 1 hop"));
}

#[test]
fn test_custom_link_inversion() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create custom config with inversion
    let config_path = dir.path().join(".qipu/config.toml");
    let config_content = r#"
version = 1
default_note_type = "fleeting"

[links.inverses]
recommends = "recommended-by"
"recommended-by" = "recommends"

[links.descriptions]
recommends = "This note recommends another note"
"#;
    std::fs::write(config_path, config_content).unwrap();

    // Create two notes
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 1"])
        .output()
        .unwrap();
    let id1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Note 2"])
        .output()
        .unwrap();
    let id2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    // Add custom link
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "recommends"])
        .assert()
        .success();

    // Build index
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // List links from target should show custom inverted edge
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id2])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id1))
        .stdout(predicate::str::contains("recommended-by"))
        .stdout(predicate::str::contains("(virtual)"));
}
