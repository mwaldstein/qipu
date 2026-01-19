//! Setup command for installing qipu integration instructions
//!
//! Provides instructions for integrating qipu with common agent tools.
//! At minimum, supports the AGENTS.md standard (cross-tool compatible).

use crate::cli::{Cli, OutputFormat};
use crate::lib::error::QipuError;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_cli(format: OutputFormat) -> Cli {
        Cli {
            root: None,
            store: None,
            format,
            quiet: false,
            verbose: false,
            log_level: None,
            log_json: false,
            no_resolve_compaction: false,
            with_compaction_ids: false,
            compaction_depth: None,
            compaction_max_nodes: None,
            expand_compaction: false,
            workspace: None,
            no_semantic_inversion: false,
            command: None,
        }
    }

    fn read_stdin() -> String {
        "Test note content".to_string()
    }

    #[test]
    fn test_execute_list_human() {
        let cli = create_cli(OutputFormat::Human);
        let stdout = capture_stdout(|| execute_list(&cli).unwrap());
        assert!(stdout.contains("Available integrations"));
        assert!(stdout.contains("agents-md"));
        assert!(stdout.contains("Usage: qipu setup agents-md"));
    }

    #[test]
    fn test_execute_list_records() {
        let cli = create_cli(OutputFormat::Records);
        let stdout = capture_stdout(|| execute_list(&cli).unwrap());
        assert!(stdout.contains("records=1 mode=setup.list"));
        assert!(stdout.contains("integration name=agents-md"));
    }

    #[test]
    fn test_execute_print_json() {
        let cli = create_cli(OutputFormat::Json);
        let result = execute_print(&cli);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_print_human() {
        let cli = create_cli(OutputFormat::Human);
        let result = execute_print(&cli);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_print_records() {
        let cli = create_cli(OutputFormat::Records);
        let result = execute_print(&cli);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_install_success() {
        let cli = create_cli(OutputFormat::Human);
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let result = execute_install(&cli, "agents-md");
        assert!(result.is_ok());

        let agents_md_path = temp_dir.path().join("AGENTS.md");
        assert!(agents_md_path.exists());

        let content = fs::read_to_string(&agents_md_path).unwrap();
        assert!(content.contains("Qipu is a Zettelkasten-inspired"));
        assert!(content.contains("Quick Start"));

        // Cleanup
        fs::remove_file(agents_md_path).unwrap();
    }

    #[test]
    fn test_execute_install_json() {
        let cli = create_cli(OutputFormat::Json);
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let result = execute_install(&cli, "agents-md");
        assert!(result.is_ok());

        let agents_md_path = temp_dir.path().join("AGENTS.md");
        assert!(agents_md_path.exists());

        // Cleanup
        fs::remove_file(agents_md_path).unwrap();
    }

    #[test]
    fn test_execute_install_records() {
        let cli = create_cli(OutputFormat::Records);
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let result = execute_install(&cli, "agents-md");
        assert!(result.is_ok());

        let agents_md_path = temp_dir.path().join("AGENTS.md");
        assert!(agents_md_path.exists());

        // Cleanup
        fs::remove_file(agents_md_path).unwrap();
    }

    #[test]
    fn test_execute_install_unknown_tool() {
        let cli = create_cli(OutputFormat::Human);
        let result = execute_install(&cli, "unknown-tool");
        assert!(result.is_err());

        match result.unwrap_err() {
            QipuError::UsageError(msg) => {
                assert!(msg.contains("Unknown integration"));
                assert!(msg.contains("unknown-tool"));
            }
            _ => panic!("Expected UsageError"),
        }
    }

    #[test]
    fn test_execute_install_already_exists() {
        let cli = create_cli(OutputFormat::Json);
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        // Create AGENTS.md first
        let path = temp_dir.path().join("AGENTS.md");
        fs::write(&path, "Some content").unwrap();

        let result = execute_install(&cli, "agents-md");
        assert!(result.is_ok());

        let stdout = capture_stdout(|| {});
        assert!(stdout.contains("status"));
        assert!(stdout.contains("exists"));
        assert!(stdout.contains("already exists"));

        // File should still exist
        assert!(path.exists());
    }

    #[test]
    fn test_execute_check_installed() {
        let cli = create_cli(OutputFormat::Json);
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        // Create AGENTS.md
        let path = temp_dir.path().join("AGENTS.md");
        fs::write(&path, "Some content").unwrap();

        let result = execute_check(&cli, "agents-md");
        assert!(result.is_ok());

        let stdout = capture_stdout(|| {});
        assert!(stdout.contains("installed"));
        assert!(stdout.contains("true"));
        assert!(stdout.contains("AGENTS.md"));
    }

    #[test]
    fn test_execute_check_not_installed() {
        let cli = create_cli(OutputFormat::Json);
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let result = execute_check(&cli, "agents-md");
        assert!(result.is_ok());

        let stdout = capture_stdout(|| {});
        assert!(stdout.contains("installed"));
        assert!(stdout.contains("false"));
        assert!(!stdout.contains("path"));
    }

    #[test]
    fn test_execute_check_human() {
        let cli = create_cli(OutputFormat::Human);
        let result = execute_check(&cli, "agents-md");
        assert!(result.is_ok());

        let stdout = capture_stdout(|| {});
        assert!(stdout.contains("AGENTS.md integration is not installed"));
        assert!(stdout.contains("Run `qipu setup agents-md`"));
    }

    #[test]
    fn test_execute_remove_success() {
        let cli = create_cli(OutputFormat::Human);
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        // Create AGENTS.md first
        let path = temp_dir.path().join("AGENTS.md");
        fs::write(&path, "Some content").unwrap();

        let result = execute_remove(&cli, "agents-md");
        assert!(result.is_ok());

        let stdout = capture_stdout(|| {});
        assert!(stdout.contains("Removed AGENTS.md"));

        // File should be removed
        assert!(!path.exists());
    }

    #[test]
    fn test_execute_remove_json() {
        let cli = create_cli(OutputFormat::Json);
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        // Create AGENTS.md first
        let path = temp_dir.path().join("AGENTS.md");
        fs::write(&path, "Some content").unwrap();

        let result = execute_remove(&cli, "agents-md");
        assert!(result.is_ok());

        let stdout = capture_stdout(|| {});
        assert!(stdout.contains("status"));
        assert!(stdout.contains("removed"));
    }

    #[test]
    fn test_execute_remove_not_found() {
        let cli = create_cli(OutputFormat::Json);
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let result = execute_remove(&cli, "agents-md");
        assert!(result.is_ok());

        let stdout = capture_stdout(|| {});
        assert!(stdout.contains("status"));
        assert!(stdout.contains("not-found"));
    }

    #[test]
    fn test_execute_remove_records() {
        let cli = create_cli(OutputFormat::Records);
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        // Create AGENTS.md first
        let path = temp_dir.path().join("AGENTS.md");
        fs::write(&path, "Some content").unwrap();

        let result = execute_remove(&cli, "agents-md");
        assert!(result.is_ok());

        let stdout = capture_stdout(|| {});
        assert!(stdout.contains("mode=setup.remove"));
        assert!(stdout.contains("status=removed"));
    }

    #[test]
    fn test_execute_remove_unknown_tool() {
        let cli = create_cli(OutputFormat::Human);
        let result = execute_remove(&cli, "unknown-tool");
        assert!(result.is_err());

        match result.unwrap_err() {
            QipuError::UsageError(msg) => {
                assert!(msg.contains("Unknown integration"));
                assert!(msg.contains("unknown-tool"));
            }
            _ => panic!("Expected UsageError"),
        }
    }

    #[test]
    fn test_execute_with_list_flag() {
        let cli = create_cli(OutputFormat::Human);
        let result = execute(&cli, true, None, false, false, false);
        assert!(result.is_ok());

        let stdout = capture_stdout(|| {});
        assert!(stdout.contains("Available integrations"));
    }

    #[test]
    fn test_execute_with_print_flag() {
        let cli = create_cli(OutputFormat::Human);
        let result = execute(&cli, false, None, true, false, false);
        assert!(result.is_ok());

        let stdout = capture_stdout(|| {});
        assert!(stdout.contains("Qipu is a Zettelkasten-inspired"));
    }

    #[test]
    fn test_execute_with_check_flag() {
        let cli = create_cli(OutputFormat::Human);
        let result = execute(&cli, false, Some("agents-md"), false, true, false);
        assert!(result.is_ok());

        let stdout = capture_stdout(|| {});
        assert!(stdout.contains("not installed"));
    }

    #[test]
    fn test_execute_with_remove_flag() {
        let cli = create_cli(OutputFormat::Human);
        let result = execute(&cli, false, Some("agents-md"), false, false, true);
        assert!(result.is_ok());

        let stdout = capture_stdout(|| {});
        assert!(stdout.contains("Removed AGENTS.md"));
    }

    #[test]
    fn test_execute_no_args() {
        let cli = create_cli(OutputFormat::Human);
        let result = execute(&cli, false, None, false, false, false);
        assert!(result.is_err());

        match result.unwrap_err() {
            QipuError::UsageError(msg) => {
                assert!(msg.contains("Specify --list, --print, or provide a tool name"));
            }
            _ => panic!("Expected UsageError"),
        }
    }

    #[test]
    fn test_execute_json_output_all_branches() {
        let cli = create_cli(OutputFormat::Json);

        // Test list
        let result = execute(&cli, true, None, false, false, false);
        assert!(result.is_ok());

        // Test print
        let result = execute(&cli, false, None, true, false, false);
        assert!(result.is_ok());

        // Test check
        let result = execute(&cli, false, Some("agents-md"), false, true, false);
        assert!(result.is_ok());

        // Test remove
        let result = execute(&cli, false, Some("agents-md"), false, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_records_output_all_branches() {
        let cli = create_cli(OutputFormat::Records);

        // Test list
        let result = execute(&cli, true, None, false, false, false);
        assert!(result.is_ok());

        // Test print
        let result = execute(&cli, false, None, true, false, false);
        assert!(result.is_ok());

        // Test check
        let result = execute(&cli, false, Some("agents-md"), false, true, false);
        assert!(result.is_ok());

        // Test remove
        let result = execute(&cli, false, Some("agents-md"), false, false, true);
        assert!(result.is_ok());
    }
}

const AGENTS_MD_CONTENT: &str = r#"# Qipu Agent Integration

Qipu is a Zettelkasten-inspired knowledge management system designed for agent workflows.

## Quick Start

Add this section to your agent tool's configuration or prompt:

```
## Qipu Knowledge Memory

You have access to qipu, a knowledge management CLI for capturing research notes and navigating knowledge via links, tags, and Maps of Content.

### Core Commands

- `qipu prime` - Get a session-start primer (store overview, key MOCs, recent notes)
- `qipu create <title>` - Create a new note
- `qipu capture` - Capture note from stdin
- `qipu list` - List notes (filter by --tag, --type, --since)
- `qipu show <id>` - Display a note
- `qipu search <query>` - Search notes by title and body
- `qipu inbox` - Show unprocessed notes (fleeting/literature)
- `qipu context` - Build context bundle for LLM (use --note, --tag, --moc, or --query to select)
- `qipu link list <id>` - List links for a note
- `qipu link tree <id>` - Show link tree (graph neighborhood)
- `qipu link path <from> <to>` - Find path between notes

### Output Formats

All commands support `--format <human|json|records>`:
- `human` - Human-readable (default)
- `json` - Machine-readable structured output
- `records` - Line-oriented format optimized for context injection

### Example Workflows

**Session Start:**
```bash
qipu prime --format records
```

**Capture Research:**
```bash
qipu create "Paper: XYZ" --type literature --tag paper
echo "Key findings..." | qipu capture --title "Insights from XYZ"
```

**Build Context for a Task:**
```bash
# Get overview first
qipu link tree <topic-note-id> --max-hops 2 --format records --max-chars 8000

# Then fetch full content for selected notes
qipu context --note <id1> --note <id2> --format records --with-body --max-chars 16000
```

**Explore Knowledge:**
```bash
qipu search "compaction" --format json
qipu link list <id> --direction both --format json
qipu inbox --exclude-linked
```

### Best Practices

1. **Progressive Disclosure**: Use `qipu link tree` with `--max-chars` to get summaries, then `qipu context --with-body` for details
2. **Deterministic Output**: All commands produce stable, deterministic output for reproducible workflows
3. **Budgeting**: Use `--max-chars` to fit within context limits
4. **Types**: Use note types (fleeting, literature, permanent, moc) to organize knowledge lifecycle
5. **Links**: Use typed links (derived-from, supports, contradicts, part-of) for explicit relationships
```

## Integration Examples

### OpenCode / Cline / Roo-Cline
Add to your project's `AGENTS.md` file (this file is automatically loaded by these tools).

### Cursor
Add to your `.cursorrules` file or project instructions.

### Other Agent Tools
Refer to your tool's documentation for adding custom instructions or tool integrations.

## Store Location

Qipu stores are discovered by walking up from the current directory looking for `.qipu/` or `qipu/`.

To create a store: `qipu init`

For stealth mode (gitignored): `qipu init --stealth`

## More Information

Run `qipu --help` for complete command reference.
Visit the qipu repository for full documentation.
"#;

/// Execute the setup command
pub fn execute(
    cli: &Cli,
    list: bool,
    tool: Option<&str>,
    print: bool,
    check: bool,
    remove: bool,
) -> Result<(), QipuError> {
    // Handle --list
    if list {
        return execute_list(cli);
    }

    // Handle --print
    if print {
        return execute_print(cli);
    }

    // Handle <tool> with optional --check or --remove
    if let Some(tool_name) = tool {
        if check {
            return execute_check(cli, tool_name);
        }
        if remove {
            return execute_remove(cli, tool_name);
        }
        return execute_install(cli, tool_name);
    }

    // No flags specified - show usage
    Err(QipuError::UsageError(
        "Specify --list, --print, or provide a tool name. See `qipu setup --help`".to_string(),
    ))
}

/// List available integrations
fn execute_list(cli: &Cli) -> Result<(), QipuError> {
    match cli.format {
        OutputFormat::Json => {
            let integrations = vec![serde_json::json!({
                "name": "agents-md",
                "description": "AGENTS.md standard (OpenCode, Cline, Roo-Cline, etc.)",
                "status": "available"
            })];
            println!("{}", serde_json::to_string_pretty(&integrations)?);
        }
        OutputFormat::Records => {
            println!("H qipu=1 records=1 mode=setup.list integrations=1");
            println!("D integration name=agents-md description=\"AGENTS.md standard (OpenCode, Cline, Roo-Cline, etc.)\" status=available");
        }
        OutputFormat::Human => {
            println!("Available integrations:");
            println!();
            println!("  agents-md");
            println!(
                "    AGENTS.md standard for OpenCode, Cline, Roo-Cline, and other agent tools"
            );
            println!("    Usage: qipu setup agents-md");
            println!();
            println!("Run `qipu setup <integration>` to install.");
        }
    }
    Ok(())
}

/// Print integration instructions to stdout
fn execute_print(cli: &Cli) -> Result<(), QipuError> {
    match cli.format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "integration": "agents-md",
                "content": AGENTS_MD_CONTENT
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Records => {
            println!("H qipu=1 records=1 mode=setup.print integration=agents-md");
            println!("B agents-md");
            print!("{}", AGENTS_MD_CONTENT);
            println!("B-END");
        }
        OutputFormat::Human => {
            print!("{}", AGENTS_MD_CONTENT);
        }
    }
    Ok(())
}

/// Install integration for a specific tool
fn execute_install(cli: &Cli, tool: &str) -> Result<(), QipuError> {
    // Normalize tool name
    let normalized = tool.to_lowercase().replace('_', "-");

    if normalized != "agents-md" {
        return Err(QipuError::UsageError(format!(
            "Unknown integration: '{}'. Run `qipu setup --list` to see available integrations.",
            tool
        )));
    }

    // Check if AGENTS.md already exists
    let agents_md_path = std::path::Path::new("AGENTS.md");
    if agents_md_path.exists() {
        return match cli.format {
            OutputFormat::Json => {
                let output = serde_json::json!({
                    "status": "exists",
                    "message": "AGENTS.md already exists. Use --print to see the recommended content, or manually update the file.",
                    "path": "AGENTS.md"
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
                Ok(())
            }
            OutputFormat::Records => {
                println!(
                    "H qipu=1 records=1 mode=setup.install integration=agents-md status=exists"
                );
                println!(
                    "D message \"AGENTS.md already exists. Use --print to see recommended content.\""
                );
                Ok(())
            }
            OutputFormat::Human => {
                println!("AGENTS.md already exists in the current directory.");
                println!();
                println!("To see the recommended qipu integration content, run:");
                println!("  qipu setup --print");
                println!();
                println!("Then manually add or update the qipu section in your AGENTS.md file.");
                Ok(())
            }
        };
    }

    // Create AGENTS.md
    std::fs::write(agents_md_path, AGENTS_MD_CONTENT)?;

    match cli.format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "status": "installed",
                "message": "AGENTS.md created successfully",
                "path": "AGENTS.md"
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Records => {
            println!(
                "H qipu=1 records=1 mode=setup.install integration=agents-md status=installed"
            );
            println!("D path AGENTS.md");
        }
        OutputFormat::Human => {
            println!("✓ Created AGENTS.md");
            println!();
            println!("Integration complete! Agent tools that support AGENTS.md will automatically");
            println!("load these instructions when working in this directory.");
            println!();
            println!("Try running: qipu prime");
        }
    }

    Ok(())
}

/// Check if integration is installed
fn execute_check(cli: &Cli, tool: &str) -> Result<(), QipuError> {
    // Normalize tool name
    let normalized = tool.to_lowercase().replace('_', "-");

    if normalized != "agents-md" {
        return Err(QipuError::UsageError(format!(
            "Unknown integration: '{}'. Run `qipu setup --list` to see available integrations.",
            tool
        )));
    }

    let agents_md_path = std::path::Path::new("AGENTS.md");
    let exists = agents_md_path.exists();

    match cli.format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "integration": "agents-md",
                "installed": exists,
                "path": if exists { Some("AGENTS.md") } else { None }
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Records => {
            let status = if exists { "installed" } else { "not-installed" };
            println!(
                "H qipu=1 records=1 mode=setup.check integration=agents-md status={}",
                status
            );
            if exists {
                println!("D path AGENTS.md");
            }
        }
        OutputFormat::Human => {
            if exists {
                println!("✓ AGENTS.md integration is installed");
                println!("  Path: AGENTS.md");
            } else {
                println!("✗ AGENTS.md integration is not installed");
                println!();
                println!("Run `qipu setup agents-md` to install.");
            }
        }
    }

    Ok(())
}

/// Remove integration
fn execute_remove(cli: &Cli, tool: &str) -> Result<(), QipuError> {
    // Normalize tool name
    let normalized = tool.to_lowercase().replace('_', "-");

    if normalized != "agents-md" {
        return Err(QipuError::UsageError(format!(
            "Unknown integration: '{}'. Run `qipu setup --list` to see available integrations.",
            tool
        )));
    }

    let agents_md_path = std::path::Path::new("AGENTS.md");

    if !agents_md_path.exists() {
        return match cli.format {
            OutputFormat::Json => {
                let output = serde_json::json!({
                    "status": "not-found",
                    "message": "AGENTS.md does not exist"
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
                Ok(())
            }
            OutputFormat::Records => {
                println!(
                    "H qipu=1 records=1 mode=setup.remove integration=agents-md status=not-found"
                );
                Ok(())
            }
            OutputFormat::Human => {
                println!("AGENTS.md does not exist (nothing to remove).");
                Ok(())
            }
        };
    }

    // Remove AGENTS.md
    std::fs::remove_file(agents_md_path)?;

    match cli.format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "status": "removed",
                "message": "AGENTS.md removed successfully"
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Records => {
            println!("H qipu=1 records=1 mode=setup.remove integration=agents-md status=removed");
        }
        OutputFormat::Human => {
            println!("✓ Removed AGENTS.md");
        }
    }

    Ok(())
}
