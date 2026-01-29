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
fn test_workspace_merge_rename_strategy_multiple_conflicts_same_id() {
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
