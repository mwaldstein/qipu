use assert_cmd::{cargo::cargo_bin_cmd, Command};
use std::fs;
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

/// Setup a test store with initialized directory
/// Returns (temp_dir, qipu_command) where command is pre-configured with current_dir
#[allow(dead_code)]
pub fn setup_test_store() -> (TempDir, Command) {
    let dir = TempDir::new().unwrap();
    let mut cmd = qipu();
    cmd.current_dir(dir.path());

    cmd.arg("init").assert().success();

    // Return fresh command for subsequent operations
    let mut new_cmd = qipu();
    new_cmd.current_dir(dir.path());

    (dir, new_cmd)
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

/// Create a note with content and return its ID
#[allow(dead_code)]
pub fn create_note_with_content(dir: &TempDir, title: &str, content: &str) -> String {
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", title])
        .write_stdin(content)
        .output()
        .unwrap();
    extract_id(&output)
}

/// Create a note with specific type and return its ID
#[allow(dead_code)]
pub fn create_note_with_type(dir: &TempDir, title: &str, note_type: &str) -> String {
    let output = qipu()
        .current_dir(dir.path())
        .args(["create", title, "--type", note_type])
        .output()
        .unwrap();
    extract_id(&output)
}

/// Create a note with tags and return its ID
#[allow(dead_code)]
pub fn create_note_with_tags(dir: &TempDir, title: &str, tags: &[&str]) -> String {
    let mut args = vec!["create", title];
    for tag in tags {
        args.push("--tag");
        args.push(tag);
    }
    let output = qipu().current_dir(dir.path()).args(&args).output().unwrap();
    extract_id(&output)
}

/// Initialize a store in the given directory
/// Runs `qipu init` and asserts success
#[allow(dead_code)]
pub fn init_store(dir: &TempDir) {
    qipu()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();
}

/// Setup a custom ontology by writing config to .qipu/config.toml
/// The config_content should be a valid TOML string for the ontology configuration
#[allow(dead_code)]
pub fn setup_custom_ontology(dir: &TempDir, config_content: &str) {
    let config_path = dir.path().join(".qipu/config.toml");
    fs::write(config_path, config_content).unwrap();
}
