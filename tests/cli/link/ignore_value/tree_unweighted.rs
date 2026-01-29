use crate::cli::support::{extract_id, qipu};
use tempfile::tempdir;

#[test]
fn test_link_tree_ignore_value_unweighted() {
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

    let note_ids: Vec<String> = json["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap().to_string())
        .collect();

    assert!(note_ids.contains(&id_root));
    assert!(note_ids.contains(&id_child1));
    assert!(note_ids.contains(&id_child2));

    let spanning_tree = json["spanning_tree"].as_array().unwrap();
    assert_eq!(spanning_tree.len(), 2);

    for entry in spanning_tree {
        assert_eq!(entry["hop"].as_u64().unwrap(), 1);
    }
}

#[test]
fn test_unweighted_alias_tree() {
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

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_b, "0"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_b, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output_unweighted = qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id_a, "--unweighted", "--format", "json"])
        .output()
        .unwrap();

    let json_unweighted: serde_json::Value =
        serde_json::from_slice(&output_unweighted.stdout).unwrap();

    let spanning = json_unweighted["spanning_tree"].as_array().unwrap();
    let hop_b = spanning
        .iter()
        .find(|e| e["to"].as_str().unwrap() == id_b)
        .unwrap()["hop"]
        .as_u64()
        .unwrap();

    assert_eq!(hop_b, 1);

    let output_weighted = qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id_a, "--format", "json"])
        .output()
        .unwrap();

    let json_weighted: serde_json::Value = serde_json::from_slice(&output_weighted.stdout).unwrap();

    let spanning_weighted = json_weighted["spanning_tree"].as_array().unwrap();
    let hop_b_weighted = spanning_weighted
        .iter()
        .find(|e| e["to"].as_str().unwrap() == id_b)
        .unwrap()["hop"]
        .as_u64()
        .unwrap();

    assert_eq!(hop_b_weighted, 2);
}
