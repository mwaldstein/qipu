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
fn test_workspace_merge_skip_strategy() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // 1. Init store
    qipu().arg("init").current_dir(root).assert().success();

    // 2. Create note in primary
    let output = qipu()
        .arg("create")
        .arg("Primary Original")
        .current_dir(root)
        .output()
        .unwrap();
    let primary_id = extract_id(&output);

    // 3. Create workspace
    qipu()
        .arg("workspace")
        .arg("new")
        .arg("ws_skip")
        .arg("--empty")
        .current_dir(root)
        .assert()
        .success();

    let ws_store_path = root.join(".qipu/workspaces/ws_skip");
    let ws_store_str = ws_store_path.to_str().unwrap();

    // 4. Create conflicting note in workspace (same ID, different content)
    qipu()
        .arg("--store")
        .arg(ws_store_str)
        .arg("create")
        .arg("Workspace Modified")
        .arg("--id")
        .arg(&primary_id)
        .current_dir(root)
        .assert()
        .success();

    // 5. Create unique note in workspace
    let output = qipu()
        .arg("--store")
        .arg(ws_store_str)
        .arg("create")
        .arg("Workspace New")
        .current_dir(root)
        .output()
        .unwrap();
    let new_id = extract_id(&output);

    // 6. Merge with skip strategy
    qipu()
        .arg("workspace")
        .arg("merge")
        .arg("ws_skip")
        .arg(".")
        .arg("--strategy")
        .arg("skip")
        .current_dir(root)
        .assert()
        .success();

    // 7. Verify primary note was NOT modified (skip strategy keeps target)
    let show_output = qipu()
        .arg("show")
        .arg(&primary_id)
        .current_dir(root)
        .output()
        .unwrap();
    let show_stdout = String::from_utf8(show_output.stdout).unwrap();
    assert!(show_stdout.contains("Primary Original"));
    assert!(!show_stdout.contains("Workspace Modified"));

    // 8. Verify new note was added to primary
    let list_output = qipu().arg("list").current_dir(root).output().unwrap();
    let list_stdout = String::from_utf8(list_output.stdout).unwrap();
    assert!(list_stdout.contains(&new_id));
    assert!(list_stdout.contains("Workspace New"));
}

#[test]
fn test_workspace_merge_overwrite_strategy() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // 1. Init store
    qipu().arg("init").current_dir(root).assert().success();

    // 2. Create note in primary
    let output = qipu()
        .arg("create")
        .arg("Primary Original")
        .current_dir(root)
        .output()
        .unwrap();
    let primary_id = extract_id(&output);

    // Add some body content to make it distinctive by editing the file
    let notes_dir = root.join(".qipu/notes");
    let note_files: Vec<_> = std::fs::read_dir(&notes_dir)
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(note_files.len(), 1);
    let primary_note_path = note_files[0].path();
    let mut content = std::fs::read_to_string(&primary_note_path).unwrap();
    content.push_str("\nThis is the primary version");
    std::fs::write(&primary_note_path, content).unwrap();

    // 3. Create workspace
    qipu()
        .arg("workspace")
        .arg("new")
        .arg("ws_overwrite")
        .arg("--empty")
        .current_dir(root)
        .assert()
        .success();

    let ws_store_path = root.join(".qipu/workspaces/ws_overwrite");
    let ws_store_str = ws_store_path.to_str().unwrap();

    // 4. Create conflicting note in workspace (same ID, different content)
    qipu()
        .arg("--store")
        .arg(ws_store_str)
        .arg("create")
        .arg("Workspace Updated")
        .arg("--id")
        .arg(&primary_id)
        .current_dir(root)
        .assert()
        .success();

    // Add different body content to workspace version
    let ws_notes_dir = ws_store_path.join("notes");
    let ws_note_files: Vec<_> = std::fs::read_dir(&ws_notes_dir)
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(ws_note_files.len(), 1);
    let ws_note_path = ws_note_files[0].path();
    let mut ws_content = std::fs::read_to_string(&ws_note_path).unwrap();
    ws_content.push_str("\nThis is the workspace version");
    std::fs::write(&ws_note_path, ws_content).unwrap();

    // 5. Create unique note in workspace
    let output = qipu()
        .arg("--store")
        .arg(ws_store_str)
        .arg("create")
        .arg("Workspace New")
        .current_dir(root)
        .output()
        .unwrap();
    let new_id = extract_id(&output);

    // 6. Merge with overwrite strategy
    qipu()
        .arg("workspace")
        .arg("merge")
        .arg("ws_overwrite")
        .arg(".")
        .arg("--strategy")
        .arg("overwrite")
        .current_dir(root)
        .assert()
        .success();

    // 7. Verify primary note was overwritten with workspace version
    let show_output = qipu()
        .arg("show")
        .arg(&primary_id)
        .current_dir(root)
        .output()
        .unwrap();
    let show_stdout = String::from_utf8(show_output.stdout).unwrap();

    // The title should be from workspace version, not primary
    assert!(
        show_stdout.contains("Workspace Updated"),
        "Expected 'Workspace Updated' title but got: {}",
        show_stdout
    );
    assert!(
        !show_stdout.contains("Primary Original"),
        "Should not contain 'Primary Original' but got: {}",
        show_stdout
    );

    // 8. Verify new note was added to primary
    let list_output = qipu().arg("list").current_dir(root).output().unwrap();
    let list_stdout = String::from_utf8(list_output.stdout).unwrap();
    assert!(list_stdout.contains(&new_id));
    assert!(list_stdout.contains("Workspace New"));
}
