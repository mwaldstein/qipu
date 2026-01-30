use assert_cmd::{cargo::cargo_bin_cmd, Command};
use std::process::Output;
use tempfile::TempDir;

/// Get a Command for qipu
pub fn qipu() -> Command {
    cargo_bin_cmd!("qipu")
}

/// Extract note ID from create command output (first line)
/// Create outputs: <id>\n<path>\n, so we take the first line
pub fn extract_id(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .map(|s| s.to_string())
        .unwrap_or_default()
}

/// Extract note ID from stdout bytes (first line only)
/// Use when you have raw stdout bytes from a command
pub fn extract_id_from_bytes(stdout: &[u8]) -> String {
    String::from_utf8_lossy(stdout)
        .lines()
        .next()
        .map(|s| s.to_string())
        .unwrap_or_default()
}

/// Setup a test store and return the directory only
/// Use when you need full control over command construction
pub fn setup_test_dir() -> TempDir {
    let dir = TempDir::new().unwrap();
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();
    dir
}

/// Create a note and return its ID
pub fn create_note(dir: &TempDir, title: &str) -> String {
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", title])
        .output()
        .unwrap();
    extract_id(&output)
}
