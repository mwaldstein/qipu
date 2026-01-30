use crate::support::setup_test_dir;
use crate::support::{extract_id, qipu};

#[test]
fn test_show_links_semantic_inversion_default() {
    let dir = setup_test_dir();

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
    let dir = setup_test_dir();

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
    let dir = setup_test_dir();

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
    let dir = setup_test_dir();

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
