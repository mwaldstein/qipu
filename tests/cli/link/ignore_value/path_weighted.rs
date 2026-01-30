use crate::support::{extract_id, qipu, setup_test_dir};

#[test]
fn test_link_path_weighted_by_default() {
    let dir = setup_test_dir();

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

    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "path", &id_a, &id_c, "--format", "json"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["found"], true);
    assert_eq!(json["path_length"].as_u64().unwrap(), 2);

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
