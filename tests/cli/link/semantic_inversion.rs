use crate::cli::support::{extract_id, qipu};
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_link_list_semantic_inversion_default() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output_a = qipu()
        .current_dir(dir.path())
        .args(["create", "Semantic Source"])
        .output()
        .unwrap();
    let id_a = extract_id(&output_a);

    let output_b = qipu()
        .current_dir(dir.path())
        .args(["create", "Semantic Target"])
        .output()
        .unwrap();
    let id_b = extract_id(&output_b);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_b, "--type", "supports"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id_b, "--format", "json"])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let links = json.as_array().unwrap();

    assert_eq!(links.len(), 1, "Should have one link");
    let link = &links[0];

    assert_eq!(link["direction"].as_str().unwrap(), "out");
    assert_eq!(link["id"].as_str().unwrap(), id_a);
    assert_eq!(link["type"].as_str().unwrap(), "supported-by");
    assert_eq!(link["source"].as_str().unwrap(), "virtual");
    assert_eq!(link["title"].as_str().unwrap(), "Semantic Source");
}

#[test]
fn test_link_list_semantic_inversion_disabled() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output_a = qipu()
        .current_dir(dir.path())
        .args(["create", "Semantic Disabled Source"])
        .output()
        .unwrap();
    let id_a = extract_id(&output_a);

    let output_b = qipu()
        .current_dir(dir.path())
        .args(["create", "Semantic Disabled Target"])
        .output()
        .unwrap();
    let id_b = extract_id(&output_b);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_b, "--type", "supports"])
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
            "list",
            &id_b,
            "--format",
            "json",
            "--no-semantic-inversion",
        ])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let links = json.as_array().unwrap();

    assert_eq!(links.len(), 1, "Should have one link");
    let link = &links[0];

    assert_eq!(link["direction"].as_str().unwrap(), "in");
    assert_eq!(link["id"].as_str().unwrap(), id_a);
    assert_eq!(link["type"].as_str().unwrap(), "supports");
    assert_eq!(link["source"].as_str().unwrap(), "typed");
    assert_eq!(link["title"].as_str().unwrap(), "Semantic Disabled Source");
}

#[test]
fn test_link_list_semantic_inversion_type_filter() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output_a = qipu()
        .current_dir(dir.path())
        .args(["create", "Type Filter Source"])
        .output()
        .unwrap();
    let id_a = extract_id(&output_a);

    let output_b = qipu()
        .current_dir(dir.path())
        .args(["create", "Type Filter Target"])
        .output()
        .unwrap();
    let id_b = extract_id(&output_b);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_b, "--type", "supports"])
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
            "list",
            &id_b,
            "--format",
            "json",
            "--type",
            "supported-by",
        ])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let links = json.as_array().unwrap();

    assert_eq!(
        links.len(),
        1,
        "Should find link when filtering by inverted type"
    );
    let link = &links[0];

    assert_eq!(link["type"].as_str().unwrap(), "supported-by");
    assert_eq!(link["source"].as_str().unwrap(), "virtual");
}

#[test]
fn test_link_list_semantic_inversion_type_filter_disabled() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output_a = qipu()
        .current_dir(dir.path())
        .args(["create", "Type Filter Disabled Source"])
        .output()
        .unwrap();
    let id_a = extract_id(&output_a);

    let output_b = qipu()
        .current_dir(dir.path())
        .args(["create", "Type Filter Disabled Target"])
        .output()
        .unwrap();
    let id_b = extract_id(&output_b);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_b, "--type", "supports"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "list",
            &id_b,
            "--format",
            "json",
            "--type",
            "supported-by",
            "--no-semantic-inversion",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("[]"));
}

#[test]
fn test_link_list_semantic_inversion_type_filter_original() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output_a = qipu()
        .current_dir(dir.path())
        .args(["create", "Original Type Filter Source"])
        .output()
        .unwrap();
    let id_a = extract_id(&output_a);

    let output_b = qipu()
        .current_dir(dir.path())
        .args(["create", "Original Type Filter Target"])
        .output()
        .unwrap();
    let id_b = extract_id(&output_b);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_b, "--type", "supports"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args([
            "link", "list", &id_b, "--format", "json", "--type", "supports",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("[]"));

    qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "list",
            &id_b,
            "--format",
            "json",
            "--type",
            "supports",
            "--no-semantic-inversion",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"direction\": \"in\""))
        .stdout(predicate::str::contains("\"type\": \"supports\""));
}
