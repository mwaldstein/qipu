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
        .arg("Workspace Note")
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
