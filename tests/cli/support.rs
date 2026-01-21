use assert_cmd::{cargo::cargo_bin_cmd, Command};
use std::process::Output;

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
