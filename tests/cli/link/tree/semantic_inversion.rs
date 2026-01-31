//! Tests for link command
use crate::support::{extract_id, qipu, setup_test_dir};
use predicates::prelude::*;

#[test]
fn test_link_tree_semantic_inversion_default() {
    let dir = setup_test_dir();

    let output_a = qipu()
        .current_dir(dir.path())
        .args(["create", "Note A"])
        .output()
        .unwrap();
    let id_a = extract_id(&output_a);

    let output_b = qipu()
        .current_dir(dir.path())
        .args(["create", "Note B"])
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
            "tree",
            &id_b,
            "--direction",
            "in",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    let links = json["links"].as_array().unwrap();
    let inverted_link = links
        .iter()
        .find(|l| l["from"].as_str() == Some(&id_b) && l["to"].as_str() == Some(&id_a));

    assert!(inverted_link.is_some(), "Should have virtual inverted link");
    let link = inverted_link.unwrap();
    assert_eq!(link["type"].as_str().unwrap(), "supported-by");
    assert_eq!(link["source"].as_str().unwrap(), "virtual");
}

#[test]
fn test_link_tree_semantic_inversion_disabled() {
    let dir = setup_test_dir();

    let output_a = qipu()
        .current_dir(dir.path())
        .args(["create", "Note A"])
        .output()
        .unwrap();
    let id_a = extract_id(&output_a);

    let output_b = qipu()
        .current_dir(dir.path())
        .args(["create", "Note B"])
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
            "tree",
            &id_b,
            "--direction",
            "in",
            "--no-semantic-inversion",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    let links = json["links"].as_array().unwrap();
    let raw_link = links
        .iter()
        .find(|l| l["from"].as_str() == Some(&id_a) && l["to"].as_str() == Some(&id_b));

    assert!(raw_link.is_some(), "Should have raw backlink");
    let link = raw_link.unwrap();
    assert_eq!(link["type"].as_str().unwrap(), "supports");
    assert_eq!(link["source"].as_str().unwrap(), "typed");
}

#[test]
fn test_link_tree_semantic_inversion_type_filter() {
    let dir = setup_test_dir();

    let output_a = qipu()
        .current_dir(dir.path())
        .args(["create", "Note A"])
        .output()
        .unwrap();
    let id_a = extract_id(&output_a);

    let output_b = qipu()
        .current_dir(dir.path())
        .args(["create", "Note B"])
        .output()
        .unwrap();
    let id_b = extract_id(&output_b);

    let output_c = qipu()
        .current_dir(dir.path())
        .args(["create", "Note C"])
        .output()
        .unwrap();
    let id_c = extract_id(&output_c);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_b, "--type", "supports"])
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

    qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "tree",
            &id_b,
            "--direction",
            "in",
            "--type",
            "supported-by",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Note A"))
        .stdout(predicate::str::contains("Note C").not());

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "tree",
            &id_b,
            "--direction",
            "in",
            "--type",
            "supports",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("Note A"),
        "Should not find Note A when filtering by original type with semantic inversion"
    );

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "tree",
            &id_b,
            "--direction",
            "in",
            "--no-semantic-inversion",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let note_ids: Vec<&str> = json["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap())
        .collect();
    assert!(note_ids.contains(&id_a.as_str()), "Should find Note A");
    assert!(!note_ids.contains(&id_c.as_str()), "Should not find Note C");

    let links = json["links"].as_array().unwrap();
    let link = links
        .iter()
        .find(|l| l["from"].as_str() == Some(&id_a) && l["to"].as_str() == Some(&id_b));
    assert!(link.is_some(), "Should have raw link A -> B");
    assert_eq!(link.unwrap()["type"].as_str().unwrap(), "supports");

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "tree",
            &id_b,
            "--direction",
            "in",
            "--no-semantic-inversion",
            "--type",
            "supports",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let note_ids: Vec<&str> = json["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap())
        .collect();
    assert!(
        note_ids.contains(&id_a.as_str()),
        "Should find Note A when filtering by original type"
    );

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "tree",
            &id_b,
            "--direction",
            "in",
            "--no-semantic-inversion",
            "--type",
            "supported-by",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let note_ids: Vec<&str> = json["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap())
        .collect();
    assert!(
        !note_ids.contains(&id_a.as_str()),
        "Should not find Note A when filtering by inverted type without semantic inversion"
    );
}
