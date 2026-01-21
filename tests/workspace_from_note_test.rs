use assert_cmd::{cargo::cargo_bin_cmd, Command};
use tempfile::tempdir;

fn qipu() -> Command {
    cargo_bin_cmd!("qipu")
}

#[test]
fn test_workspace_from_note_performs_graph_slice() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // 1. Init store
    qipu().arg("init").current_dir(root).assert().success();

    // 2. Create a root note
    let output = qipu()
        .arg("create")
        .arg("Root Note")
        .current_dir(root)
        .output()
        .unwrap();
    let root_id = String::from_utf8(output.stdout).unwrap().trim().to_string();
    assert!(!root_id.is_empty(), "Root ID should not be empty");

    // 3. Create linked notes (1 hop away)
    let output = qipu()
        .arg("create")
        .arg("Child 1")
        .current_dir(root)
        .output()
        .unwrap();
    let child1_id = String::from_utf8(output.stdout).unwrap().trim().to_string();

    let output = qipu()
        .arg("create")
        .arg("Child 2")
        .current_dir(root)
        .output()
        .unwrap();
    let child2_id = String::from_utf8(output.stdout).unwrap().trim().to_string();

    // 4. Link root to children
    qipu()
        .arg("link")
        .arg("add")
        .arg("--type")
        .arg("part-of")
        .arg(&root_id)
        .arg(&child1_id)
        .current_dir(root)
        .assert()
        .success();

    qipu()
        .arg("link")
        .arg("add")
        .arg("--type")
        .arg("part-of")
        .arg(&root_id)
        .arg(&child2_id)
        .current_dir(root)
        .assert()
        .success();

    // 5. Create a grandchild (2 hops away)
    let output = qipu()
        .arg("create")
        .arg("Grandchild")
        .current_dir(root)
        .output()
        .unwrap();
    let grandchild_id = String::from_utf8(output.stdout).unwrap().trim().to_string();

    qipu()
        .arg("link")
        .arg("add")
        .arg("--type")
        .arg("part-of")
        .arg(&child1_id)
        .arg(&grandchild_id)
        .current_dir(root)
        .assert()
        .success();

    // 6. Create a note far away (4 hops away - should not be included)
    let output = qipu()
        .arg("create")
        .arg("Far Away Note")
        .current_dir(root)
        .output()
        .unwrap();
    let far_id = String::from_utf8(output.stdout).unwrap().trim().to_string();

    qipu()
        .arg("link")
        .arg("add")
        .arg("--type")
        .arg("related")
        .arg(&grandchild_id)
        .arg(&far_id)
        .current_dir(root)
        .assert()
        .success();

    // 7. Create workspace with --from-note (should include notes within 3 hops)
    qipu()
        .arg("workspace")
        .arg("new")
        .arg("slice_test")
        .arg("--from-note")
        .arg(&root_id)
        .current_dir(root)
        .assert()
        .success();

    // 8. List notes in workspace
    let output = qipu()
        .arg("list")
        .arg("--workspace")
        .arg("slice_test")
        .current_dir(root)
        .output()
        .unwrap();
    let workspace_notes = String::from_utf8(output.stdout).unwrap();

    // 9. Verify all notes within 3 hops are included
    assert!(
        workspace_notes.contains(&root_id),
        "Root note should be in workspace"
    );
    assert!(
        workspace_notes.contains(&child1_id),
        "Child 1 (1 hop) should be in workspace"
    );
    assert!(
        workspace_notes.contains(&child2_id),
        "Child 2 (1 hop) should be in workspace"
    );
    assert!(
        workspace_notes.contains(&grandchild_id),
        "Grandchild (2 hops) should be in workspace"
    );

    // 10. Verify far away note (4 hops) is NOT included
    assert!(
        !workspace_notes.contains(&far_id),
        "Far away note (4 hops) should NOT be in workspace"
    );

    // 11. Verify workspace has 4 notes total
    let line_count = workspace_notes.lines().filter(|l| !l.is_empty()).count();
    assert_eq!(
        line_count, 4,
        "Workspace should contain exactly 4 notes (root + 2 children + grandchild)"
    );
}
