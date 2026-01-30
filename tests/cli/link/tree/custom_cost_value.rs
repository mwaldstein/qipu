use crate::support::{extract_id, qipu};
use tempfile::tempdir;

#[test]
fn test_custom_link_cost_with_value_penalties() {
    let dir = setup_test_dir();

    let config_path = dir.path().join(".qipu/config.toml");
    let config_content = r#"
version = 1
default_note_type = "fleeting"

[graph.types.cheap]
inverse = "inverse-cheap"
cost = 0.5
"#;
    std::fs::write(config_path, config_content).unwrap();

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

    let spanning_tree = json["spanning_tree"].as_array().unwrap();
    let b_entry = spanning_tree
        .iter()
        .find(|e| e["to"].as_str().unwrap() == id_b)
        .unwrap();
    assert_eq!(b_entry["hop"].as_u64().unwrap(), 1);
}

#[test]
fn test_custom_link_cost_with_ignore_value() {
    let dir = setup_test_dir();

    let config_path = dir.path().join(".qipu/config.toml");
    let config_content = r#"
version = 1
default_note_type = "fleeting"

[graph.types.cheap]
inverse = "inverse-cheap"
cost = 0.5
"#;
    std::fs::write(config_path, config_content).unwrap();

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
