//! Tests for link command
use crate::support::{extract_id, qipu, setup_test_dir};

#[test]
fn test_custom_link_cost_overrides_standard() {
    let dir = setup_test_dir();

    let config_path = dir.path().join(".qipu/config.toml");
    let config_content = r#"
version = 1
default_note_type = "fleeting"

[graph.types."part-of"]
inverse = "has-part"
cost = 2.0
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
fn test_custom_link_cost_affects_pathfinding() {
    let dir = setup_test_dir();

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
        .args(["link", "add", &id_b, &id_d, "--type", "cheap"])
        .assert()
        .success();

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
