use crate::support::setup_test_dir;
use crate::support::{extract_id, qipu};
use tempfile::tempdir;

#[test]
fn test_context_2hop_neighborhood_basic() {
    let dir = setup_test_dir();

    let note1_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Note One"])
        .output()
        .unwrap();
    let note1_id = extract_id(&note1_output);

    let note2_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Note Two"])
        .output()
        .unwrap();
    let note2_id = extract_id(&note2_output);

    let note3_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Note Three"])
        .output()
        .unwrap();
    let note3_id = extract_id(&note3_output);

    let note4_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Note Four"])
        .output()
        .unwrap();
    let note4_id = extract_id(&note4_output);

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
        .args(["link", "add", &note1_id, &note4_id, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &note4_id, &note3_id, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--note",
            &note1_id,
            "--related",
            "0.1",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let notes = json["notes"].as_array().unwrap();
    let note_ids: Vec<&str> = notes.iter().map(|n| n["id"].as_str().unwrap()).collect();

    assert!(note_ids.contains(&note1_id.as_str()));
    assert!(note_ids.contains(&note3_id.as_str()));
    assert!(!note_ids.contains(&note2_id.as_str()));
    assert!(!note_ids.contains(&note4_id.as_str()));

    let note_three: &serde_json::Value = notes
        .iter()
        .find(|n| n["id"].as_str().unwrap() == note3_id)
        .unwrap();
    assert!(note_three.get("via").is_some());
    let via = note_three["via"].as_str().unwrap();
    assert!(via.starts_with("2hop:"));
}

#[test]
fn test_context_2hop_excludes_1hop() {
    let dir = setup_test_dir();

    let note1_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Central Note"])
        .output()
        .unwrap();
    let note1_id = extract_id(&note1_output);

    let note2_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Linked Note"])
        .output()
        .unwrap();
    let note2_id = extract_id(&note2_output);

    let note3_output = qipu()
        .current_dir(dir.path())
        .args(["create", "2-Hop Note"])
        .output()
        .unwrap();
    let note3_id = extract_id(&note3_output);

    let note4_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Another 2-Hop Note"])
        .output()
        .unwrap();
    let note4_id = extract_id(&note4_output);

    let note5_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Another Link"])
        .output()
        .unwrap();
    let note5_id = extract_id(&note5_output);

    let note6_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Yet Another Link"])
        .output()
        .unwrap();
    let note6_id = extract_id(&note6_output);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &note1_id, &note2_id, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &note5_id, &note2_id, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &note2_id, &note3_id, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &note2_id, &note4_id, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &note5_id, &note6_id, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &note1_id, &note6_id, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--note",
            &note2_id,
            "--related",
            "0.1",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let notes = json["notes"].as_array().unwrap();
    let note_ids: Vec<&str> = notes.iter().map(|n| n["id"].as_str().unwrap()).collect();

    assert!(note_ids.contains(&note2_id.as_str()));
    assert!(note_ids.contains(&note6_id.as_str()));
    assert!(!note_ids.contains(&note1_id.as_str()));
    assert!(!note_ids.contains(&note3_id.as_str()));
    assert!(!note_ids.contains(&note4_id.as_str()));
    assert!(!note_ids.contains(&note5_id.as_str()));

    let note_six: &serde_json::Value = notes
        .iter()
        .find(|n| n["id"].as_str().unwrap() == note6_id)
        .unwrap();
    assert!(note_six.get("via").is_some());
    assert!(note_six["via"].as_str().unwrap().starts_with("2hop:"));
}

#[test]
fn test_context_2hop_with_multiple_paths() {
    let dir = setup_test_dir();

    let note1_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Start Note"])
        .output()
        .unwrap();
    let note1_id = extract_id(&note1_output);

    let note2_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Bridge One"])
        .output()
        .unwrap();
    let note2_id = extract_id(&note2_output);

    let note3_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Bridge Two"])
        .output()
        .unwrap();
    let note3_id = extract_id(&note3_output);

    let note4_output = qipu()
        .current_dir(dir.path())
        .args(["create", "Destination Note"])
        .output()
        .unwrap();
    let note4_id = extract_id(&note4_output);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &note1_id, &note2_id, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &note1_id, &note3_id, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &note2_id, &note4_id, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &note3_id, &note4_id, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args([
            "context",
            "--note",
            &note1_id,
            "--related",
            "0.1",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let notes = json["notes"].as_array().unwrap();
    let note_ids: Vec<&str> = notes.iter().map(|n| n["id"].as_str().unwrap()).collect();

    assert!(note_ids.contains(&note1_id.as_str()));
    assert!(note_ids.contains(&note4_id.as_str()));
    assert!(!note_ids.contains(&note2_id.as_str()));
    assert!(!note_ids.contains(&note3_id.as_str()));

    let note_four: &serde_json::Value = notes
        .iter()
        .find(|n| n["id"].as_str().unwrap() == note4_id)
        .unwrap();
    assert!(note_four.get("via").is_some());
    let via = note_four["via"].as_str().unwrap();
    assert!(via.starts_with("2hop:"));
    assert!(via.contains("2.00"));
}
