use crate::cli::support::{extract_id, qipu};
use tempfile::tempdir;

#[test]
fn test_search_sort_by_value() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Low Value Note"])
        .output()
        .unwrap();
    let low_id = extract_id(&output);

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Medium Value Note"])
        .output()
        .unwrap();
    let medium_id = extract_id(&output);

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "High Value Note"])
        .output()
        .unwrap();
    let high_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &low_id, "30"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &medium_id, "60"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &high_id, "90"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "search", "--sort", "value", "value"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let results: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let results_array = results.as_array().expect("Expected JSON array");

    assert_eq!(results_array.len(), 3, "Should find all three notes");

    assert_eq!(
        results_array[0]["title"].as_str().unwrap(),
        "High Value Note"
    );
    assert_eq!(
        results_array[1]["title"].as_str().unwrap(),
        "Medium Value Note"
    );
    assert_eq!(
        results_array[2]["title"].as_str().unwrap(),
        "Low Value Note"
    );
}

#[test]
fn test_search_sort_by_value_with_defaults() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Explicit Value Note"])
        .output()
        .unwrap();
    let explicit_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["create", "Default Value Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &explicit_id, "90"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "search", "--sort", "value", "value"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let results: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let results_array = results.as_array().expect("Expected JSON array");

    assert_eq!(results_array.len(), 2, "Should find both notes");

    assert_eq!(
        results_array[0]["title"].as_str().unwrap(),
        "Explicit Value Note"
    );
    assert_eq!(
        results_array[1]["title"].as_str().unwrap(),
        "Default Value Note"
    );
}

#[test]
fn test_search_sort_by_value_all_defaults() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "First Default Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Second Default Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Third Default Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "search", "--sort", "value", "note"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let results: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let results_array = results.as_array().expect("Expected JSON array");

    assert_eq!(results_array.len(), 3, "Should find all three notes");
}

#[test]
fn test_search_sort_by_value_same_value() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Same Value Note One"])
        .output()
        .unwrap();
    let id1 = extract_id(&output);

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Same Value Note Two"])
        .output()
        .unwrap();
    let id2 = extract_id(&output);

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Same Value Note Three"])
        .output()
        .unwrap();
    let id3 = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id1, "75"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id2, "75"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &id3, "75"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "search", "--sort", "value", "note"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let results: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let results_array = results.as_array().expect("Expected JSON array");

    assert_eq!(results_array.len(), 3, "Should find all three notes");
}

#[test]
fn test_search_sort_by_value_min_max() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Min Value Note"])
        .output()
        .unwrap();
    let min_id = extract_id(&output);

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Max Value Note"])
        .output()
        .unwrap();
    let max_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &min_id, "0"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &max_id, "100"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "search", "--sort", "value", "note"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let results: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let results_array = results.as_array().expect("Expected JSON array");

    assert_eq!(results_array.len(), 2, "Should find both notes");

    assert_eq!(
        results_array[0]["title"].as_str().unwrap(),
        "Max Value Note"
    );

    assert_eq!(
        results_array[1]["title"].as_str().unwrap(),
        "Min Value Note"
    );
}

#[test]
fn test_search_sort_by_value_with_min_max_and_defaults() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Min Value Note"])
        .output()
        .unwrap();
    let min_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["create", "Default Value Note"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Max Value Note"])
        .output()
        .unwrap();
    let max_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &min_id, "0"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &max_id, "100"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "json", "search", "--sort", "value", "note"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let results: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let results_array = results.as_array().expect("Expected JSON array");

    assert_eq!(results_array.len(), 3, "Should find all three notes");

    assert_eq!(
        results_array[0]["title"].as_str().unwrap(),
        "Max Value Note"
    );

    assert_eq!(
        results_array[1]["title"].as_str().unwrap(),
        "Default Value Note"
    );

    assert_eq!(
        results_array[2]["title"].as_str().unwrap(),
        "Min Value Note"
    );
}
