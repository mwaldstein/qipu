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

#[test]
fn test_workspace_merge_rename_strategy_link_rewriting() {
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

    let output = qipu()
        .arg("create")
        .arg("Primary Note B")
        .current_dir(root)
        .output()
        .unwrap();
    let id_b = extract_id(&output);

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
