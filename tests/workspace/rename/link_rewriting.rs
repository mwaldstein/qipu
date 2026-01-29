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
