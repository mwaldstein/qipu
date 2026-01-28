use assert_cmd::{cargo::cargo_bin_cmd, Command};

/// Get a Command for llm-tool-test
pub fn llm_tool_test() -> Command {
    cargo_bin_cmd!("llm-tool-test")
}
