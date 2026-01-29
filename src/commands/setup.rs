//! Setup command for installing qipu integration instructions
//!
//! Provides instructions for integrating qipu with common agent tools.
//! At minimum, supports the AGENTS.md standard (cross-tool compatible).

use crate::cli::{Cli, OutputFormat};
use crate::commands::format::{
    print_json_status, print_records_data, print_records_header, wrap_records_body,
};
use qipu_core::error::QipuError;
use std::path::PathBuf;

const ONBOARD_SNIPPET: &str = r#"## Qipu Knowledge

This project uses **qipu** for knowledge management.
Run `qipu prime` for workflow context.

**Quick reference:**
- `qipu prime` - Get store overview
- `qipu create` - Create note
- `qipu capture` - Quick capture
- `qipu search` - Search notes
- `qipu context` - Build LLM context

For full workflow: `qipu prime`
"#;

fn get_agents_md_path(cli: &Cli) -> PathBuf {
    match &cli.root {
        Some(root) => root.join("AGENTS.md"),
        None => PathBuf::from("AGENTS.md"),
    }
}

fn validate_tool_name(tool: &str) -> Result<&str, QipuError> {
    let normalized = tool.to_lowercase().replace('_', "-");
    if normalized != "agents-md" {
        return Err(QipuError::UsageError(format!(
            "Unknown integration: '{}'. Run `qipu setup --list` to see available integrations.",
            tool
        )));
    }
    Ok(tool)
}

const AGENTS_MD_CONTENT: &str = r#"# Qipu Agent Integration

Qipu is a Zettelkasten-inspired knowledge management system designed for agent workflows.

## Quick Start

Add this section to your agent tool's configuration or prompt:

```
## Qipu Knowledge Memory

You have access to qipu, a knowledge management CLI for capturing research notes and navigating knowledge via links, tags, and Maps of Content.

### Important: Always Use the CLI

**Never directly read files from `.qipu/notes/` or `.qipu/mocs/`.** Always use the qipu CLI commands:

- The CLI provides consistent formatting (human, json, records)
- Budget control with `--max-chars` ensures you stay within context limits
- Graph context is preserved (links, tags, relationships are resolved correctly)
- Compaction and other internal features work correctly via CLI queries

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

/// Execute the onboard command - display minimal AGENTS.md snippet
pub fn execute_onboard(cli: &Cli) -> Result<(), QipuError> {
    match cli.format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "snippet": ONBOARD_SNIPPET,
                "instruction": "Add this snippet to AGENTS.md for qipu integration"
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Records => {
            print_records_header("onboard", &[]);
            wrap_records_body("snippet", ONBOARD_SNIPPET);
        }
        OutputFormat::Human => {
            print!("{}", ONBOARD_SNIPPET);
        }
    }
    Ok(())
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
            print_records_header("setup.print", &[("integration", "agents-md")]);
            wrap_records_body("agents-md", AGENTS_MD_CONTENT);
        }
        OutputFormat::Human => {
            print!("{}", AGENTS_MD_CONTENT);
        }
    }
    Ok(())
}

/// Install integration for a specific tool
fn execute_install(cli: &Cli, tool: &str) -> Result<(), QipuError> {
    validate_tool_name(tool)?;

    let agents_md_path = get_agents_md_path(cli);
    if agents_md_path.exists() {
        return match cli.format {
            OutputFormat::Json => {
                print_json_status(
                    "exists",
                    Some("AGENTS.md already exists. Use --print to see the recommended content, or manually update the file."),
                    &[("path", serde_json::json!("AGENTS.md"))],
                )
            }
            OutputFormat::Records => {
                print_records_header("setup.install", &[("integration", "agents-md"), ("status", "exists")]);
                print_records_data("message", "AGENTS.md already exists. Use --print to see recommended content.");
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

    std::fs::write(agents_md_path, AGENTS_MD_CONTENT)?;

    match cli.format {
        OutputFormat::Json => print_json_status(
            "installed",
            Some("AGENTS.md created successfully"),
            &[("path", serde_json::json!("AGENTS.md"))],
        )?,
        OutputFormat::Records => {
            print_records_header(
                "setup.install",
                &[("integration", "agents-md"), ("status", "installed")],
            );
            print_records_data("path", "AGENTS.md");
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
    validate_tool_name(tool)?;

    let agents_md_path = get_agents_md_path(cli);
    let exists = agents_md_path.exists();

    match cli.format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "integration": "agents-md",
                "installed": exists,
                "path": if exists { Some(agents_md_path.display().to_string()) } else { None }
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Records => {
            let status = if exists { "installed" } else { "not-installed" };
            print_records_header(
                "setup.check",
                &[("integration", "agents-md"), ("status", status)],
            );
            if exists {
                print_records_data("path", "AGENTS.md");
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
    validate_tool_name(tool)?;

    let agents_md_path = get_agents_md_path(cli);

    if !agents_md_path.exists() {
        return match cli.format {
            OutputFormat::Json => {
                print_json_status("not-found", Some("AGENTS.md does not exist"), &[])
            }
            OutputFormat::Records => {
                print_records_header(
                    "setup.remove",
                    &[("integration", "agents-md"), ("status", "not-found")],
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
            print_json_status("removed", Some("AGENTS.md removed successfully"), &[])?
        }
        OutputFormat::Records => {
            print_records_header(
                "setup.remove",
                &[("integration", "agents-md"), ("status", "removed")],
            );
        }
        OutputFormat::Human => {
            println!("✓ Removed AGENTS.md");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
