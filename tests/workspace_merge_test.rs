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
fn test_workspace_merge_strategies_links() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // 1. Init
    qipu().arg("init").current_dir(root).assert().success();

    // 2. Create note in main
    let output = qipu()
        .arg("create")
        .arg("Target")
        .current_dir(root)
        .output()
        .unwrap();
    let id = extract_id(&output);

    // 3. Copy to workspace
    qipu()
        .arg("workspace")
        .arg("new")
        .arg("ws_links")
        .arg("--copy-primary")
        .current_dir(root)
        .assert()
        .success();

    // 4. Create WorkspaceOnly note in workspace
    let ws_store_path = root.join(".qipu/workspaces/ws_links");
    // We need to use InitOptions::default() or just ensure path exists?
    // CLI --store argument should work.
    let ws_store_str = ws_store_path.to_str().unwrap();

    let output = qipu()
        .arg("--store")
        .arg(ws_store_str)
        .arg("create")
        .arg("WorkspaceOnly")
        .current_dir(root)
        .output()
        .unwrap();
    let ws_id = extract_id(&output);

    // Link Target -> WorkspaceOnly in workspace
    qipu()
        .arg("--store")
        .arg(ws_store_str)
        .arg("link")
        .arg("add")
        .arg("--type")
        .arg("related")
        .arg(&id)
        .arg(&ws_id)
        .current_dir(root)
        .assert()
        .success();

    // 5. Create MainOnly in main
    let output = qipu()
        .arg("create")
        .arg("MainOnly")
        .current_dir(root)
        .output()
        .unwrap();
    let main_only_id = extract_id(&output);

    // Link Target -> MainOnly in main
    qipu()
        .arg("link")
        .arg("add")
        .arg("--type")
        .arg("related")
        .arg(&id)
        .arg(&main_only_id)
        .current_dir(root)
        .assert()
        .success();

    // 6. Merge workspace into main with merge-links
    qipu()
        .arg("workspace")
        .arg("merge")
        .arg("ws_links")
        .arg(".")
        .arg("--strategy")
        .arg("merge-links")
        .current_dir(root)
        .assert()
        .success();

    // 7. Verify Target in main has BOTH links
    let links_out = qipu()
        .arg("link")
        .arg("list")
        .arg(&id)
        .current_dir(root)
        .output()
        .unwrap();
    let links = String::from_utf8(links_out.stdout).unwrap();

    assert!(
        links.contains(&ws_id),
        "Merged note should contain link from workspace"
    );
    assert!(
        links.contains(&main_only_id),
        "Merged note should contain link from main"
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
        .stdout(predicates::str::contains("unmerged changes"));

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
