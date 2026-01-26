use crate::cli::support::{extract_id, qipu};
use tempfile::tempdir;

#[test]
fn test_custom_link_cost_reduces_hop_cost() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create custom config with cheap link type (cost 0.5)
    let config_path = dir.path().join(".qipu/config.toml");
    let config_content = r#"
version = 1
default_note_type = "fleeting"

[graph.types.cheap]
inverse = "expensive"
cost = 0.5
"#;
    std::fs::write(config_path, config_content).unwrap();

    // Create a chain: A -> B -> C -> D (all with "cheap" type, cost 0.5 each)
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

    let output_d = qipu()
        .current_dir(dir.path())
        .args(["create", "Node D"])
        .output()
        .unwrap();
    let id_d = extract_id(&output_d);

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_a, "100"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_b, "100"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_c, "100"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_d, "100"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_b, "--type", "cheap"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_b, &id_c, "--type", "cheap"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_c, &id_d, "--type", "cheap"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // With max_hops=1 (cost 1.0), we can reach D because cheap links cost 0.5 each
    // Path: A (0) -> B (0.5) -> C (1.0) -> D (1.5)
    // Wait, D should not be reachable because 3 * 0.5 = 1.5 > 1.0
    // Let me recalculate: A(0) -> B(0.5) -> C(1.0) means D is at cost 1.5 > 1.0
    // So only A, B, C should be reachable
    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id_a, "--max-hops", "1", "--format", "json"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    let note_ids: Vec<String> = json["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap().to_string())
        .collect();

    assert!(note_ids.contains(&id_a));
    assert!(note_ids.contains(&id_b));
    assert!(note_ids.contains(&id_c));
    assert!(
        !note_ids.contains(&id_d),
        "D should not be reachable with max_hops=1 and cheap links"
    );
}

#[test]
fn test_custom_link_cost_increases_hop_cost() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create custom config with expensive link type (cost 2.0)
    let config_path = dir.path().join(".qipu/config.toml");
    let config_content = r#"
version = 1
default_note_type = "fleeting"

[graph.types.expensive]
inverse = "cheap"
cost = 2.0
"#;
    std::fs::write(config_path, config_content).unwrap();

    // Create a chain: A -> B -> C
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
        .args(["value", "set", &id_a, "100"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_b, "100"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_c, "100"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_b, "--type", "expensive"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_b, &id_c, "--type", "expensive"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // With max_hops=2, we can only reach B (cost 2.0), not C (cost 4.0)
    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id_a, "--max-hops", "2", "--format", "json"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    let note_ids: Vec<String> = json["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap().to_string())
        .collect();

    assert!(note_ids.contains(&id_a));
    assert!(note_ids.contains(&id_b));
    assert!(
        !note_ids.contains(&id_c),
        "C should not be reachable with max_hops=2 and expensive links"
    );
}

#[test]
fn test_custom_link_cost_with_value_penalties() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create custom config with cheap link type (cost 0.5)
    let config_path = dir.path().join(".qipu/config.toml");
    let config_content = r#"
version = 1
default_note_type = "fleeting"

[graph.types.cheap]
inverse = "inverse-cheap"
cost = 0.5
"#;
    std::fs::write(config_path, config_content).unwrap();

    // Create A -> B -> C where B has low value (max penalty)
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
        .args(["create", "Node C"])
        .output()
        .unwrap();
    let id_c = extract_id(&output_c);

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

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_b, "--type", "cheap"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_b, &id_c, "--type", "cheap"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Edge costs:
    // A -> B: 0.5 (link cost) * (1 + (100-0)/100) = 0.5 * 2.0 = 1.0
    // B -> C: 0.5 (link cost) * (1 + (100-100)/100) = 0.5 * 1.0 = 0.5
    // Total to C: 1.0 + 0.5 = 1.5
    // With max_hops=1, only A and B should be reachable
    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id_a, "--max-hops", "1", "--format", "json"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    let note_ids: Vec<String> = json["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap().to_string())
        .collect();

    assert!(note_ids.contains(&id_a));
    assert!(note_ids.contains(&id_b));
    assert!(
        !note_ids.contains(&id_c),
        "C should not be reachable with max_hops=1 when B has low value"
    );

    // Check the hop count for B in spanning tree
    let spanning_tree = json["spanning_tree"].as_array().unwrap();
    let b_entry = spanning_tree
        .iter()
        .find(|e| e["to"].as_str().unwrap() == id_b)
        .unwrap();
    assert_eq!(b_entry["hop"].as_u64().unwrap(), 1);
}

#[test]
fn test_custom_link_cost_overrides_standard() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Override standard "part-of" link cost (normally 0.5) to 2.0
    let config_path = dir.path().join(".qipu/config.toml");
    let config_content = r#"
version = 1
default_note_type = "fleeting"

[graph.types."part-of"]
inverse = "has-part"
cost = 2.0
"#;
    std::fs::write(config_path, config_content).unwrap();

    // Create A -> B -> C using "part-of" (now expensive)
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
        .args(["value", "set", &id_a, "100"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_b, "100"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_c, "100"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_b, "--type", "part-of"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_b, &id_c, "--type", "part-of"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // With max_hops=2, only A and B should be reachable (cost 2.0 each)
    // Total to C would be 4.0 > 2.0
    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id_a, "--max-hops", "2", "--format", "json"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    let note_ids: Vec<String> = json["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap().to_string())
        .collect();

    assert!(note_ids.contains(&id_a));
    assert!(note_ids.contains(&id_b));
    assert!(
        !note_ids.contains(&id_c),
        "C should not be reachable with max_hops=2 when part-of cost is overridden to 2.0"
    );
}

#[test]
fn test_custom_link_cost_with_ignore_value() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create custom config with cheap link type (cost 0.5)
    let config_path = dir.path().join(".qipu/config.toml");
    let config_content = r#"
version = 1
default_note_type = "fleeting"

[graph.types.cheap]
inverse = "inverse-cheap"
cost = 0.5
"#;
    std::fs::write(config_path, config_content).unwrap();

    // Create A -> B -> C where B has low value
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
        .args(["create", "Node C"])
        .output()
        .unwrap();
    let id_c = extract_id(&output_c);

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

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_b, "--type", "cheap"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_b, &id_c, "--type", "cheap"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // With --ignore-value, edge costs are just link type costs (no value penalty)
    // A -> B: 0.5, B -> C: 0.5, total to C: 1.0
    // With max_hops=1, C should NOT be reachable (cost 1.0 > 1.0 limit?)
    // Actually max_hops=1 means cost limit of 1.0, and C costs exactly 1.0, so it should be at the boundary
    // Let's check: A(0) -> B(0.5) -> C(1.0)
    // With max_hops=1, only A and B should be reachable (C is at the limit)
    let output = qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "tree",
            &id_a,
            "--max-hops",
            "1",
            "--ignore-value",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    let note_ids: Vec<String> = json["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap().to_string())
        .collect();

    assert!(note_ids.contains(&id_a));
    assert!(note_ids.contains(&id_b));
    assert!(
        note_ids.contains(&id_c),
        "C should be reachable with max_hops=1 (cost equals limit)"
    );

    // Verify hop counts in spanning tree
    let spanning_tree = json["spanning_tree"].as_array().unwrap();
    assert_eq!(spanning_tree.len(), 2);
    let b_entry = spanning_tree
        .iter()
        .find(|e| e["to"].as_str().unwrap() == id_b)
        .unwrap();
    let c_entry = spanning_tree
        .iter()
        .find(|e| e["to"].as_str().unwrap() == id_c)
        .unwrap();
    assert_eq!(b_entry["hop"].as_u64().unwrap(), 0);
    assert_eq!(c_entry["hop"].as_u64().unwrap(), 1);
}

#[test]
fn test_custom_link_cost_affects_pathfinding() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create custom config with cheap and expensive link types
    let config_path = dir.path().join(".qipu/config.toml");
    let config_content = r#"
version = 1
default_note_type = "fleeting"

[graph.types.cheap]
inverse = "inverse-cheap"
cost = 0.3

[graph.types.expensive]
inverse = "inverse-expensive"
cost = 2.0
"#;
    std::fs::write(config_path, config_content).unwrap();

    // Create diamond: A -> B -> D (cheap), A -> C -> D (expensive)
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

    let output_d = qipu()
        .current_dir(dir.path())
        .args(["create", "Node D"])
        .output()
        .unwrap();
    let id_d = extract_id(&output_d);

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_a, "100"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_b, "100"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_c, "100"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_d, "100"])
        .assert()
        .success();

    // Create cheap path: A -> B -> D
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_b, "--type", "cheap"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_b, &id_d, "--type", "cheap"])
        .assert()
        .success();

    // Create expensive path: A -> C -> D
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_c, "--type", "expensive"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_c, &id_d, "--type", "expensive"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // With max_hops=1, all nodes should be reachable (discovered before max_hops expansion check)
    // Cheap path cost: 0.3 + 0.3 = 0.6
    // Expensive path cost: 2.0 + 2.0 = 4.0
    // C is reachable via expensive link (cost 2.0) even though it exceeds max_hops
    // because max_hops only prevents expansion, not inclusion
    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id_a, "--max-hops", "1", "--format", "json"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    let note_ids: Vec<String> = json["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap().to_string())
        .collect();

    assert!(note_ids.contains(&id_a));
    assert!(note_ids.contains(&id_b));
    assert!(
        note_ids.contains(&id_c),
        "C should be reachable even though cost 2.0 > max_hops (included before expansion check)"
    );
    assert!(
        note_ids.contains(&id_d),
        "D should be reachable via cheap path with cost 0.6"
    );
}
