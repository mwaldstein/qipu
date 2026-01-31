use crate::support::{extract_id, qipu};

use tempfile::tempdir;

#[test]
fn test_workspace_merge_delete_source_flag() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // 1. Init
    qipu().arg("init").current_dir(root).assert().success();

    // 2. Create workspace
    qipu()
        .args(["workspace", "new", "ws_delete"])
        .current_dir(root)
        .assert()
        .success();

    // 3. Create note in workspace
    let ws_store_path = root.join(".qipu/workspaces/ws_delete");
    let ws_store_str = ws_store_path.to_str().unwrap();

    let output = qipu()
        .arg("--store")
        .arg(ws_store_str)
        .arg("create")
        .arg("Workspace Note")
        .current_dir(root)
        .output()
        .unwrap();
    let ws_id = extract_id(&output);

    // 4. Verify workspace exists before merge
    let ws_dir = root.join(".qipu/workspaces/ws_delete");
    assert!(
        ws_dir.exists(),
        "Workspace directory should exist before merge"
    );

    // 5. Merge with --delete-source flag
    qipu()
        .args([
            "workspace",
            "merge",
            "ws_delete",
            ".",
            "--delete-source",
            "--strategy",
            "skip",
        ])
        .current_dir(root)
        .assert()
        .success();

    // 6. Verify workspace directory is deleted
    assert!(
        !ws_dir.exists(),
        "Workspace directory should be deleted after merge with --delete-source"
    );

    // 7. Verify note was merged into primary
    let list_output = qipu().arg("list").current_dir(root).output().unwrap();
    let list_stdout = String::from_utf8(list_output.stdout).unwrap();
    assert!(
        list_stdout.contains(&ws_id),
        "Merged note ID should be present in primary store"
    );
}

#[test]
fn test_workspace_merge_without_delete_source_preserves_workspace() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // 1. Init
    qipu().arg("init").current_dir(root).assert().success();

    // 2. Create workspace
    qipu()
        .args(["workspace", "new", "ws_keep"])
        .current_dir(root)
        .assert()
        .success();

    // 3. Create note in workspace
    let ws_store_path = root.join(".qipu/workspaces/ws_keep");
    let ws_store_str = ws_store_path.to_str().unwrap();

    let output = qipu()
        .arg("--store")
        .arg(ws_store_str)
        .arg("create")
        .arg("Workspace Note")
        .current_dir(root)
        .output()
        .unwrap();
    let ws_id = extract_id(&output);

    // 4. Merge WITHOUT --delete-source flag
    qipu()
        .args(["workspace", "merge", "ws_keep", ".", "--strategy", "skip"])
        .current_dir(root)
        .assert()
        .success();

    // 5. Verify workspace directory still exists
    let ws_dir = root.join(".qipu/workspaces/ws_keep");
    assert!(
        ws_dir.exists(),
        "Workspace directory should still exist when --delete-source is not used"
    );

    // 6. Verify note was merged into primary
    let list_output = qipu().arg("list").current_dir(root).output().unwrap();
    let list_stdout = String::from_utf8(list_output.stdout).unwrap();
    assert!(
        list_stdout.contains(&ws_id),
        "Merged note ID should be present in primary store"
    );
}

#[test]
fn test_workspace_merge_delete_source_does_not_delete_primary() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // 1. Init
    qipu().arg("init").current_dir(root).assert().success();

    // 2. Create note in primary
    let primary_output = qipu()
        .arg("create")
        .arg("Primary Note")
        .current_dir(root)
        .output()
        .unwrap();
    let primary_id = extract_id(&primary_output);

    // 3. Create workspace
    qipu()
        .args(["workspace", "new", "ws_test"])
        .current_dir(root)
        .assert()
        .success();

    // 4. Create note in workspace
    let ws_store_path = root.join(".qipu/workspaces/ws_test");
    let ws_store_str = ws_store_path.to_str().unwrap();

    let output = qipu()
        .arg("--store")
        .arg(ws_store_str)
        .arg("create")
        .arg("Workspace Note")
        .current_dir(root)
        .output()
        .unwrap();
    extract_id(&output);

    // 5. Attempt to merge from primary (.) to workspace with --delete-source
    // This should NOT delete the primary store
    qipu()
        .args([
            "workspace",
            "merge",
            ".",
            "ws_test",
            "--delete-source",
            "--strategy",
            "skip",
        ])
        .current_dir(root)
        .assert()
        .success();

    // 6. Verify primary store still exists
    let primary_dir = root.join(".qipu");
    assert!(
        primary_dir.exists(),
        "Primary store should still exist even with --delete-source when source is '.'"
    );

    // 7. Verify primary note is still accessible
    let list_output = qipu().arg("list").current_dir(root).output().unwrap();
    let list_stdout = String::from_utf8(list_output.stdout).unwrap();
    assert!(
        list_stdout.contains(&primary_id),
        "Primary note should still be accessible"
    );

    // 8. Verify workspace still exists and contains the merged primary note
    let ws_dir = root.join(".qipu/workspaces/ws_test");
    assert!(ws_dir.exists(), "Workspace should still exist");

    let ws_list_output = qipu()
        .arg("list")
        .arg("--workspace")
        .arg("ws_test")
        .current_dir(root)
        .output()
        .unwrap();
    let ws_list_stdout = String::from_utf8(ws_list_output.stdout).unwrap();
    assert!(
        ws_list_stdout.contains(&primary_id),
        "Primary note should be present in workspace"
    );
}
