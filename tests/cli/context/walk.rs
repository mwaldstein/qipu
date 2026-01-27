use crate::cli::support::{extract_id, qipu};
use predicates::prelude::*;
use tempfile::tempdir;

// ============================================================================
// Context walk tests
// ============================================================================

#[test]
fn test_context_walk_basic() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create root note
    let root_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Root Note"])
        .output()
        .unwrap();
    let root_id = extract_id(&root_output);

    // Create child note
    let child_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Child Note"])
        .output()
        .unwrap();
    let child_id = extract_id(&child_output);

    // Create grandchild note
    let grandchild_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Grandchild Note"])
        .output()
        .unwrap();
    let grandchild_id = extract_id(&grandchild_output);

    // Add links: root -> child -> grandchild
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &root_id, &child_id, "--type", "supports"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "add",
            &child_id,
            &grandchild_id,
            "--type",
            "supports",
        ])
        .assert()
        .success();

    // Walk from root with 2 hops should include all three notes
    qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--walk",
            &root_id,
            "--walk-max-hops",
            "2",
            "--related",
            "0",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Root Note"))
        .stdout(predicate::str::contains("Child Note"))
        .stdout(predicate::str::contains("Grandchild Note"));
}

#[test]
fn test_context_walk_max_hops() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create root note
    let root_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Root Note"])
        .output()
        .unwrap();
    let root_id = extract_id(&root_output);

    // Create child note
    let child_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Child Note"])
        .output()
        .unwrap();
    let child_id = extract_id(&child_output);

    // Create grandchild note
    let grandchild_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Grandchild Note"])
        .output()
        .unwrap();
    let grandchild_id = extract_id(&grandchild_output);

    // Add links: root -> child -> grandchild
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &root_id, &child_id, "--type", "supports"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "add",
            &child_id,
            &grandchild_id,
            "--type",
            "supports",
        ])
        .assert()
        .success();

    // Walk from root with 1 hop should only include root and child
    qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--walk",
            &root_id,
            "--walk-max-hops",
            "1",
            "--related",
            "0",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Root Note"))
        .stdout(predicate::str::contains("Child Note"))
        .stdout(predicate::str::contains("Grandchild Note").not());
}

#[test]
fn test_context_walk_direction() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create root note
    let root_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Root Note"])
        .output()
        .unwrap();
    let root_id = extract_id(&root_output);

    // Create child note
    let child_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Child Note"])
        .output()
        .unwrap();
    let child_id = extract_id(&child_output);

    // Add link: root -> child
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &root_id, &child_id, "--type", "supports"])
        .assert()
        .success();

    // Walk from root with direction=out should include child
    qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--walk",
            &root_id,
            "--walk-direction",
            "out",
            "--related",
            "0",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Root Note"))
        .stdout(predicate::str::contains("Child Note"));

    // Walk from child with direction=in should include root (via backlink)
    qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--walk",
            &child_id,
            "--walk-direction",
            "in",
            "--related",
            "0",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Child Note"))
        .stdout(predicate::str::contains("Root Note"));
}

#[test]
fn test_context_walk_with_type_filter() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create root note
    let root_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Root Note"])
        .output()
        .unwrap();
    let root_id = extract_id(&root_output);

    // Create child notes with different link types
    let child1_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Child Note 1"])
        .output()
        .unwrap();
    let child1_id = extract_id(&child1_output);

    let child2_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Child Note 2"])
        .output()
        .unwrap();
    let child2_id = extract_id(&child2_output);

    // Add links with different types
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &root_id, &child1_id, "--type", "supports"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "add",
            &root_id,
            &child2_id,
            "--type",
            "derived-from",
        ])
        .assert()
        .success();

    // Walk with type filter should only include notes with matching link type
    qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--walk",
            &root_id,
            "--walk-type",
            "supports",
            "--related",
            "0",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Root Note"))
        .stdout(predicate::str::contains("Child Note 1"))
        .stdout(predicate::str::contains("Child Note 2").not());
}

#[test]
fn test_context_walk_json_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create root note
    let root_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Root Note"])
        .output()
        .unwrap();
    let root_id = extract_id(&root_output);

    // Create child note
    let child_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Child Note"])
        .output()
        .unwrap();
    let child_id = extract_id(&child_output);

    // Add link
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &root_id, &child_id, "--type", "supports"])
        .assert()
        .success();

    // Walk should produce valid JSON output
    qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--walk",
            &root_id,
            "--format",
            "json",
            "--related",
            "0",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""notes":"#))
        .stdout(predicate::str::contains(r#""id":"#));
}

#[test]
fn test_context_walk_semantic_inversion_default() {
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

#[test]
fn test_context_walk_min_value_filter_all_match() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Root Note"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "High Value Note"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id1, "90"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id2, "95"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
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
            "context",
            "--walk",
            &id1,
            "--walk-min-value",
            "80",
            "--related",
            "0",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Root Note"))
        .stdout(predicate::str::contains("High Value Note"));
}

#[test]
fn test_context_walk_min_value_filter_some_match() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Root Note"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Low Value Note"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    let output3 = qipu()
        .current_dir(dir.path())
        .args(["create", "High Value Note"])
        .output()
        .unwrap();
    let id3 = extract_id(&output3);

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id1, "90"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id2, "30"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id3, "95"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id3, "--type", "related"])
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
            "context",
            "--walk",
            &id1,
            "--walk-min-value",
            "80",
            "--related",
            "0",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Root Note"))
        .stdout(predicate::str::contains("High Value Note"))
        .stdout(predicate::str::contains("Low Value Note").not());
}

#[test]
fn test_context_walk_min_value_filter_with_defaults() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Root Note"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Default Value Note"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id1, "90"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
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
            "context",
            "--walk",
            &id1,
            "--walk-min-value",
            "50",
            "--related",
            "0",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Root Note"))
        .stdout(predicate::str::contains("Default Value Note"));
}

#[test]
fn test_context_walk_min_value_filter_excludes_root() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Low Value Root"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "High Value Child"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id1, "20"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id2, "90"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
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
            "context",
            "--walk",
            &id1,
            "--walk-min-value",
            "80",
            "--related",
            "0",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Low Value Root").not())
        .stdout(predicate::str::contains("High Value Child").not());
}

#[test]
fn test_context_walk_ignore_value_unweighted() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Root Note"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Low Value Note"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    let output3 = qipu()
        .current_dir(dir.path())
        .args(["create", "High Value Note"])
        .output()
        .unwrap();
    let id3 = extract_id(&output3);

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id1, "50"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id2, "0"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id3, "100"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id3, "--type", "related"])
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
            "context",
            "--walk",
            &id1,
            "--walk-ignore-value",
            "--related",
            "0",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Root Note"))
        .stdout(predicate::str::contains("Low Value Note"))
        .stdout(predicate::str::contains("High Value Note"));
}
