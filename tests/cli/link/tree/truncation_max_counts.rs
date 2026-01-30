use crate::support::{extract_id, qipu, setup_test_dir};
use tempfile::tempdir;

#[test]
fn test_link_tree_max_nodes_truncation() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Root"])
        .output()
        .unwrap();
    let root = extract_id(&output);

    let mut child_ids = Vec::new();
    for label in ["A", "B", "C"] {
        let output = qipu()
            .current_dir(dir.path())
            .args(["create", label])
            .output()
            .unwrap();
        let child_id = extract_id(&output);
        child_ids.push(child_id.clone());

        qipu()
            .current_dir(dir.path())
            .args(["link", "add", &root, &child_id, "--type", "related"])
            .assert()
            .success();

        let output = qipu()
            .current_dir(dir.path())
            .args(["create", &format!("{} Child", label)])
            .output()
            .unwrap();
        let grandchild_id = extract_id(&output);

        qipu()
            .current_dir(dir.path())
            .args([
                "link",
                "add",
                &child_id,
                &grandchild_id,
                "--type",
                "related",
            ])
            .assert()
            .success();
    }

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "json",
            "link",
            "tree",
            &root,
            "--max-nodes",
            "3",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["truncated"], true);
    assert_eq!(json["truncation_reason"], "max_nodes");

    let note_count = json["notes"].as_array().unwrap().len();
    assert_eq!(note_count, 3);
}

#[test]
fn test_link_tree_max_edges_truncation() {
    let dir = setup_test_dir();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Root"])
        .output()
        .unwrap();
    let root = extract_id(&output);

    let mut child_ids = Vec::new();
    for label in ["A", "B", "C", "D"] {
        let output = qipu()
            .current_dir(dir.path())
            .args(["create", label])
            .output()
            .unwrap();
        let child_id = extract_id(&output);
        child_ids.push(child_id.clone());

        qipu()
            .current_dir(dir.path())
            .args(["link", "add", &root, &child_id, "--type", "related"])
            .assert()
            .success();
    }

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "json",
            "link",
            "tree",
            &root,
            "--max-edges",
            "2",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["truncated"], true);
    assert_eq!(json["truncation_reason"], "max_edges");

    let edge_count = json["links"].as_array().unwrap().len();
    assert_eq!(edge_count, 2);
}
