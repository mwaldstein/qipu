use assert_cmd::{cargo::cargo_bin_cmd, Command};
use std::process::Output;
use tempfile::tempdir;

fn qipu() -> Command {
    cargo_bin_cmd!("qipu")
}

/// Extract note ID from create command output (first line)
/// Create outputs: <id>\n<path>\n, so we take the first line
fn extract_id(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .map(|s| s.to_string())
        .unwrap_or_default()
}

#[test]
fn test_workspace_copy_primary_preserves_id() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // 1. Init store
    qipu().arg("init").current_dir(root).assert().success();

    // 2. Create a note in main store
    let output = qipu()
        .arg("create")
        .arg("Main Note")
        .current_dir(root)
        .output()
        .unwrap();
    let main_id = extract_id(&output);
    assert!(!main_id.is_empty(), "Main ID should not be empty");

    // 3. Create a workspace with copy-primary
    qipu()
        .arg("workspace")
        .arg("new")
        .arg("dev")
        .arg("--copy-primary")
        .current_dir(root)
        .assert()
        .success();

    // 4. List notes in workspace and verify ID matches
    let output = qipu()
        .arg("list")
        .arg("--workspace")
        .arg("dev")
        .current_dir(root)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Check if main_id is present in the output
    assert!(
        stdout.contains(&main_id),
        "Workspace note ID should match primary note ID"
    );
}

#[test]
fn test_workspace_delete_protection() {
    // 1. Setup primary store
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Init store
    qipu().current_dir(root).arg("init").assert().success();

    // 2. Create a workspace
    qipu()
        .current_dir(root)
        .args(["workspace", "new", "test_ws"])
        .assert()
        .success();

    // 3. Add a note to the workspace (unmerged change)
    qipu()
        .current_dir(root)
        .args(["create", "My Note", "--workspace", "test_ws"])
        .assert()
        .success();

    // 4. Try to delete without --force (should fail with the fix)
    qipu()
        .current_dir(root)
        .args(["workspace", "delete", "test_ws"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("unmerged changes"));

    // 5. Try to delete WITH --force (should succeed)
    qipu()
        .current_dir(root)
        .args(["workspace", "delete", "test_ws", "--force"])
        .assert()
        .success();

    // Verify workspace dir is gone
    let ws_path = root.join(".qipu/workspaces/test_ws");
    assert!(!ws_path.exists());
}

#[test]
fn test_workspace_merge_dry_run() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // 1. Init store
    qipu().arg("init").current_dir(root).assert().success();

    // 2. Create note in primary
    let output = qipu()
        .arg("create")
        .arg("Primary Note")
        .current_dir(root)
        .output()
        .unwrap();
    let primary_id = extract_id(&output);

    // 3. Create workspace
    qipu()
        .arg("workspace")
        .arg("new")
        .arg("ws_test")
        .arg("--empty")
        .current_dir(root)
        .assert()
        .success();

    // 4. Create a note with same ID in workspace (to create conflict)
    let ws_store_path = root.join(".qipu/workspaces/ws_test");
    let ws_store_str = ws_store_path.to_str().unwrap();

    qipu()
        .arg("--store")
        .arg(ws_store_str)
        .arg("create")
        .arg("Workspace Conflict")
        .arg("--id")
        .arg(&primary_id)
        .current_dir(root)
        .assert()
        .success();

    // 5. Create a new note in workspace (no conflict)
    let output = qipu()
        .arg("--store")
        .arg(ws_store_str)
        .arg("create")
        .arg("Workspace Unique")
        .current_dir(root)
        .output()
        .unwrap();
    let unique_id = extract_id(&output);

    // 6. Run dry-run merge
    let output = qipu()
        .arg("workspace")
        .arg("merge")
        .arg("ws_test")
        .arg(".")
        .arg("--dry-run")
        .arg("--strategy")
        .arg("overwrite")
        .current_dir(root)
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Verify dry-run output
    assert!(stdout.contains("Dry-run:"));
    assert!(stdout.contains("Notes to add: 1"));
    assert!(stdout.contains(&unique_id));
    assert!(stdout.contains("Conflicts: 1"));
    assert!(stdout.contains(&primary_id));
    assert!(stdout.contains("Strategy: overwrite"));

    // 7. Verify primary note was NOT modified (dry-run should not make changes)
    let show_output = qipu()
        .arg("show")
        .arg(&primary_id)
        .current_dir(root)
        .output()
        .unwrap();
    let show_stdout = String::from_utf8(show_output.stdout).unwrap();
    assert!(show_stdout.contains("Primary Note"));
    assert!(!show_stdout.contains("Workspace Conflict"));

    // 8. Verify unique note was NOT added to primary (dry-run should not make changes)
    let list_output = qipu().arg("list").current_dir(root).output().unwrap();
    let list_stdout = String::from_utf8(list_output.stdout).unwrap();
    assert!(!list_stdout.contains("Workspace Unique"));
}

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
    let _ws_id = extract_id(&output);

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
