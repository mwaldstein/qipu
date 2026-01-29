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
fn test_workspace_merge_rename_strategy_basic() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    qipu().arg("init").current_dir(root).assert().success();

    let output = qipu()
        .arg("create")
        .arg("Primary Original")
        .current_dir(root)
        .output()
        .unwrap();
    let primary_id = extract_id(&output);

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

    let show_output = qipu()
        .arg("show")
        .arg(&primary_id)
        .current_dir(root)
        .output()
        .unwrap();
    let show_stdout = String::from_utf8(show_output.stdout).unwrap();
    assert!(show_stdout.contains("Primary Original"));
    assert!(!show_stdout.contains("Workspace Conflict"));

    let list_output = qipu().arg("list").current_dir(root).output().unwrap();
    let list_stdout = String::from_utf8(list_output.stdout).unwrap();

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
