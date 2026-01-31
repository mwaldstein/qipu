//! Tests for context command advanced semantic walk
use crate::support::setup_test_dir;
use crate::support::{extract_id, qipu};

#[test]
fn test_context_walk_semantic_inversion_default() {
    let dir = setup_test_dir();

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

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--walk",
            &id_b,
            "--format",
            "json",
            "--related",
            "0",
        ])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let notes = json["notes"].as_array().unwrap();

    assert!(notes.len() >= 2, "Should have at least two notes");

    let note_ids: Vec<&str> = notes.iter().filter_map(|n| n["id"].as_str()).collect();

    assert!(
        note_ids.contains(&id_a.as_str()),
        "Should include source note"
    );
    assert!(
        note_ids.contains(&id_b.as_str()),
        "Should include target note"
    );
}

#[test]
fn test_context_walk_semantic_inversion_disabled() {
    let dir = setup_test_dir();

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

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--walk",
            &id_b,
            "--format",
            "json",
            "--no-semantic-inversion",
            "--related",
            "0",
        ])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let notes = json["notes"].as_array().unwrap();

    assert!(notes.len() >= 2, "Should have at least two notes");

    let note_ids: Vec<&str> = notes.iter().filter_map(|n| n["id"].as_str()).collect();

    assert!(
        note_ids.contains(&id_a.as_str()),
        "Should include source note"
    );
    assert!(
        note_ids.contains(&id_b.as_str()),
        "Should include target note"
    );
}
