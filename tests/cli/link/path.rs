use crate::cli::support::{extract_id, qipu};
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_link_path_direct() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Start"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "End"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

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

    // Find path from start to end
    qipu()
        .current_dir(dir.path())
        .args(["link", "path", &id1, &id2])
        .assert()
        .success()
        .stdout(predicate::str::contains("Start"))
        .stdout(predicate::str::contains("End"))
        .stdout(predicate::str::contains("Path length: 1 hop"));
}

#[test]
fn test_link_path_multi_hop() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create A -> B -> C
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

    // Find path from A to C (2 hops)
    qipu()
        .current_dir(dir.path())
        .args(["link", "path", &id_a, &id_c])
        .assert()
        .success()
        .stdout(predicate::str::contains("Node A"))
        .stdout(predicate::str::contains("Node B"))
        .stdout(predicate::str::contains("Node C"))
        .stdout(predicate::str::contains("Path length: 2 hop"));
}

#[test]
fn test_link_path_not_found() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two unconnected notes
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Isolated 1"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Isolated 2"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Path should not be found
    qipu()
        .current_dir(dir.path())
        .args(["link", "path", &id1, &id2])
        .assert()
        .success()
        .stdout(predicate::str::contains("No path found"));
}

#[test]
fn test_link_path_json_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "JSON Start"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "JSON End"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

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
        .args(["--format", "json", "link", "path", &id1, &id2])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"from\":"))
        .stdout(predicate::str::contains("\"to\":"))
        .stdout(predicate::str::contains("\"found\": true"))
        .stdout(predicate::str::contains("\"notes\":"))
        .stdout(predicate::str::contains("\"links\":"));
}

#[test]
fn test_link_path_records_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Records Start"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Records End"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

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
        .args(["--format", "records", "link", "path", &id1, &id2])
        .assert()
        .success()
        .stdout(predicate::str::contains("H qipu=1 records=1 store="))
        .stdout(predicate::str::contains("mode=link.path"))
        .stdout(predicate::str::contains("found=true"));
}

#[test]
fn test_link_path_type_filter() {
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
        .args(["link", "path", &id_a, &id_c, "--type", "supports"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No path found"));

    qipu()
        .current_dir(dir.path())
        .args(["link", "path", &id_a, &id_b, "--type", "supports"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Node A"))
        .stdout(predicate::str::contains("Node B"))
        .stdout(predicate::str::contains("Path length: 1 hop"));
}

#[test]
fn test_link_path_exclude_type_filter() {
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
        .args(["link", "path", &id_a, &id_c, "--exclude-type", "related"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No path found"));
}

#[test]
fn test_link_path_typed_only() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output_child1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Typed Child"])
        .output()
        .unwrap();
    let id_child1 = extract_id(&output_child1);

    let output_child2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Inline Child"])
        .output()
        .unwrap();
    let id_child2 = extract_id(&output_child2);

    let output_root = qipu()
        .current_dir(dir.path())
        .args(["capture", "--title", "Root"])
        .write_stdin(format!("See [[{}]]", id_child2))
        .output()
        .unwrap();
    let id_root = extract_id(&output_root);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_root, &id_child1, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "path", &id_root, &id_child2, "--typed-only"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No path found"));

    qipu()
        .current_dir(dir.path())
        .args(["link", "path", &id_root, &id_child1, "--typed-only"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Root"))
        .stdout(predicate::str::contains("Typed Child"));
}

#[test]
fn test_link_path_inline_only() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output_child1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Typed Child"])
        .output()
        .unwrap();
    let id_child1 = extract_id(&output_child1);

    let output_child2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Inline Child"])
        .output()
        .unwrap();
    let id_child2 = extract_id(&output_child2);

    let output_root = qipu()
        .current_dir(dir.path())
        .args(["capture", "--title", "Root"])
        .write_stdin(format!("See [[{}]]", id_child2))
        .output()
        .unwrap();
    let id_root = extract_id(&output_root);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_root, &id_child1, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "path", &id_root, &id_child1, "--inline-only"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No path found"));

    qipu()
        .current_dir(dir.path())
        .args(["link", "path", &id_root, &id_child2, "--inline-only"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Root"))
        .stdout(predicate::str::contains("Inline Child"));
}

#[test]
fn test_link_path_direction_in() {
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

    qipu()
        .current_dir(dir.path())
        .args(["link", "path", &id_c, &id_a, "--direction", "in"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Node C"))
        .stdout(predicate::str::contains("Node B"))
        .stdout(predicate::str::contains("Node A"))
        .stdout(predicate::str::contains("Path length: 2 hop"));
}

#[test]
fn test_link_path_direction_both() {
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
        .args(["link", "add", &id_a, &id_b, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "path", &id_a, &id_b, "--direction", "both"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Node A"))
        .stdout(predicate::str::contains("Node B"))
        .stdout(predicate::str::contains("Path length: 1 hop"));

    qipu()
        .current_dir(dir.path())
        .args(["link", "path", &id_b, &id_a, "--direction", "both"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Node B"))
        .stdout(predicate::str::contains("Node A"))
        .stdout(predicate::str::contains("Path length: 1 hop"));
}

#[test]
fn test_link_path_min_value_filter_excludes_from_note() {
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
        .args(["link", "add", &id_a, &id_b, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Set Node A to low value
    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_a, "10"])
        .assert()
        .success();

    // Path should not be found because from note has low value
    qipu()
        .current_dir(dir.path())
        .args(["link", "path", &id_a, &id_b, "--min-value", "50"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No path found"));
}

#[test]
fn test_link_path_min_value_filter_excludes_to_note() {
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
        .args(["link", "add", &id_a, &id_b, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Set Node B to low value
    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_b, "10"])
        .assert()
        .success();

    // Path should not be found because to note has low value
    qipu()
        .current_dir(dir.path())
        .args(["link", "path", &id_a, &id_b, "--min-value", "50"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No path found"));
}

#[test]
fn test_link_path_min_value_filter_excludes_intermediate_note() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create A -> B -> C
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

    // Set Node B to low value (intermediate note)
    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_b, "10"])
        .assert()
        .success();

    // Path should not be found because intermediate note has low value
    qipu()
        .current_dir(dir.path())
        .args(["link", "path", &id_a, &id_c, "--min-value", "50"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No path found"));
}

#[test]
fn test_link_path_min_value_filter_all_match() {
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
        .args(["link", "add", &id_a, &id_b, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Set both nodes to high value
    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_a, "80"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id_b, "90"])
        .assert()
        .success();

    // Path should be found because both nodes pass the threshold
    qipu()
        .current_dir(dir.path())
        .args(["link", "path", &id_a, &id_b, "--min-value", "50"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Node A"))
        .stdout(predicate::str::contains("Node B"))
        .stdout(predicate::str::contains("Path length: 1 hop"));
}

#[test]
fn test_link_path_min_value_filter_with_defaults() {
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
        .args(["link", "add", &id_a, &id_b, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // Notes with default value (None, treated as 50) should pass threshold of 50
    qipu()
        .current_dir(dir.path())
        .args(["link", "path", &id_a, &id_b, "--min-value", "50"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Node A"))
        .stdout(predicate::str::contains("Node B"))
        .stdout(predicate::str::contains("Path length: 1 hop"));
}
