use crate::golden::common::{assert_golden_output, qipu};
use std::path::Path;

#[test]
fn test_golden_help_output() {
    let output = String::from_utf8(qipu().arg("--help").output().unwrap().stdout).unwrap();

    let golden_path = Path::new("tests/golden/help.txt");
    assert_golden_output(&output, golden_path).unwrap();
}

#[test]
fn test_golden_version_output() {
    let output = String::from_utf8(qipu().arg("--version").output().unwrap().stdout).unwrap();

    let golden_path = Path::new("tests/golden/version.txt");
    assert_golden_output(&output, golden_path).unwrap();
}
