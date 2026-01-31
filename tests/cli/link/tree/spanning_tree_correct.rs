//! Tests for link command
use crate::support::{extract_id, qipu, setup_test_dir};

#[test]
fn test_link_tree_spanning_tree_not_all_links() {
    let dir = setup_test_dir();

    // Create a graph where Node A links to both B and C,
    // and Node B also links to C
    let output_root = qipu()
        .current_dir(dir.path())
        .args(["create", "Root"])
        .output()
        .unwrap();
    let id_root = extract_id(&output_root);

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

    // Root -> A, Root -> B, Root -> C
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_root, &id_a, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_root, &id_b, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_root, &id_c, "--type", "related"])
        .assert()
        .success();

    // A -> B, A -> C
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_b, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_c, "--type", "related"])
        .assert()
        .success();

    // B -> C
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

    // Get JSON output to check spanning_tree
    let json_output = qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id_root, "--format", "json"])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&json_output.stdout).unwrap();

    // The spanning_tree should only have 3 edges (first discovery of each node)
    let spanning_tree = json["spanning_tree"].as_array().unwrap();
    assert_eq!(
        spanning_tree.len(),
        3,
        "spanning_tree should have exactly 3 edges (one per child of root)"
    );

    // All spanning_tree edges should have from = root
    for entry in spanning_tree {
        assert_eq!(
            entry["from"].as_str().unwrap(),
            id_root,
            "All spanning_tree edges should start from root"
        );
    }

    // links[] will have more edges (all discovered edges)
    let links = json["links"].as_array().unwrap();
    assert!(
        links.len() > 3,
        "links[] should contain all discovered edges, not just spanning tree"
    );

    // Now check human output - it should show only the spanning tree
    let human_output = qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id_root])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&human_output.stdout);

    // Root should have 3 direct children in the human view (A, B, C)
    let root_line_count = stdout
        .lines()
        .filter(|line| line.contains(&id_root))
        .count();
    assert_eq!(root_line_count, 1, "Root should appear once");

    // Each of A, B, C should appear exactly once
    let a_count = stdout.matches(&id_a).count();
    let b_count = stdout.matches(&id_b).count();
    let c_count = stdout.matches(&id_c).count();

    assert_eq!(a_count, 1, "Node A should appear exactly once in the tree");
    assert_eq!(b_count, 1, "Node B should appear exactly once in the tree");
    assert_eq!(c_count, 1, "Node C should appear exactly once in the tree");

    // No node should have "(seen)" because all nodes are reachable from root
    assert!(
        !stdout.contains("(seen)"),
        "No nodes should be marked as (seen)"
    );
}
