use tempfile::tempdir;

use crate::support::{extract_id, qipu};

#[test]
fn test_workspace_merge_rename_strategy_preserves_unique_notes() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    qipu().arg("init").current_dir(root).assert().success();

    let output = qipu()
        .arg("create")
        .arg("Primary Note A")
        .current_dir(root)
        .output()
        .unwrap();
    let id_a = extract_id(&output);

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

    let list_output = qipu().arg("list").current_dir(root).output().unwrap();
    let list_stdout = String::from_utf8(list_output.stdout).unwrap();
    assert!(list_stdout.contains(&id_a), "Primary note A should exist");
    assert!(list_stdout.contains(&id_b), "Workspace note B should exist");
    assert!(list_stdout.contains(&id_c), "Workspace note C should exist");
    assert!(list_stdout.contains("Primary Note A"));
    assert!(list_stdout.contains("Workspace Note B"));
    assert!(list_stdout.contains("Workspace Note C"));
}
