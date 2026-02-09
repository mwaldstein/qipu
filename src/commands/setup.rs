//! Setup command for installing qipu integration instructions
//!
//! Provides instructions for integrating qipu with common agent tools.
//! At minimum, supports the AGENTS.md standard (cross-tool compatible).

mod content;

use crate::cli::Cli;
use crate::commands::dispatch::trace_command_always;
use crate::commands::format::{
    output_by_format_result, print_json_status, print_records_data, print_records_header,
    wrap_records_body,
};
use crate::output_by_format;
use content::{AGENTS_MD_CONTENT, CURSOR_RULES_CONTENT, ONBOARD_SNIPPET};
use qipu_core::bail_usage;
use qipu_core::error::QipuError;
use std::path::PathBuf;
use std::time::Instant;

fn get_agents_md_path(cli: &Cli) -> PathBuf {
    match &cli.root {
        Some(root) => root.join("AGENTS.md"),
        None => PathBuf::from("AGENTS.md"),
    }
}

fn get_cursor_rules_path(cli: &Cli) -> PathBuf {
    match &cli.root {
        Some(root) => root.join(".cursor").join("rules").join("qipu.mdc"),
        None => PathBuf::from(".cursor").join("rules").join("qipu.mdc"),
    }
}

/// Execute the setup command
pub fn execute(
    cli: &Cli,
    list: bool,
    tool: Option<&str>,
    print: bool,
    check: bool,
    remove: bool,
    start: Instant,
) -> Result<(), QipuError> {
    // Handle --list
    if list {
        let result = execute_list(cli, start);
        return result;
    }

    // Handle --print
    if print {
        let result = execute_print(cli, start);
        return result;
    }

    // Handle <tool> with optional --check or --remove
    if let Some(tool_name) = tool {
        let normalized = tool_name.to_lowercase().replace('_', "-");
        let valid = ["agents-md", "cursor"];
        if !valid.contains(&normalized.as_str()) {
            bail_usage!(format!(
                "Unknown integration: '{}'. Run `qipu setup --list` to see available integrations.",
                tool_name
            ));
        }
        if check {
            let result = match normalized.as_str() {
                "agents-md" => execute_check_agents_md(cli),
                "cursor" => execute_check_cursor(cli),
                _ => unreachable!(),
            };
            trace_command_always!(start, "setup_check");
            return result;
        }
        if remove {
            let result = match normalized.as_str() {
                "agents-md" => execute_remove_agents_md(cli),
                "cursor" => execute_remove_cursor(cli),
                _ => unreachable!(),
            };
            trace_command_always!(start, "setup_remove");
            return result;
        }
        let result = match normalized.as_str() {
            "agents-md" => execute_install_agents_md(cli),
            "cursor" => execute_install_cursor(cli),
            _ => unreachable!(),
        };
        trace_command_always!(start, "setup_install");
        return result;
    }

    // No flags specified - show usage
    Err(QipuError::UsageError(
        "Specify --list, --print, or provide a tool name. See `qipu setup --help`".to_string(),
    ))
}

/// Execute the onboard command - display minimal AGENTS.md snippet
pub fn execute_onboard(cli: &Cli, start: Instant) -> Result<(), QipuError> {
    output_by_format!(cli.format,
        json => {
            let output = serde_json::json!({
                "snippet": ONBOARD_SNIPPET,
                "instruction": "Add this snippet to AGENTS.md for qipu integration"
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        },
        human => {
            print!("{}", ONBOARD_SNIPPET);
        },
        records => {
            print_records_header("onboard", &[]);
            wrap_records_body("snippet", ONBOARD_SNIPPET);
        }
    );
    trace_command_always!(start, "setup_onboard");
    Ok(())
}

/// List available integrations
fn execute_list(cli: &Cli, start: Instant) -> Result<(), QipuError> {
    output_by_format!(cli.format,
        json => {
            let integrations = vec![
                serde_json::json!({
                    "name": "agents-md",
                    "description": "AGENTS.md standard (OpenCode, Cline, Roo-Cline, etc.)",
                    "status": "available"
                }),
                serde_json::json!({
                    "name": "cursor",
                    "description": "Cursor IDE rules (.cursor/rules/qipu.mdc)",
                    "status": "available"
                }),
            ];
            println!("{}", serde_json::to_string_pretty(&integrations)?);
        },
        human => {
            println!("Available integrations:");
            println!();
            println!("  agents-md");
            println!(
                "    AGENTS.md standard for OpenCode, Cline, Roo-Cline, and other agent tools"
            );
            println!("    Usage: qipu setup agents-md");
            println!();
            println!("  cursor");
            println!("    Cursor IDE project rules");
            println!("    Usage: qipu setup cursor");
            println!();
            println!("Run `qipu setup <integration>` to install.");
        },
        records => {
            println!("H qipu=1 records=1 mode=setup.list integrations=2");
            println!("D integration name=agents-md description=\"AGENTS.md standard (OpenCode, Cline, Roo-Cline, etc.)\" status=available");
            println!("D integration name=cursor description=\"Cursor IDE rules (.cursor/rules/qipu.mdc)\" status=available");
        }
    );
    trace_command_always!(start, "setup_list");
    Ok(())
}

/// Print integration instructions to stdout
fn execute_print(cli: &Cli, start: Instant) -> Result<(), QipuError> {
    // Default to agents-md content for --print flag
    output_by_format!(cli.format,
        json => {
            let output = serde_json::json!({
                "integration": "agents-md",
                "content": AGENTS_MD_CONTENT
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        },
        human => {
            print!("{}", AGENTS_MD_CONTENT);
        },
        records => {
            print_records_header("setup.print", &[("integration", "agents-md")]);
            wrap_records_body("agents-md", AGENTS_MD_CONTENT);
        }
    );
    trace_command_always!(start, "setup_print");
    Ok(())
}

/// Install AGENTS.md integration
fn execute_install_agents_md(cli: &Cli) -> Result<(), QipuError> {
    let agents_md_path = get_agents_md_path(cli);
    if agents_md_path.exists() {
        return output_by_format!(cli.format,
            json => {
                print_json_status(
                    "exists",
                    Some("AGENTS.md already exists. Use --print to see the recommended content, or manually update the file."),
                    &[("path", serde_json::json!("AGENTS.md"))],
                )
            },
            human => {
                println!("AGENTS.md already exists in the current directory.");
                println!();
                println!("To see the recommended qipu integration content, run:");
                println!("  qipu setup --print");
                println!();
                println!("Then manually add or update the qipu section in your AGENTS.md file.");
                Ok(())
            },
            records => {
                print_records_header("setup.install", &[("integration", "agents-md"), ("status", "exists")]);
                print_records_data("message", "AGENTS.md already exists. Use --print to see recommended content.");
                Ok(())
            }
        );
    }

    std::fs::write(agents_md_path, AGENTS_MD_CONTENT)?;

    output_by_format_result!(cli.format,
        json => print_json_status(
            "installed",
            Some("AGENTS.md created successfully"),
            &[("path", serde_json::json!("AGENTS.md"))],
        ),
        human => {
            println!("✓ Created AGENTS.md");
            println!();
            println!("Integration complete! Agent tools that support AGENTS.md will automatically");
            println!("load these instructions when working in this directory.");
            println!();
            println!("Try running: qipu prime");
        },
        records => {
            print_records_header(
                "setup.install",
                &[("integration", "agents-md"), ("status", "installed")],
            );
            print_records_data("path", "AGENTS.md");
        }
    )
}

/// Install Cursor rules integration
fn execute_install_cursor(cli: &Cli) -> Result<(), QipuError> {
    let cursor_rules_path = get_cursor_rules_path(cli);

    // Create .cursor/rules directory if it doesn't exist
    let rules_dir = cursor_rules_path.parent().unwrap();
    if !rules_dir.exists() {
        std::fs::create_dir_all(rules_dir)?;
    }

    if cursor_rules_path.exists() {
        return output_by_format!(cli.format,
            json => {
                print_json_status(
                    "exists",
                    Some("Cursor rules already exist. Use `cat .cursor/rules/qipu.mdc` to see the current content."),
                    &[("path", serde_json::json!(".cursor/rules/qipu.mdc"))],
                )
            },
            human => {
                println!("Cursor rules already exist (.cursor/rules/qipu.mdc).");
                println!();
                println!("To see the content, run:");
                println!("  cat .cursor/rules/qipu.mdc");
                println!();
                println!("To update, manually edit the file or remove it and reinstall.");
                Ok(())
            },
            records => {
                print_records_header("setup.install", &[("integration", "cursor"), ("status", "exists")]);
                print_records_data("message", "Cursor rules already exist (.cursor/rules/qipu.mdc).");
                Ok(())
            }
        );
    }

    std::fs::write(&cursor_rules_path, CURSOR_RULES_CONTENT)?;

    output_by_format_result!(cli.format,
        json => print_json_status(
            "installed",
            Some("Cursor rules created successfully"),
            &[("path", serde_json::json!(".cursor/rules/qipu.mdc"))],
        ),
        human => {
            println!("✓ Created .cursor/rules/qipu.mdc");
            println!();
            println!("Integration complete! Cursor will automatically apply these rules");
            println!("when working in this directory.");
            println!();
            println!("Try running: qipu prime");
        },
        records => {
            print_records_header(
                "setup.install",
                &[("integration", "cursor"), ("status", "installed")],
            );
            print_records_data("path", ".cursor/rules/qipu.mdc");
        }
    )
}

/// Check if AGENTS.md integration is installed
fn execute_check_agents_md(cli: &Cli) -> Result<(), QipuError> {
    let agents_md_path = get_agents_md_path(cli);
    let exists = agents_md_path.exists();

    output_by_format_result!(cli.format,
        json => {
            let output = serde_json::json!({
                "integration": "agents-md",
                "installed": exists,
                "path": if exists { Some(agents_md_path.display().to_string()) } else { None }
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
            Ok(())
        },
        human => {
            if exists {
                println!("✓ AGENTS.md integration is installed");
                println!("  Path: AGENTS.md");
            } else {
                println!("✗ AGENTS.md integration is not installed");
                println!();
                println!("Run `qipu setup agents-md` to install.");
            }
        },
        records => {
            let status = if exists { "installed" } else { "not-installed" };
            print_records_header(
                "setup.check",
                &[("integration", "agents-md"), ("status", status)],
            );
            if exists {
                print_records_data("path", "AGENTS.md");
            }
        }
    )
}

/// Check if Cursor rules integration is installed
fn execute_check_cursor(cli: &Cli) -> Result<(), QipuError> {
    let cursor_rules_path = get_cursor_rules_path(cli);
    let exists = cursor_rules_path.exists();

    output_by_format_result!(cli.format,
        json => {
            let output = serde_json::json!({
                "integration": "cursor",
                "installed": exists,
                "path": if exists { Some(cursor_rules_path.display().to_string()) } else { None }
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
            Ok(())
        },
        human => {
            if exists {
                println!("✓ Cursor rules integration is installed");
                println!("  Path: .cursor/rules/qipu.mdc");
            } else {
                println!("✗ Cursor rules integration is not installed");
                println!();
                println!("Run `qipu setup cursor` to install.");
            }
        },
        records => {
            let status = if exists { "installed" } else { "not-installed" };
            print_records_header(
                "setup.check",
                &[("integration", "cursor"), ("status", status)],
            );
            if exists {
                print_records_data("path", ".cursor/rules/qipu.mdc");
            }
        }
    )
}

/// Remove AGENTS.md integration
fn execute_remove_agents_md(cli: &Cli) -> Result<(), QipuError> {
    let agents_md_path = get_agents_md_path(cli);

    if !agents_md_path.exists() {
        return output_by_format!(cli.format,
            json => {
                print_json_status("not-found", Some("AGENTS.md does not exist"), &[])
            },
            human => {
                println!("AGENTS.md does not exist (nothing to remove).");
                Ok(())
            },
            records => {
                print_records_header(
                    "setup.remove",
                    &[("integration", "agents-md"), ("status", "not-found")],
                );
                Ok(())
            }
        );
    }

    // Remove AGENTS.md
    std::fs::remove_file(agents_md_path)?;

    output_by_format_result!(cli.format,
        json => print_json_status("removed", Some("AGENTS.md removed successfully"), &[]),
        human => {
            println!("✓ Removed AGENTS.md");
        },
        records => {
            print_records_header(
                "setup.remove",
                &[("integration", "agents-md"), ("status", "removed")],
            );
        }
    )
}

/// Remove Cursor rules integration
fn execute_remove_cursor(cli: &Cli) -> Result<(), QipuError> {
    let cursor_rules_path = get_cursor_rules_path(cli);

    if !cursor_rules_path.exists() {
        return output_by_format!(cli.format,
            json => {
                print_json_status("not-found", Some("Cursor rules do not exist"), &[])
            },
            human => {
                println!("Cursor rules do not exist (nothing to remove).");
                Ok(())
            },
            records => {
                print_records_header(
                    "setup.remove",
                    &[("integration", "cursor"), ("status", "not-found")],
                );
                Ok(())
            }
        );
    }

    // Remove cursor rules file
    std::fs::remove_file(&cursor_rules_path)?;

    output_by_format_result!(cli.format,
        json => print_json_status("removed", Some("Cursor rules removed successfully"), &[]),
        human => {
            println!("✓ Removed .cursor/rules/qipu.mdc");
        },
        records => {
            print_records_header(
                "setup.remove",
                &[("integration", "cursor"), ("status", "removed")],
            );
        }
    )
}

#[cfg(test)]
mod tests;
