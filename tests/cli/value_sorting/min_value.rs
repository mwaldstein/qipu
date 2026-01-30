use crate::support::{extract_id, qipu};
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_search_with_min_value_filter() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "High Value Programming"])
        .output()
        .unwrap();
    let high_id = extract_id(&output);

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Medium Value Programming"])
        .output()
        .unwrap();
    let medium_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["create", "Low Value Programming"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &high_id, "90"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &medium_id, "70"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["search", "--min-value", "60", "programming"])
        .assert()
        .success()
        .stdout(predicate::str::contains("High Value Programming"))
        .stdout(predicate::str::contains("Medium Value Programming"))
        .stdout(predicate::str::contains("Low Value Programming").not());

    qipu()
        .current_dir(dir.path())
        .args(["search", "--min-value", "85", "programming"])
        .assert()
        .success()
        .stdout(predicate::str::contains("High Value Programming"))
        .stdout(predicate::str::contains("Medium Value Programming").not())
        .stdout(predicate::str::contains("Low Value Programming").not());
}

#[test]
fn test_search_min_value_and_sort_combined() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Very High Note"])
        .output()
        .unwrap();
    let very_high_id = extract_id(&output);

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "High Note"])
        .output()
        .unwrap();
    let high_id = extract_id(&output);

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Medium Note"])
        .output()
        .unwrap();
    let medium_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["create", "Low Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &very_high_id, "95"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &high_id, "80"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["value", "set", &medium_id, "65"])
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
            "--format",
            "json",
            "search",
            "--min-value",
            "60",
            "--sort",
            "value",
            "note",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let results: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let results_array = results.as_array().expect("Expected JSON array");

    assert_eq!(
        results_array.len(),
        3,
        "Should find only notes with value >= 60"
    );

    assert_eq!(
        results_array[0]["title"].as_str().unwrap(),
        "Very High Note"
    );
    assert_eq!(results_array[1]["title"].as_str().unwrap(), "High Note");
    assert_eq!(results_array[2]["title"].as_str().unwrap(), "Medium Note");
}
