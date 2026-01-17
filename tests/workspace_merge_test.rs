use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_workspace_copy_primary_preserves_id() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // 1. Init store
    Command::cargo_bin("qipu")
        .unwrap()
        .arg("init")
        .current_dir(root)
        .assert()
        .success();

    // 2. Create a note in main store
    let output = Command::cargo_bin("qipu")
        .unwrap()
        .arg("create")
        .arg("Main Note")
        .current_dir(root)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let main_id = stdout.trim();
    assert!(!main_id.is_empty(), "Main ID should not be empty");

    // 3. Create a workspace with copy-primary
    Command::cargo_bin("qipu")
        .unwrap()
        .arg("workspace")
        .arg("new")
        .arg("dev")
        .arg("--copy-primary")
        .current_dir(root)
        .assert()
        .success();

    // 4. List notes in workspace and verify ID matches
    let output = Command::cargo_bin("qipu")
        .unwrap()
        .arg("list")
        .arg("--workspace")
        .arg("dev")
        .current_dir(root)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Check if main_id is present in the output
    assert!(
        stdout.contains(main_id),
        "Workspace note ID should match primary note ID"
    );
}

#[test]
fn test_workspace_merge_strategies_links() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // 1. Init
    Command::cargo_bin("qipu")
        .unwrap()
        .arg("init")
        .current_dir(root)
        .assert()
        .success();

    // 2. Create note in main
    let output = Command::cargo_bin("qipu")
        .unwrap()
        .arg("create")
        .arg("Target")
        .current_dir(root)
        .output()
        .unwrap();
    let id = String::from_utf8(output.stdout).unwrap().trim().to_string();

    // 3. Copy to workspace
    Command::cargo_bin("qipu")
        .unwrap()
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

    let output = Command::cargo_bin("qipu")
        .unwrap()
        .arg("--store")
        .arg(ws_store_str)
        .arg("create")
        .arg("WorkspaceOnly")
        .current_dir(root)
        .output()
        .unwrap();
    let ws_id = String::from_utf8(output.stdout).unwrap().trim().to_string();

    // Link Target -> WorkspaceOnly in workspace
    Command::cargo_bin("qipu")
        .unwrap()
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
    let output = Command::cargo_bin("qipu")
        .unwrap()
        .arg("create")
        .arg("MainOnly")
        .current_dir(root)
        .output()
        .unwrap();
    let main_only_id = String::from_utf8(output.stdout).unwrap().trim().to_string();

    // Link Target -> MainOnly in main
    Command::cargo_bin("qipu")
        .unwrap()
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
    Command::cargo_bin("qipu")
        .unwrap()
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
    let links_out = Command::cargo_bin("qipu")
        .unwrap()
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
