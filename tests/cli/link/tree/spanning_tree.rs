use crate::support::{extract_id, qipu};
use tempfile::tempdir;

#[test]
fn test_link_tree_spanning_tree_ordering() {
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

    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id_root, "--format", "json"])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    let spanning_tree = json["spanning_tree"].as_array().unwrap();
    assert_eq!(spanning_tree.len(), 3);

    let types: Vec<&str> = spanning_tree
        .iter()
        .map(|e| e["type"].as_str().unwrap())
        .collect();

    assert_eq!(types[0], "derived-from");
    assert_eq!(types[1], "related");
    assert_eq!(types[2], "supports");

    assert_eq!(spanning_tree[0]["to"].as_str().unwrap(), id_b);
    assert_eq!(spanning_tree[1]["to"].as_str().unwrap(), id_a);
    assert_eq!(spanning_tree[2]["to"].as_str().unwrap(), id_c);
}
