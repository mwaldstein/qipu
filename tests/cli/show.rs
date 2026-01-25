use crate::cli::support::{extract_id, qipu};
use predicates::prelude::*;
use tempfile::tempdir;

// ============================================================================
// Show command tests
// ============================================================================

#[test]
fn test_show_note() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create and capture ID
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Show Test"])
        .output()
        .unwrap();

    let id = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap()
        .to_string();

    // Show should display the note
    qipu()
        .current_dir(dir.path())
        .args(["show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Show Test"))
        .stdout(predicate::str::contains(&id));
}

#[test]
fn test_show_nonexistent() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["show", "qp-nonexistent"])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_show_links_no_links() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Note with Links"])
        .output()
        .unwrap();

    let id = extract_id(&output);

    // Show --links with JSON format
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id, "--links"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"links\""))
        .stdout(predicate::str::contains(&id));
}

#[test]
fn test_show_links_records_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create two notes and link them
    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Source"])
        .output()
        .unwrap();
    let id1 = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Target"])
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
        .args(["index", "--rebuild"])
        .assert()
        .success();

    // Show --links with records format should include header and edge lines
    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "show", &id1, "--links"])
        .assert()
        .success()
        .stdout(predicate::str::contains("H qipu=1"))
        .stdout(predicate::str::contains("mode=show.links"))
        .stdout(predicate::str::contains("E "));
}

#[test]
fn test_show_json_includes_value() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note with value
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Valued Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Set value
    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id, "75"])
        .assert()
        .success();

    // Show with JSON format should include value
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"value\": 75"));
}

#[test]
fn test_show_records_includes_value() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note with value
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Valued Note"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Set value
    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id, "50"])
        .assert()
        .success();

    // Show with records format should include value
    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "show", &id])
        .assert()
        .success()
        .stdout(predicate::str::contains("value=50"));
}

#[test]
fn test_show_json_custom_omitted_by_default() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Custom Test"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Set custom metadata
    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &id, "priority", "high"])
        .assert()
        .success();

    // Show JSON without --custom should NOT include custom field
    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("\"custom\""),
        "custom should be omitted by default"
    );
}

#[test]
fn test_show_json_custom_opt_in() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Custom Test"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Set custom metadata
    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &id, "priority", "high"])
        .assert()
        .success();

    // Show JSON with --custom should include custom field
    qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "show", &id, "--custom"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"custom\""))
        .stdout(predicate::str::contains("\"priority\""))
        .stdout(predicate::str::contains("high"));
}

#[test]
fn test_show_records_custom_opt_in() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    // Create a note
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Custom Records Test"])
        .output()
        .unwrap();
    let id = extract_id(&output);

    // Set custom metadata
    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &id, "score", "42"])
        .assert()
        .success();

    // Show records with --custom should include C line
    qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "show", &id, "--custom"])
        .assert()
        .success()
        .stdout(predicate::str::contains("C "))
        .stdout(predicate::str::contains("score=42"));
}

#[test]
fn test_show_links_semantic_inversion_default() {
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
        .args(["show", &id_b, "--links", "--format", "json"])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let links = json["links"].as_array().unwrap();

    assert_eq!(links.len(), 1, "Should have one link");
    let link = &links[0];

    assert_eq!(link["direction"].as_str().unwrap(), "out");
    assert_eq!(link["id"].as_str().unwrap(), id_a);
    assert_eq!(link["type"].as_str().unwrap(), "supported-by");
    assert_eq!(link["source"].as_str().unwrap(), "virtual");
    assert_eq!(link["title"].as_str().unwrap(), "Semantic Source");
}

#[test]
fn test_show_links_semantic_inversion_disabled() {
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
            "show",
            &id_b,
            "--links",
            "--format",
            "json",
            "--no-semantic-inversion",
        ])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let links = json["links"].as_array().unwrap();

    assert_eq!(links.len(), 1, "Should have one link");
    let link = &links[0];

    assert_eq!(link["direction"].as_str().unwrap(), "in");
    assert_eq!(link["id"].as_str().unwrap(), id_a);
    assert_eq!(link["type"].as_str().unwrap(), "supports");
    assert_eq!(link["source"].as_str().unwrap(), "typed");
    assert_eq!(link["title"].as_str().unwrap(), "Semantic Disabled Source");
}

#[test]
fn test_show_links_semantic_inversion_human_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output_a = qipu()
        .current_dir(dir.path())
        .args(["create", "Human Source"])
        .output()
        .unwrap();
    let id_a = extract_id(&output_a);

    let output_b = qipu()
        .current_dir(dir.path())
        .args(["create", "Human Target"])
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
        .args(["show", &id_b, "--links"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Outbound links"),
        "Should show outbound links header"
    );
    assert!(stdout.contains(&id_a), "Should contain source ID");
    assert!(
        stdout.contains("Human Source"),
        "Should contain source title"
    );
    assert!(stdout.contains("supported-by"), "Should show inverted type");
}

#[test]
fn test_show_links_semantic_inversion_disabled_human_format() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output_a = qipu()
        .current_dir(dir.path())
        .args(["create", "Human Disabled Source"])
        .output()
        .unwrap();
    let id_a = extract_id(&output_a);

    let output_b = qipu()
        .current_dir(dir.path())
        .args(["create", "Human Disabled Target"])
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
        .args(["show", &id_b, "--links", "--no-semantic-inversion"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Inbound links"),
        "Should show inbound links header"
    );
    assert!(stdout.contains(&id_a), "Should contain source ID");
    assert!(
        stdout.contains("Human Disabled Source"),
        "Should contain source title"
    );
    assert!(stdout.contains("supports"), "Should show original type");
}
