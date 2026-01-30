use crate::support::{extract_id, qipu};
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_list_filter_by_type() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "fleeting", "Fleeting Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--type", "permanent", "Permanent Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["list", "--type", "fleeting"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Fleeting Note"))
        .stdout(predicate::str::contains("Permanent Note").not());
}

#[test]
fn test_list_filter_by_tag() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args([
            "create",
            "--tag",
            "rust",
            "--tag",
            "programming",
            "Rust Note",
        ])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args([
            "create",
            "--tag",
            "python",
            "--tag",
            "programming",
            "Python Note",
        ])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--tag", "meeting", "Meeting Notes"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["list", "--tag", "rust"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust Note"))
        .stdout(predicate::str::contains("Python Note").not())
        .stdout(predicate::str::contains("Meeting Notes").not());

    qipu()
        .current_dir(dir.path())
        .args(["list", "--tag", "programming"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust Note"))
        .stdout(predicate::str::contains("Python Note"))
        .stdout(predicate::str::contains("Meeting Notes").not());
}

#[test]
fn test_list_filter_by_tag_no_matches() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "--tag", "work", "Work Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["list", "--tag", "nonexistent"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No notes found"));
}

#[test]
fn test_list_filter_by_since() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Old Note"])
        .assert()
        .success();

    std::thread::sleep(std::time::Duration::from_millis(100));

    let since_time = chrono::Utc::now().to_rfc3339();

    std::thread::sleep(std::time::Duration::from_millis(100));

    qipu()
        .current_dir(dir.path())
        .args(["create", "Recent Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["list", "--since", &since_time])
        .assert()
        .success()
        .stdout(predicate::str::contains("Recent Note"))
        .stdout(predicate::str::contains("Old Note").not());
}

#[test]
fn test_list_filter_by_since_no_matches() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Past Note"])
        .assert()
        .success();

    let future_time = (chrono::Utc::now() + chrono::Duration::days(1)).to_rfc3339();

    qipu()
        .current_dir(dir.path())
        .args(["list", "--since", &future_time])
        .assert()
        .success()
        .stdout(predicate::str::contains("No notes found"));
}

#[test]
fn test_list_filter_by_since_exact_match() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let since_time = chrono::Utc::now().to_rfc3339();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Exact Time Note"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["list", "--since", &since_time])
        .assert()
        .success()
        .stdout(predicate::str::contains("Exact Time Note"));
}

#[test]
fn test_list_filter_by_custom_string() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "Note with Custom"])
        .assert()
        .success()
        .get_output()
        .clone();

    let note_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &note_id, "status", "in-progress"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["list", "--custom", "status=in-progress"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Note with Custom"));

    qipu()
        .current_dir(dir.path())
        .args(["list", "--custom", "status=completed"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No notes found"));
}

#[test]
fn test_list_filter_by_custom_number() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["create", "High Priority Note"])
        .assert()
        .success()
        .get_output()
        .clone();

    let note_id = extract_id(&output);

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &note_id, "priority", "10"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["list", "--custom", "priority=10"])
        .assert()
        .success()
        .stdout(predicate::str::contains("High Priority Note"));

    qipu()
        .current_dir(dir.path())
        .args(["list", "--custom", "priority=5"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No notes found"));
}

#[test]
fn test_list_filter_by_custom_boolean() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "Active Note"])
        .assert()
        .success()
        .get_output()
        .clone();
    let note1_id = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "Inactive Note"])
        .assert()
        .success()
        .get_output()
        .clone();
    let note2_id = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &note1_id, "active", "true"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &note2_id, "active", "false"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["list", "--custom", "active=true"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Active Note"))
        .stdout(predicate::str::contains("Inactive Note").not());

    qipu()
        .current_dir(dir.path())
        .args(["list", "--custom", "active=false"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Inactive Note"))
        .stdout(predicate::str::contains("Active Note").not());
}

#[test]
fn test_list_filter_by_custom_with_tag() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let output1 = qipu()
        .current_dir(dir.path())
        .args(["create", "--tag", "work", "Work Note with Custom"])
        .assert()
        .success()
        .get_output()
        .clone();
    let work_id = extract_id(&output1);

    let output2 = qipu()
        .current_dir(dir.path())
        .args(["create", "--tag", "personal", "Personal Note with Custom"])
        .assert()
        .success()
        .get_output()
        .clone();
    let personal_id = extract_id(&output2);

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &work_id, "status", "review"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["custom", "set", &personal_id, "status", "review"])
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .arg("index")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["list", "--tag", "work", "--custom", "status=review"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Work Note with Custom"))
        .stdout(predicate::str::contains("Personal Note with Custom").not());
}
