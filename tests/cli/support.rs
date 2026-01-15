use assert_cmd::{cargo::cargo_bin_cmd, Command};

/// Get a Command for qipu
pub fn qipu() -> Command {
    cargo_bin_cmd!("qipu")
}
