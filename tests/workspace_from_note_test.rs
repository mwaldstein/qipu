//! Tests for workspace create with --from-note option
//!
//! Tests verify that workspace creation from a note includes notes
//! within the specified hop distance (default: 3 hops).

#[path = "support/mod.rs"]
mod support;

use std::path::Path;
use tempfile::tempdir;

use support::{create_link_at_path, create_note_at_path, init_store_at_path, qipu};

struct TestGraph {
    root_id: String,
    child1_id: String,
    child2_id: String,
    grandchild_id: String,
    far_id: String,
}

fn setup_test_graph(root: &Path) -> TestGraph {
    init_store_at_path(root);

    let root_id = create_note_at_path(root, "Root Note");
    assert!(!root_id.is_empty(), "Root ID should not be empty");

    let child1_id = create_note_at_path(root, "Child 1");
    let child2_id = create_note_at_path(root, "Child 2");

    create_link_at_path(root, &root_id, &child1_id, "part-of");
    create_link_at_path(root, &root_id, &child2_id, "part-of");

    let grandchild_id = create_note_at_path(root, "Grandchild");
    create_link_at_path(root, &child1_id, &grandchild_id, "part-of");

    let far_id = create_note_at_path(root, "Far Away Note");
    create_link_at_path(root, &grandchild_id, &far_id, "related");

    TestGraph {
        root_id,
        child1_id,
        child2_id,
        grandchild_id,
        far_id,
    }
}

fn create_workspace_from_note(root: &Path, name: &str, note_id: &str) {
    qipu()
        .current_dir(root)
        .args(["workspace", "new", name, "--from-note", note_id])
        .assert()
        .success();
}

fn list_workspace_notes(root: &Path, workspace: &str) -> String {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_qipu"))
        .env("CARGO_MANIFEST_DIR", env!("CARGO_MANIFEST_DIR"))
        .current_dir(root)
        .args(["list", "--workspace", workspace])
        .output()
        .unwrap();
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn test_workspace_from_note_includes_root() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let graph = setup_test_graph(root);
    create_workspace_from_note(root, "slice_test", &graph.root_id);

    let workspace_notes = list_workspace_notes(root, "slice_test");
    assert!(
        workspace_notes.contains(&graph.root_id),
        "Root note should be in workspace"
    );
}

#[test]
fn test_workspace_from_note_includes_1_hop_notes() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let graph = setup_test_graph(root);
    create_workspace_from_note(root, "slice_test", &graph.root_id);

    let workspace_notes = list_workspace_notes(root, "slice_test");
    assert!(
        workspace_notes.contains(&graph.child1_id),
        "Child 1 (1 hop) should be in workspace"
    );
    assert!(
        workspace_notes.contains(&graph.child2_id),
        "Child 2 (1 hop) should be in workspace"
    );
}

#[test]
fn test_workspace_from_note_includes_2_hop_notes() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let graph = setup_test_graph(root);
    create_workspace_from_note(root, "slice_test", &graph.root_id);

    let workspace_notes = list_workspace_notes(root, "slice_test");
    assert!(
        workspace_notes.contains(&graph.grandchild_id),
        "Grandchild (2 hops) should be in workspace"
    );
}

#[test]
fn test_workspace_from_note_excludes_notes_beyond_3_hops() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let graph = setup_test_graph(root);
    create_workspace_from_note(root, "slice_test", &graph.root_id);

    let workspace_notes = list_workspace_notes(root, "slice_test");
    assert!(
        !workspace_notes.contains(&graph.far_id),
        "Far away note (4 hops) should NOT be in workspace"
    );
}

#[test]
fn test_workspace_from_note_expected_note_count() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let graph = setup_test_graph(root);
    create_workspace_from_note(root, "slice_test", &graph.root_id);

    let workspace_notes = list_workspace_notes(root, "slice_test");
    let line_count = workspace_notes.lines().filter(|l| !l.is_empty()).count();
    assert_eq!(
        line_count, 4,
        "Workspace should contain exactly 4 notes (root + 2 children + grandchild)"
    );
}
