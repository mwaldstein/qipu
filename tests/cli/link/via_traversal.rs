use crate::support::setup_test_dir;
use crate::support::{extract_id, qipu};

#[test]
fn test_link_list_via_basic() {
    let dir = setup_test_dir();

    let note1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Source"])
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

    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "list", &note1_id, "--format", "json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let links = json.as_array().unwrap();

    assert!(
        links.iter().all(|l| l.get("via").is_none()),
        "Should not have via without compaction"
    );
}

#[test]
fn test_link_list_via_compacted() {
    let dir = setup_test_dir();

    let note1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Compacted Source 1"])
        .output()
        .unwrap();
    let note1_id = extract_id(&note1);
    let note2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Compacted Source 2"])
        .output()
        .unwrap();
    let note2_id = extract_id(&note2);
    let note3 = qipu()
        .current_dir(dir.path())
        .args(["create", "Target"])
        .output()
        .unwrap();
    let note3_id = extract_id(&note3);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &note1_id, &note3_id, "--type", "related"])
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
        .args([
            "compact", "apply", &digest_id, "--note", &note1_id, "--note", &note2_id,
        ])
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
            "link",
            "list",
            &note3_id,
            "--direction",
            "in",
            "--format",
            "json",
            "--no-semantic-inversion",
        ])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let links = json.as_array().unwrap();

    let has_via = links.iter().any(|l| {
        l.get("via").is_some()
            && l["id"].as_str() == Some(&digest_id)
            && l["direction"].as_str() == Some("in")
    });

    assert!(
        has_via,
        "Should show digest with via when source notes are compacted"
    );
}

#[test]
fn test_link_list_via_records_format() {
    let dir = setup_test_dir();

    let note1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Records Source 1"])
        .output()
        .unwrap();
    let note1_id = extract_id(&note1);
    let note2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Records Source 2"])
        .output()
        .unwrap();
    let note2_id = extract_id(&note2);
    let note3 = qipu()
        .current_dir(dir.path())
        .args(["create", "Records Target"])
        .output()
        .unwrap();
    let note3_id = extract_id(&note3);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &note1_id, &note3_id, "--type", "related"])
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
        .args(["create", "Records Digest"])
        .output()
        .unwrap();
    let digest_id = extract_id(&digest);

    qipu()
        .current_dir(dir.path())
        .args([
            "compact", "apply", &digest_id, "--note", &note1_id, "--note", &note2_id,
        ])
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
            "link",
            "list",
            &note3_id,
            "--direction",
            "in",
            "--format",
            "records",
            "--no-semantic-inversion",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains(&format!("via={}", note1_id))
            || stdout.contains(&format!("via={}", note2_id)),
        "Records output should contain via annotation for compacted notes"
    );
}
