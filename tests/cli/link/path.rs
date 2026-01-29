use crate::cli::support::{extract_id, qipu};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_link_path_with_compaction() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output_start = qipu()
        .current_dir(dir.path())
        .args(["create", "Start Note"])
        .output()
        .unwrap();
    let start_id = extract_id(&output_start);

    let output_middle = qipu()
        .current_dir(dir.path())
        .args(["create", "Middle Note"])
        .output()
        .unwrap();
    let middle_id = extract_id(&output_middle);

    let output_end = qipu()
        .current_dir(dir.path())
        .args(["create", "End Note"])
        .output()
        .unwrap();
    let end_id = extract_id(&output_end);

    let output_digest = qipu()
        .current_dir(dir.path())
        .args(["create", "Digest Note"])
        .output()
        .unwrap();
    let digest_id = extract_id(&output_digest);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &start_id, &middle_id, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &middle_id, &end_id, "--type", "related"])
        .assert()
        .success();

    let notes_dir = dir.path().join(".qipu/notes");
    for entry in fs::read_dir(&notes_dir).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        if name.to_string_lossy().starts_with(&digest_id) {
            let note_content = fs::read_to_string(entry.path()).unwrap();
            let new_content = note_content.replace(
                &format!("id: {}", digest_id),
                &format!("id: {}\ncompacts:\n  - {}", digest_id, middle_id),
            );
            fs::write(entry.path(), new_content).unwrap();
            break;
        }
    }

    qipu()
        .current_dir(dir.path())
        .args(["index", "--rebuild"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "path", &start_id, &end_id])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains(&start_id));
    assert!(stdout.contains(&digest_id));
    assert!(stdout.contains(&end_id));
    assert!(!stdout.contains(&middle_id));
    assert!(stdout.contains("Path length: 2 hop"));
}

#[test]
fn test_link_path_via_basic() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let note1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Start"])
        .output()
        .unwrap();
    let note1_id = extract_id(&note1);
    let note2 = qipu()
        .current_dir(dir.path())
        .args(["create", "End"])
        .output()
        .unwrap();
    let note2_id = extract_id(&note2);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &note1_id, &note2_id, "--type", "related"])
        .assert()
        .success();
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "path", &note1_id, &note2_id, "--format", "json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert!(
        json["notes"]
            .as_array()
            .unwrap()
            .iter()
            .all(|n| n.get("via").is_none()),
        "Should not have via without compaction"
    );
}

#[test]
fn test_link_path_via_compacted_json() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let note1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Compacted Source"])
        .output()
        .unwrap();
    let note1_id = extract_id(&note1);
    let note2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Target"])
        .output()
        .unwrap();
    let note2_id = extract_id(&note2);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &note1_id, &note2_id, "--type", "related"])
        .assert()
        .success();
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let digest = qipu()
        .current_dir(dir.path())
        .args(["create", "Digest"])
        .output()
        .unwrap();
    let digest_id = extract_id(&digest);

    qipu()
        .current_dir(dir.path())
        .args(["compact", "apply", &digest_id, "--note", &note1_id])
        .assert()
        .success();
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "path", &note1_id, &note2_id, "--format", "json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert!(json["found"].as_bool().unwrap());

    let links = json["links"].as_array().unwrap();

    let has_via = links
        .iter()
        .any(|l| l.get("via").is_some() && l["via"].as_str() == Some(&note1_id));

    assert!(
        has_via,
        "Should show via annotation on link when source is compacted"
    );
}

#[test]
fn test_link_path_via_multi_hop() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let note1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Start"])
        .output()
        .unwrap();
    let note1_id = extract_id(&note1);
    let note2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Middle"])
        .output()
        .unwrap();
    let note2_id = extract_id(&note2);
    let note3 = qipu()
        .current_dir(dir.path())
        .args(["create", "End"])
        .output()
        .unwrap();
    let note3_id = extract_id(&note3);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &note1_id, &note2_id, "--type", "related"])
        .assert()
        .success();
    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &note2_id, &note3_id, "--type", "related"])
        .assert()
        .success();
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let digest = qipu()
        .current_dir(dir.path())
        .args(["create", "Digest"])
        .output()
        .unwrap();
    let digest_id = extract_id(&digest);

    qipu()
        .current_dir(dir.path())
        .args(["compact", "apply", &digest_id, "--note", &note2_id])
        .assert()
        .success();
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "path", &note1_id, &note3_id, "--format", "json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert!(json["found"].as_bool().unwrap());
    assert_eq!(json["path_length"].as_u64().unwrap(), 2);

    let links = json["links"].as_array().unwrap();

    let has_via = links
        .iter()
        .any(|l| l.get("via").is_some() && l["via"].as_str() == Some(&note2_id));

    assert!(
        has_via,
        "Should show via annotation on link in multi-hop path"
    );
}
