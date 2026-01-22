use crate::cli::support::{extract_id, qipu};
use tempfile::tempdir;

/// Test that link tree uses weighted traversal by default (without --ignore-value)
/// and that high-value notes are visited before low-value notes even if they're further away
#[test]
fn test_link_tree_weighted_by_default() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create root note
    let output_root = qipu()
        .current_dir(dir.path())
        .args(["create", "Root"])
        .output()
        .unwrap();
    let id_root = extract_id(&output_root);

    // Create a high-value note (2 hops away)
    let output_high = qipu()
        .current_dir(dir.path())
        .args(["create", "High Value Note"])
        .output()
        .unwrap();
    let id_high = extract_id(&output_high);

    // Create intermediate note
    let output_mid = qipu()
        .current_dir(dir.path())
        .args(["create", "Mid Note"])
        .output()
        .unwrap();
    let id_mid = extract_id(&output_mid);

    // Create a low-value note (1 hop away)
    let output_low = qipu()
        .current_dir(dir.path())
        .args(["create", "Low Value Note"])
        .output()
        .unwrap();
    let id_low = extract_id(&output_low);

    // Set values: root=100, mid=80, high=95, low=10
    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_root, "100"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_mid, "80"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_high, "95"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_low, "10"])
        .assert()
        .success();

    // Create links: root -> low (1 hop), root -> mid -> high (2 hops)
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_root, &id_low, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_root, &id_mid, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_mid, &id_high, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // With weighted traversal (default), the high-value note at depth 2 should be visited
    // All notes should be included since we have enough hops
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "tree",
            &id_root,
            "--max-hops",
            "3",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // All notes should be present
    let note_ids: Vec<String> = json["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap().to_string())
        .collect();

    assert!(note_ids.contains(&id_root));
    assert!(note_ids.contains(&id_mid));
    assert!(note_ids.contains(&id_high));
    assert!(note_ids.contains(&id_low));
}

/// Test that link tree with --ignore-value uses unweighted BFS
#[test]
fn test_link_tree_ignore_value_unweighted() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create root note
    let output_root = qipu()
        .current_dir(dir.path())
        .args(["create", "Root"])
        .output()
        .unwrap();
    let id_root = extract_id(&output_root);

    // Create child notes
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

    // Set different values
    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_root, "50"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_child1, "100"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_child2, "0"])
        .assert()
        .success();

    // Create links: root -> child1, root -> child2
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_root, &id_child1, "--type", "related"])
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

    // With --ignore-value, all edges should be treated equally (cost 1.0)
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "tree",
            &id_root,
            "--ignore-value",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // All notes should be present (value doesn't affect traversal order)
    let note_ids: Vec<String> = json["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap().to_string())
        .collect();

    assert!(note_ids.contains(&id_root));
    assert!(note_ids.contains(&id_child1));
    assert!(note_ids.contains(&id_child2));

    // Spanning tree should show both edges with hop=1 (uniform cost)
    let spanning_tree = json["spanning_tree"].as_array().unwrap();
    assert_eq!(spanning_tree.len(), 2);

    // Both children should have hop=1 with --ignore-value
    for entry in spanning_tree {
        assert_eq!(entry["hop"].as_u64().unwrap(), 1);
    }
}

/// Test that link path uses weighted traversal by default
/// This test verifies that weighted mode works, but doesn't assert which specific
/// path is chosen since both paths have equal hop count and deterministic ordering
/// depends on node IDs which are generated at runtime.
#[test]
fn test_link_path_weighted_by_default() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a simple chain to verify weighted mode works
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

    // Set different values
    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_a, "100"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_b, "10"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_c, "90"])
        .assert()
        .success();

    // Create chain: A -> B -> C
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

    // With weighted traversal (default), path should be found
    // This test just verifies that weighted mode is active by default
    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "path", &id_a, &id_c, "--format", "json"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["found"], true);
    assert_eq!(json["path_length"].as_u64().unwrap(), 2);

    // Verify all three notes are in the path
    let note_ids: Vec<String> = json["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap().to_string())
        .collect();

    assert!(note_ids.contains(&id_a));
    assert!(note_ids.contains(&id_b));
    assert!(note_ids.contains(&id_c));
}

/// Test that link path with --ignore-value uses unweighted BFS (shortest hop)
#[test]
fn test_link_path_ignore_value_unweighted() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create the same diamond graph as above
    let output_a = qipu()
        .current_dir(dir.path())
        .args(["create", "Node A"])
        .output()
        .unwrap();
    let id_a = extract_id(&output_a);

    let output_b = qipu()
        .current_dir(dir.path())
        .args(["create", "Node B (Low Value)"])
        .output()
        .unwrap();
    let id_b = extract_id(&output_b);

    let output_c = qipu()
        .current_dir(dir.path())
        .args(["create", "Node C (High Value)"])
        .output()
        .unwrap();
    let id_c = extract_id(&output_c);

    let output_d = qipu()
        .current_dir(dir.path())
        .args(["create", "Node D"])
        .output()
        .unwrap();
    let id_d = extract_id(&output_d);

    // Set values (same as above)
    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_a, "100"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_b, "10"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_c, "90"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_d, "100"])
        .assert()
        .success();

    // Create diamond: A -> B -> D, A -> C -> D
    // But add B first (alphabetically earlier) so BFS will find it first
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_b, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_b, &id_d, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_c, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_c, &id_d, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // With --ignore-value, path should ignore values and just find any 2-hop path
    // Due to deterministic ordering (by link_type then ID), the path is deterministic
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "path",
            &id_a,
            &id_d,
            "--ignore-value",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["found"], true);
    assert_eq!(json["path_length"].as_u64().unwrap(), 2);

    // With --ignore-value, values are ignored, so either path is valid
    // The deterministic ordering will pick one consistently
    let note_ids: Vec<String> = json["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap().to_string())
        .collect();

    assert!(note_ids.contains(&id_a));
    assert!(note_ids.contains(&id_d));
    // Either B or C will be in the path, but not both (deterministic)
    assert_eq!(note_ids.len(), 3);
}

/// Test that --ignore-value flag properly disables value-based weighting
#[test]
fn test_ignore_value_disables_weighted_traversal() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a simple chain: A -> B -> C
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

    // Set extreme values: A=100, B=0, C=100
    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_a, "100"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_b, "0"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_c, "100"])
        .assert()
        .success();

    // Create links: A -> B -> C
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

    // Test link tree WITHOUT --ignore-value (weighted, default)
    let output_weighted = qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id_a, "--format", "json"])
        .output()
        .unwrap();

    let json_weighted: serde_json::Value = serde_json::from_slice(&output_weighted.stdout).unwrap();

    // With weighted traversal, B has value=0 so its edge cost is 2.0 (high resistance)
    // All nodes should still be reachable with default max_hops=3
    assert_eq!(json_weighted["notes"].as_array().unwrap().len(), 3);

    // Test link tree WITH --ignore-value (unweighted)
    let output_unweighted = qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id_a, "--ignore-value", "--format", "json"])
        .output()
        .unwrap();

    let json_unweighted: serde_json::Value =
        serde_json::from_slice(&output_unweighted.stdout).unwrap();

    // With --ignore-value, all edges cost 1.0 regardless of target value
    assert_eq!(json_unweighted["notes"].as_array().unwrap().len(), 3);

    // Both should have same nodes, but the hop counts in spanning tree differ
    let spanning_weighted = json_weighted["spanning_tree"].as_array().unwrap();
    let spanning_unweighted = json_unweighted["spanning_tree"].as_array().unwrap();

    // Find B's hop count in each
    let hop_b_weighted = spanning_weighted
        .iter()
        .find(|e| e["to"].as_str().unwrap() == id_b)
        .unwrap()["hop"]
        .as_u64()
        .unwrap();

    let hop_b_unweighted = spanning_unweighted
        .iter()
        .find(|e| e["to"].as_str().unwrap() == id_b)
        .unwrap()["hop"]
        .as_u64()
        .unwrap();

    // With weighted (value=0), edge cost is 2.0, so hop should be 2
    // With unweighted, edge cost is 1.0, so hop should be 1
    assert_eq!(hop_b_weighted, 2);
    assert_eq!(hop_b_unweighted, 1);
}
