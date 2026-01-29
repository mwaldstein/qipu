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

#[test]
fn test_workspace_merge_rename_strategy_basic() {
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
        .arg("ws_rename")
        .arg("--empty")
        .current_dir(root)
        .assert()
        .success();

    let ws_store_path = root.join(".qipu/workspaces/ws_rename");
    let ws_store_str = ws_store_path.to_str().unwrap();

    // 4. Create conflicting note in workspace (same ID, different content)
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

    // 5. Create unique note in workspace
    let output = qipu()
        .arg("--store")
        .arg(ws_store_str)
        .arg("create")
        .arg("Workspace Unique")
        .current_dir(root)
        .output()
        .unwrap();
    let unique_id = extract_id(&output);

    // 6. Merge with rename strategy
    qipu()
        .arg("workspace")
        .arg("merge")
        .arg("ws_rename")
        .arg(".")
        .arg("--strategy")
        .arg("rename")
        .current_dir(root)
        .assert()
        .success();

    // 7. Verify primary note was NOT modified (rename strategy keeps target)
    let show_output = qipu()
        .arg("show")
        .arg(&primary_id)
        .current_dir(root)
        .output()
        .unwrap();
    let show_stdout = String::from_utf8(show_output.stdout).unwrap();
    assert!(show_stdout.contains("Primary Original"));
    assert!(!show_stdout.contains("Workspace Conflict"));

    // 8. Verify workspace's conflicting note was added with renamed ID
    let list_output = qipu().arg("list").current_dir(root).output().unwrap();
    let list_stdout = String::from_utf8(list_output.stdout).unwrap();

    // The renamed ID should be primary_id-1
    let renamed_id = format!("{}-1", primary_id);
    assert!(
        list_stdout.contains(&renamed_id),
        "Should contain renamed ID {} in list output: {}",
        renamed_id,
        list_stdout
    );
    assert!(
        list_stdout.contains("Workspace Conflict"),
        "Should contain 'Workspace Conflict' in list output"
    );

    // 9. Verify unique note was added with original ID
    assert!(
        list_stdout.contains(&unique_id),
        "Should contain unique ID {} in list output",
        unique_id
    );
    assert!(
        list_stdout.contains("Workspace Unique"),
        "Should contain 'Workspace Unique' in list output"
    );
}

#[test]
fn test_workspace_merge_rename_strategy_preserves_unique_notes() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // 1. Init store
    qipu().arg("init").current_dir(root).assert().success();

    // 2. Create note A in primary (to ensure we have at least one note)
    let output = qipu()
        .arg("create")
        .arg("Primary Note A")
        .current_dir(root)
        .output()
        .unwrap();
    let id_a = extract_id(&output);

    // 3. Create workspace
    qipu()
        .arg("workspace")
        .arg("new")
        .arg("ws_unique")
        .arg("--empty")
        .current_dir(root)
        .assert()
        .success();

    let ws_store_path = root.join(".qipu/workspaces/ws_unique");
    let ws_store_str = ws_store_path.to_str().unwrap();

    // 4. Create notes B and C in workspace (unique IDs, no conflicts)
    let output = qipu()
        .arg("--store")
        .arg(ws_store_str)
        .arg("create")
        .arg("Workspace Note B")
        .current_dir(root)
        .output()
        .unwrap();
    let id_b = extract_id(&output);

    let output = qipu()
        .arg("--store")
        .arg(ws_store_str)
        .arg("create")
        .arg("Workspace Note C")
        .current_dir(root)
        .output()
        .unwrap();
    let id_c = extract_id(&output);

    // 5. Merge with rename strategy (no conflicts, should just add notes)
    qipu()
        .arg("workspace")
        .arg("merge")
        .arg("ws_unique")
        .arg(".")
        .arg("--strategy")
        .arg("rename")
        .current_dir(root)
        .assert()
        .success();

    // 6. Verify all notes exist in primary
    let list_output = qipu().arg("list").current_dir(root).output().unwrap();
    let list_stdout = String::from_utf8(list_output.stdout).unwrap();
    assert!(list_stdout.contains(&id_a), "Primary note A should exist");
    assert!(list_stdout.contains(&id_b), "Workspace note B should exist");
    assert!(list_stdout.contains(&id_c), "Workspace note C should exist");
    assert!(list_stdout.contains("Primary Note A"));
    assert!(list_stdout.contains("Workspace Note B"));
    assert!(list_stdout.contains("Workspace Note C"));
}

#[test]
fn test_workspace_merge_rename_strategy_multiple_conflicts_same_id() {
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

    // 3. Create workspace 1 with conflicting note
    qipu()
        .arg("workspace")
        .arg("new")
        .arg("ws1")
        .arg("--empty")
        .current_dir(root)
        .assert()
        .success();

    let ws1_store_path = root.join(".qipu/workspaces/ws1");
    let ws1_store_str = ws1_store_path.to_str().unwrap();

    qipu()
        .arg("--store")
        .arg(ws1_store_str)
        .arg("create")
        .arg("WS1 Conflict")
        .arg("--id")
        .arg(&primary_id)
        .current_dir(root)
        .assert()
        .success();

    // 4. Merge ws1 with rename strategy (creates primary_id-1)
    qipu()
        .arg("workspace")
        .arg("merge")
        .arg("ws1")
        .arg(".")
        .arg("--strategy")
        .arg("rename")
        .current_dir(root)
        .assert()
        .success();

    // 5. Create workspace 2 with conflicting note (same ID as primary)
    qipu()
        .arg("workspace")
        .arg("new")
        .arg("ws2")
        .arg("--empty")
        .current_dir(root)
        .assert()
        .success();

    let ws2_store_path = root.join(".qipu/workspaces/ws2");
    let ws2_store_str = ws2_store_path.to_str().unwrap();

    qipu()
        .arg("--store")
        .arg(ws2_store_str)
        .arg("create")
        .arg("WS2 Conflict")
        .arg("--id")
        .arg(&primary_id)
        .current_dir(root)
        .assert()
        .success();

    // 6. Merge ws2 with rename strategy (should create primary_id-1, but it exists, so primary_id-2)
    qipu()
        .arg("workspace")
        .arg("merge")
        .arg("ws2")
        .arg(".")
        .arg("--strategy")
        .arg("rename")
        .current_dir(root)
        .assert()
        .success();

    // 7. Verify we have both renamed notes: primary_id-1 and primary_id-2
    let list_output = qipu().arg("list").current_dir(root).output().unwrap();
    let list_stdout = String::from_utf8(list_output.stdout).unwrap();

    let renamed_id_1 = format!("{}-1", primary_id);
    let renamed_id_2 = format!("{}-2", primary_id);

    assert!(
        list_stdout.contains(&renamed_id_1),
        "Should contain first renamed ID {}",
        renamed_id_1
    );
    assert!(
        list_stdout.contains(&renamed_id_2),
        "Should contain second renamed ID {}",
        renamed_id_2
    );
    assert!(
        list_stdout.contains("WS1 Conflict"),
        "Should contain WS1 Conflict"
    );
    assert!(
        list_stdout.contains("WS2 Conflict"),
        "Should contain WS2 Conflict"
    );
}

#[test]
fn test_workspace_merge_rename_strategy_dry_run() {
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
        .arg("ws_rename_dry")
        .arg("--empty")
        .current_dir(root)
        .assert()
        .success();

    let ws_store_path = root.join(".qipu/workspaces/ws_rename_dry");
    let ws_store_str = ws_store_path.to_str().unwrap();

    // 4. Create conflicting note in workspace
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

    // 5. Create unique note in workspace
    let output = qipu()
        .arg("--store")
        .arg(ws_store_str)
        .arg("create")
        .arg("Workspace Unique")
        .current_dir(root)
        .output()
        .unwrap();
    let unique_id = extract_id(&output);

    // 6. Run dry-run merge with rename strategy
    let output = qipu()
        .arg("workspace")
        .arg("merge")
        .arg("ws_rename_dry")
        .arg(".")
        .arg("--dry-run")
        .arg("--strategy")
        .arg("rename")
        .current_dir(root)
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Verify dry-run output
    // Note: In rename strategy, the conflicting note is shown in conflicts, not additions
    assert!(stdout.contains("Dry-run:"));
    assert!(stdout.contains("Notes to add: 1"));
    assert!(stdout.contains(&unique_id));
    assert!(stdout.contains("Conflicts: 1"));
    assert!(stdout.contains(&primary_id));
    assert!(stdout.contains("Strategy: rename"));

    // 7. Verify primary note was NOT modified
    let show_output = qipu()
        .arg("show")
        .arg(&primary_id)
        .current_dir(root)
        .output()
        .unwrap();
    let show_stdout = String::from_utf8(show_output.stdout).unwrap();
    assert!(show_stdout.contains("Primary Note"));
    assert!(!show_stdout.contains("Workspace Conflict"));

    // 8. Verify neither unique nor renamed note was added to primary
    let list_output = qipu().arg("list").current_dir(root).output().unwrap();
    let list_stdout = String::from_utf8(list_output.stdout).unwrap();
    assert!(!list_stdout.contains("Workspace Conflict"));
    assert!(!list_stdout.contains("Workspace Unique"));
}

#[test]
fn test_workspace_merge_rename_strategy_no_conflicts() {
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
        .arg("ws_no_conflict")
        .arg("--empty")
        .current_dir(root)
        .assert()
        .success();

    let ws_store_path = root.join(".qipu/workspaces/ws_no_conflict");
    let ws_store_str = ws_store_path.to_str().unwrap();

    // 4. Create unique notes in workspace (no ID conflicts)
    let output = qipu()
        .arg("--store")
        .arg(ws_store_str)
        .arg("create")
        .arg("Workspace Unique 1")
        .current_dir(root)
        .output()
        .unwrap();
    let unique_id_1 = extract_id(&output);

    let output = qipu()
        .arg("--store")
        .arg(ws_store_str)
        .arg("create")
        .arg("Workspace Unique 2")
        .current_dir(root)
        .output()
        .unwrap();
    let unique_id_2 = extract_id(&output);

    // 5. Merge with rename strategy (should work fine even with no conflicts)
    qipu()
        .arg("workspace")
        .arg("merge")
        .arg("ws_no_conflict")
        .arg(".")
        .arg("--strategy")
        .arg("rename")
        .current_dir(root)
        .assert()
        .success();

    // 6. Verify primary note unchanged
    let show_output = qipu()
        .arg("show")
        .arg(&primary_id)
        .current_dir(root)
        .output()
        .unwrap();
    let show_stdout = String::from_utf8(show_output.stdout).unwrap();
    assert!(show_stdout.contains("Primary Note"));

    // 7. Verify both workspace notes added with original IDs
    let list_output = qipu().arg("list").current_dir(root).output().unwrap();
    let list_stdout = String::from_utf8(list_output.stdout).unwrap();
    assert!(list_stdout.contains(&unique_id_1));
    assert!(list_stdout.contains("Workspace Unique 1"));
    assert!(list_stdout.contains(&unique_id_2));
    assert!(list_stdout.contains("Workspace Unique 2"));
}

#[test]
fn test_workspace_merge_rename_strategy_link_rewriting() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // 1. Init store
    qipu().arg("init").current_dir(root).assert().success();

    // 2. Create two notes in primary
    let output = qipu()
        .arg("create")
        .arg("Primary Note A")
        .current_dir(root)
        .output()
        .unwrap();
    let id_a = extract_id(&output);

    let output = qipu()
        .arg("create")
        .arg("Primary Note B")
        .current_dir(root)
        .output()
        .unwrap();
    let id_b = extract_id(&output);

    // 3. Create a note in primary that links to both
    let output = qipu()
        .arg("create")
        .arg("Primary Note C")
        .current_dir(root)
        .output()
        .unwrap();
    let id_c = extract_id(&output);

    qipu()
        .args(["link", "add", &id_c, &id_a, "--type", "related"])
        .current_dir(root)
        .assert()
        .success();

    qipu()
        .args(["link", "add", &id_c, &id_b, "--type", "related"])
        .current_dir(root)
        .assert()
        .success();

    // 4. Create workspace
    qipu()
        .arg("workspace")
        .arg("new")
        .arg("ws_link_rewriting")
        .arg("--empty")
        .current_dir(root)
        .assert()
        .success();

    let ws_store_path = root.join(".qipu/workspaces/ws_link_rewriting");
    let ws_store_str = ws_store_path.to_str().unwrap();

    // 5. Create conflicting versions of A and B in workspace (same IDs)
    qipu()
        .arg("--store")
        .arg(ws_store_str)
        .arg("create")
        .arg("Workspace Conflict A")
        .arg("--id")
        .arg(&id_a)
        .current_dir(root)
        .assert()
        .success();

    qipu()
        .arg("--store")
        .arg(ws_store_str)
        .arg("create")
        .arg("Workspace Conflict B")
        .arg("--id")
        .arg(&id_b)
        .current_dir(root)
        .assert()
        .success();

    // 6. Create a unique note in workspace that links to both A and B
    let output = qipu()
        .arg("--store")
        .arg(ws_store_str)
        .arg("create")
        .arg("Workspace Note D")
        .current_dir(root)
        .output()
        .unwrap();
    let id_d = extract_id(&output);

    qipu()
        .arg("--store")
        .arg(ws_store_str)
        .args(["link", "add", &id_d, &id_a, "--type", "derived-from"])
        .current_dir(root)
        .assert()
        .success();

    qipu()
        .arg("--store")
        .arg(ws_store_str)
        .args(["link", "add", &id_d, &id_b, "--type", "supports"])
        .current_dir(root)
        .assert()
        .success();

    // 7. Merge with rename strategy
    qipu()
        .arg("workspace")
        .arg("merge")
        .arg("ws_link_rewriting")
        .arg(".")
        .arg("--strategy")
        .arg("rename")
        .current_dir(root)
        .assert()
        .success();

    // 8. Verify primary notes A and B were NOT modified
    let show_output = qipu()
        .arg("show")
        .arg(&id_a)
        .current_dir(root)
        .output()
        .unwrap();
    let show_stdout = String::from_utf8(show_output.stdout).unwrap();
    assert!(show_stdout.contains("Primary Note A"));
    assert!(!show_stdout.contains("Workspace Conflict A"));

    let show_output = qipu()
        .arg("show")
        .arg(&id_b)
        .current_dir(root)
        .output()
        .unwrap();
    let show_stdout = String::from_utf8(show_output.stdout).unwrap();
    assert!(show_stdout.contains("Primary Note B"));
    assert!(!show_stdout.contains("Workspace Conflict B"));

    // 9. Verify workspace's conflicting notes were added with renamed IDs
    let renamed_id_a = format!("{}-1", id_a);
    let renamed_id_b = format!("{}-1", id_b);

    let list_output = qipu().arg("list").current_dir(root).output().unwrap();
    let list_stdout = String::from_utf8(list_output.stdout).unwrap();
    assert!(
        list_stdout.contains(&renamed_id_a),
        "Should contain renamed ID {} in list output: {}",
        renamed_id_a,
        list_stdout
    );
    assert!(
        list_stdout.contains(&renamed_id_b),
        "Should contain renamed ID {} in list output: {}",
        renamed_id_b,
        list_stdout
    );
    assert!(list_stdout.contains("Workspace Conflict A"));
    assert!(list_stdout.contains("Workspace Conflict B"));

    // 10. Verify workspace note D's links were rewritten to point to renamed IDs
    let show_output = qipu()
        .args(["show", &id_d, "--links", "--format", "json"])
        .current_dir(root)
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&show_output.stdout).unwrap();
    let links = json["links"].as_array().unwrap();

    assert_eq!(links.len(), 2, "Should have two links");

    let mut found_renamed_a = false;
    let mut found_renamed_b = false;
    for link in links {
        let link_id = link["id"].as_str().unwrap();
        let link_type = link["type"].as_str().unwrap();

        if link_id == renamed_id_a && link_type == "derived-from" {
            found_renamed_a = true;
        }
        if link_id == renamed_id_b && link_type == "supports" {
            found_renamed_b = true;
        }
    }

    assert!(
        found_renamed_a,
        "Should have link from D to renamed A ({} with type 'derived-from')",
        renamed_id_a
    );
    assert!(
        found_renamed_b,
        "Should have link from D to renamed B ({} with type 'supports')",
        renamed_id_b
    );

    // 11. Verify primary note C's links were NOT rewritten (still point to original A and B)
    let show_output = qipu()
        .args(["show", &id_c, "--links", "--format", "json"])
        .current_dir(root)
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&show_output.stdout).unwrap();
    let links = json["links"].as_array().unwrap();

    assert_eq!(links.len(), 2, "Should have two links");

    let mut found_original_a = false;
    let mut found_original_b = false;
    for link in links {
        let link_id = link["id"].as_str().unwrap();

        if link_id == id_a {
            found_original_a = true;
        }
        if link_id == id_b {
            found_original_b = true;
        }
    }

    assert!(
        found_original_a,
        "Primary note C should still link to original A ({})",
        id_a
    );
    assert!(
        found_original_b,
        "Primary note C should still link to original B ({})",
        id_b
    );
}
