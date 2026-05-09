use crate::golden::common::{assert_golden_output, qipu};
use std::path::Path;

#[test]
fn test_golden_help_output() {
    let output = String::from_utf8(qipu().arg("--help").output().unwrap().stdout).unwrap();

    let golden_path = Path::new("tests/golden/help.txt");
    assert_golden_output(&output, golden_path).unwrap();
}

#[test]
fn test_advanced_help_shows_hidden_global_options() {
    let output = String::from_utf8(qipu().arg("--help-advanced").output().unwrap().stdout).unwrap();

    assert!(output.contains("Advanced global options:"));
    assert!(output.contains("Hidden commands:"));
    assert!(output.contains("new         Alias for create"));
    assert!(output.contains("custom      Manage custom note metadata"));
    assert!(output.contains("--no-resolve-compaction"));
    assert!(output.contains("--with-compaction-ids"));
    assert!(output.contains("--no-semantic-inversion"));
    assert!(output.contains("hidden from standard help"));
}

#[test]
fn test_golden_version_output() {
    let output = String::from_utf8(qipu().arg("--version").output().unwrap().stdout).unwrap();

    let golden_path = Path::new("tests/golden/version.txt");
    assert_golden_output(&output, golden_path).unwrap();
}
