use crate::support::{extract_id, qipu};
use tempfile::tempdir;

#[test]
fn test_ignore_value_disables_weighted_traversal() {
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

    let output_weighted = qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id_a, "--format", "json"])
        .output()
        .unwrap();

    let json_weighted: serde_json::Value = serde_json::from_slice(&output_weighted.stdout).unwrap();

    assert_eq!(json_weighted["notes"].as_array().unwrap().len(), 3);

    let output_unweighted = qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id_a, "--ignore-value", "--format", "json"])
        .output()
        .unwrap();

    let json_unweighted: serde_json::Value =
        serde_json::from_slice(&output_unweighted.stdout).unwrap();

    assert_eq!(json_unweighted["notes"].as_array().unwrap().len(), 3);

    let spanning_weighted = json_weighted["spanning_tree"].as_array().unwrap();
    let spanning_unweighted = json_unweighted["spanning_tree"].as_array().unwrap();

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

    assert_eq!(hop_b_weighted, 2);
    assert_eq!(hop_b_unweighted, 1);
}
