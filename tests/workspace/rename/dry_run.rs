use assert_cmd::{cargo::cargo_bin_cmd, Command};
use std::process::Output;
use tempfile::tempdir;

fn qipu() -> Command {
    cargo_bin_cmd!("qipu")
}

fn extract_id(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .map(|s| s.to_string())
        .unwrap_or_default()
}

#[test]
fn test_workspace_merge_rename_strategy_dry_run() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    qipu().arg("init").current_dir(root).assert().success();

    let output = qipu()
        .arg("create")
        .arg("Primary Note")
        .current_dir(root)
        .output()
        .unwrap();
    let primary_id = extract_id(&output);

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

    let output = qipu()
        .arg("--store")
        .arg(ws_store_str)
        .arg("create")
        .arg("Workspace Unique")
        .current_dir(root)
        .output()
        .unwrap();
    let unique_id = extract_id(&output);

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

    assert!(stdout.contains("Dry-run:"));
    assert!(stdout.contains("Notes to add: 1"));
    assert!(stdout.contains(&unique_id));
    assert!(stdout.contains("Conflicts: 1"));
    assert!(stdout.contains(&primary_id));
    assert!(stdout.contains("Strategy: rename"));

    let show_output = qipu()
        .arg("show")
        .arg(&primary_id)
        .current_dir(root)
        .output()
        .unwrap();
    let show_stdout = String::from_utf8(show_output.stdout).unwrap();
    assert!(show_stdout.contains("Primary Note"));
    assert!(!show_stdout.contains("Workspace Conflict"));

    let list_output = qipu().arg("list").current_dir(root).output().unwrap();
    let list_stdout = String::from_utf8(list_output.stdout).unwrap();
    assert!(!list_stdout.contains("Workspace Conflict"));
    assert!(!list_stdout.contains("Workspace Unique"));
}
