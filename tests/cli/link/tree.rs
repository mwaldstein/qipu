use crate::cli::support::qipu;
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
