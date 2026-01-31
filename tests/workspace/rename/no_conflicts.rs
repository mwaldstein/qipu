use tempfile::tempdir;

use crate::support::{extract_id, qipu};

#[test]
fn test_workspace_merge_rename_strategy_no_conflicts() {
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
        .arg("ws_no_conflict")
        .arg("--empty")
        .current_dir(root)
        .assert()
        .success();

    let ws_store_path = root.join(".qipu/workspaces/ws_no_conflict");
    let ws_store_str = ws_store_path.to_str().unwrap();

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

    let show_output = qipu()
        .arg("show")
        .arg(&primary_id)
        .current_dir(root)
        .output()
        .unwrap();
    let show_stdout = String::from_utf8(show_output.stdout).unwrap();
    assert!(show_stdout.contains("Primary Note"));

    let list_output = qipu().arg("list").current_dir(root).output().unwrap();
    let list_stdout = String::from_utf8(list_output.stdout).unwrap();
    assert!(list_stdout.contains(&unique_id_1));
    assert!(list_stdout.contains("Workspace Unique 1"));
    assert!(list_stdout.contains(&unique_id_2));
    assert!(list_stdout.contains("Workspace Unique 2"));
}
