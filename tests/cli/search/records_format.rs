use crate::cli::support::qipu;
use tempfile::tempdir;

#[test]
fn test_search_records_format_truncated_field() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test Note"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "search", "test"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("H qipu=1 records=1"),
        "search records output should have valid header"
    );
    assert!(
        stdout.contains("mode=search"),
        "search records output should contain mode=search"
    );
}

#[test]
fn test_search_records_format_s_prefix() {
    let dir = tempdir().unwrap();

    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    qipu()
        .current_dir(dir.path())
        .args(["create", "Test note with content"])
        .assert()
        .success();

    let output = qipu()
        .current_dir(dir.path())
        .args(["--format", "records", "search", "content"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Note: search currently does not output S prefix (match_context is always None)
    // This test documents the current behavior and can be updated if match_context is implemented
    assert!(
        !stdout.contains("S "),
        "search records output currently does not contain S prefix"
    );
}
