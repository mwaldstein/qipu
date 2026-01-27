use crate::cli::support::{extract_id, qipu};
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_link_list_empty() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note without links
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Isolated Note"])
        .output()
        .unwrap();

    let id = extract_id(&output);

    // First build index
    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // List links should show no links
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("No links found"));
}

#[test]
fn test_link_list_json_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes and link them
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "JSON Source"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "JSON Target"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // List in JSON format
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "link", "list", &id1])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"direction\": \"out\""))
        .stdout(predicate::str::contains("\"source\": \"typed\""));
}

#[test]
fn test_link_list_direction_filter() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes and link them
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Direction Source"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Direction Target"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // List only outbound from source
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id1, "--direction", "out"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id2));

    // List only inbound to source should be empty
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id1, "--direction", "in"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No links found"));

    // List only inbound to target should show the link
    qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id2, "--direction", "in"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id1));
}

#[test]
fn test_link_list_records_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes and link them
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Records Source"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Records Target"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    // List in records format
    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "link", "list", &id1])
        .assert()
        .success()
        .stdout(predicate::str::contains("H qipu=1 records=1 store="))
        .stdout(predicate::str::contains("mode=link.list"))
        .stdout(predicate::str::contains("E "));
}

#[test]
fn test_link_list_records_max_chars() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Link List Budget A"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Link List Budget B"])
        .output()
        .unwrap();
    let id2 = extract_id(&output2);

    let output3 = qipu()
        .current_dir(dir.path())
        .args(["create", "Link List Budget C"])
        .output()
        .unwrap();
    let id3 = extract_id(&output3);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id2, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id1, &id3, "--type", "related"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args([
            "--format",
            "records",
            "link",
            "list",
            &id1,
            "--max-chars",
            "120",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("mode=link.list"))
        .stdout(predicate::str::contains("truncated=true"))
        .stdout(predicate::str::contains("N ").not());
}

#[test]
fn test_link_list_semantic_inversion_default() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output_a = qipu()
        .current_dir(dir.path())
        .args(["create", "Semantic Source"])
        .output()
        .unwrap();
    let id_a = extract_id(&output_a);

    let output_b = qipu()
        .current_dir(dir.path())
        .args(["create", "Semantic Target"])
        .output()
        .unwrap();
    let id_b = extract_id(&output_b);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_b, "--type", "supports"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["link", "list", &id_b, "--format", "json"])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let links = json.as_array().unwrap();

    assert_eq!(links.len(), 1, "Should have one link");
    let link = &links[0];

    assert_eq!(link["direction"].as_str().unwrap(), "out");
    assert_eq!(link["id"].as_str().unwrap(), id_a);
    assert_eq!(link["type"].as_str().unwrap(), "supported-by");
    assert_eq!(link["source"].as_str().unwrap(), "virtual");
    assert_eq!(link["title"].as_str().unwrap(), "Semantic Source");
}

#[test]
fn test_link_list_semantic_inversion_disabled() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output_a = qipu()
        .current_dir(dir.path())
        .args(["create", "Semantic Disabled Source"])
        .output()
        .unwrap();
    let id_a = extract_id(&output_a);

    let output_b = qipu()
        .current_dir(dir.path())
        .args(["create", "Semantic Disabled Target"])
        .output()
        .unwrap();
    let id_b = extract_id(&output_b);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_b, "--type", "supports"])
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
            &id_b,
            "--format",
            "json",
            "--no-semantic-inversion",
        ])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let links = json.as_array().unwrap();

    assert_eq!(links.len(), 1, "Should have one link");
    let link = &links[0];

    assert_eq!(link["direction"].as_str().unwrap(), "in");
    assert_eq!(link["id"].as_str().unwrap(), id_a);
    assert_eq!(link["type"].as_str().unwrap(), "supports");
    assert_eq!(link["source"].as_str().unwrap(), "typed");
    assert_eq!(link["title"].as_str().unwrap(), "Semantic Disabled Source");
}

#[test]
fn test_link_list_semantic_inversion_type_filter() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output_a = qipu()
        .current_dir(dir.path())
        .args(["create", "Type Filter Source"])
        .output()
        .unwrap();
    let id_a = extract_id(&output_a);

    let output_b = qipu()
        .current_dir(dir.path())
        .args(["create", "Type Filter Target"])
        .output()
        .unwrap();
    let id_b = extract_id(&output_b);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_b, "--type", "supports"])
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
            &id_b,
            "--format",
            "json",
            "--type",
            "supported-by",
        ])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let links = json.as_array().unwrap();

    assert_eq!(
        links.len(),
        1,
        "Should find link when filtering by inverted type"
    );
    let link = &links[0];

    assert_eq!(link["type"].as_str().unwrap(), "supported-by");
    assert_eq!(link["source"].as_str().unwrap(), "virtual");
}

#[test]
fn test_link_list_semantic_inversion_type_filter_disabled() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output_a = qipu()
        .current_dir(dir.path())
        .args(["create", "Type Filter Disabled Source"])
        .output()
        .unwrap();
    let id_a = extract_id(&output_a);

    let output_b = qipu()
        .current_dir(dir.path())
        .args(["create", "Type Filter Disabled Target"])
        .output()
        .unwrap();
    let id_b = extract_id(&output_b);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_b, "--type", "supports"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "list",
            &id_b,
            "--format",
            "json",
            "--type",
            "supported-by",
            "--no-semantic-inversion",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("[]"));
}

#[test]
fn test_link_list_semantic_inversion_type_filter_original() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output_a = qipu()
        .current_dir(dir.path())
        .args(["create", "Original Type Filter Source"])
        .output()
        .unwrap();
    let id_a = extract_id(&output_a);

    let output_b = qipu()
        .current_dir(dir.path())
        .args(["create", "Original Type Filter Target"])
        .output()
        .unwrap();
    let id_b = extract_id(&output_b);

    qipu()
        .current_dir(dir.path())
        .args(["link", "add", &id_a, &id_b, "--type", "supports"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args([
            "link", "list", &id_b, "--format", "json", "--type", "supports",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("[]"));

    qipu()
        .current_dir(dir.path())
        .args([
            "link",
            "list",
            &id_b,
            "--format",
            "json",
            "--type",
            "supports",
            "--no-semantic-inversion",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"direction\": \"in\""))
        .stdout(predicate::str::contains("\"type\": \"supports\""));
}

#[test]
fn test_link_list_via_basic() {
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
    let dir = tempdir().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
