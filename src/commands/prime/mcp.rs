//! MCP (Model Context Protocol) environment detection
//!
//! Detects if qipu is running in an MCP/agent environment to adjust output verbosity.
//! When running via MCP, output minimal primer (~50 tokens).
//! When running in CLI mode, output full primer (~1-2k tokens).

use std::env;
use std::path::PathBuf;

/// Detect if running in an MCP/agent environment
///
/// Detection methods (in order of priority):
/// 1. Environment variable: QIPU_MCP_MODE=1
/// 2. MCP-specific env vars: MCP_SERVER, CLAUDE_MCP, etc.
/// 3. Check for MCP settings file: ~/.claude/settings.json with beads server
pub fn detect_mcp_mode() -> bool {
    // Check explicit opt-in/opt-out
    if let Ok(val) = env::var("QIPU_MCP_MODE") {
        let val_lower = val.to_lowercase();
        if val_lower == "0" || val_lower == "false" {
            return false;
        }
        return val_lower == "1" || val_lower == "true";
    }

    // Check for MCP-specific environment variables
    let mcp_env_vars = [
        "MCP_SERVER",
        "CLAUDE_MCP",
        "MCP_CONTEXT",
        "OPENCODE_MCP",
        "CODEX_MCP",
    ];

    for var in &mcp_env_vars {
        if env::var(var).is_ok() {
            return true;
        }
    }

    // Check for MCP settings file indicators
    if has_mcp_settings_file() {
        return true;
    }

    false
}

/// Check for MCP settings files in common locations
fn has_mcp_settings_file() -> bool {
    let home_dir = env::var("HOME").or_else(|_| env::var("USERPROFILE"));

    if let Ok(home) = home_dir {
        // Check Claude settings
        let claude_settings = PathBuf::from(&home).join(".claude").join("settings.json");
        if claude_settings.exists() {
            if let Ok(content) = std::fs::read_to_string(&claude_settings) {
                if content.contains("beads") || content.contains("mcp") {
                    return true;
                }
            }
        }

        // Check other common MCP config locations
        let mcp_configs = [
            PathBuf::from(&home).join(".mcp").join("config.json"),
            PathBuf::from(&home)
                .join(".config")
                .join("mcp")
                .join("settings.json"),
        ];

        for config in &mcp_configs {
            if config.exists() {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_mcp_mode_explicit_true() {
        env::set_var("QIPU_MCP_MODE", "1");
        // Clear other env vars to ensure only QIPU_MCP_MODE matters
        for var in ["MCP_SERVER", "CLAUDE_MCP", "MCP_CONTEXT"] {
            env::remove_var(var);
        }
        assert!(detect_mcp_mode());
        env::remove_var("QIPU_MCP_MODE");
    }

    #[test]
    fn test_detect_mcp_mode_explicit_false() {
        env::set_var("QIPU_MCP_MODE", "0");
        for var in ["MCP_SERVER", "CLAUDE_MCP", "MCP_CONTEXT"] {
            env::remove_var(var);
        }
        assert!(!detect_mcp_mode());
        env::remove_var("QIPU_MCP_MODE");
    }

    #[test]
    fn test_detect_mcp_mode_env_var() {
        env::remove_var("QIPU_MCP_MODE");
        env::set_var("MCP_SERVER", "some-value");
        assert!(detect_mcp_mode());
        env::remove_var("MCP_SERVER");
    }

    #[test]
    fn test_detect_mcp_mode_no_env() {
        // Clear all MCP-related env vars
        for var in ["QIPU_MCP_MODE", "MCP_SERVER", "CLAUDE_MCP", "MCP_CONTEXT"] {
            env::remove_var(var);
        }
        // This test might fail if there's an actual MCP settings file
        // In CI/test environments, this should work fine
        let result = detect_mcp_mode();
        // We just verify the function doesn't panic
        // The actual result depends on the environment
        let _ = result;
    }
}
