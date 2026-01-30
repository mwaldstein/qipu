use crate::support::{extract_id, qipu, setup_test_dir};
use tempfile::tempdir;

#[test]
fn test_link_tree_cycle_shows_seen() {
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
        .args(["link", "add", &id_c, &id_a, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "tree", &id_a])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Node A"));
    assert!(stdout.contains("Node B"));
    assert!(stdout.contains("Node C"));

    // All three nodes should appear exactly once (no duplicates)
    let a_count = stdout.matches(&id_a).count();
    let b_count = stdout.matches(&id_b).count();
    let c_count = stdout.matches(&id_c).count();
    assert_eq!(a_count, 1, "Node A should appear exactly once");
    assert_eq!(b_count, 1, "Node B should appear exactly once");
    assert_eq!(c_count, 1, "Node C should appear exactly once");

    // No "(seen)" because spanning tree ensures each node appears once
    assert!(
        !stdout.contains("(seen)"),
        "No nodes should be marked as (seen) in spanning tree"
    );
}
