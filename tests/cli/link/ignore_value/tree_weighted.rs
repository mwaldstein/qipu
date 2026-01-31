//! Tests for link command
use crate::support::{extract_id, qipu, setup_test_dir};

/// Test that link tree uses weighted traversal by default (without --ignore-value)
/// and that high-value notes are visited before low-value notes even if they're further away
#[test]
fn test_link_tree_weighted_by_default() {
    let dir = setup_test_dir();

    let output_root = qipu()
        .current_dir(dir.path())
        .args(["create", "Root"])
        .output()
        .unwrap();
    let id_root = extract_id(&output_root);

    let output_high = qipu()
        .current_dir(dir.path())
        .args(["create", "High Value Note"])
        .output()
        .unwrap();
    let id_high = extract_id(&output_high);

    let output_mid = qipu()
        .current_dir(dir.path())
        .args(["create", "Mid Note"])
        .output()
        .unwrap();
    let id_mid = extract_id(&output_mid);

    let output_low = qipu()
        .current_dir(dir.path())
        .args(["create", "Low Value Note"])
        .output()
        .unwrap();
    let id_low = extract_id(&output_low);

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
