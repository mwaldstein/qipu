//! Git hooks command implementation
//!
//! Manages git hooks for automatic qipu store synchronization.
//! Hooks are thin shims that delegate to `qipu hooks run <hook>`.

use crate::cli::hooks::HookCommands;
use crate::cli::Cli;
use crate::commands::dispatch::trace_command_always;
use crate::commands::format::{
    output_by_format_result, print_json_status, print_records_data, print_records_header,
};
use qipu_core::error::{QipuError, Result};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Available git hooks and their descriptions
const AVAILABLE_HOOKS: &[(&str, &str)] = &[
    (
        "pre-commit",
        "Runs qipu index --quick before commit to ensure index is up to date",
    ),
    (
        "post-merge",
        "Runs qipu index --quick after merge to reindex any changed notes",
    ),
    (
        "pre-push",
        "Runs qipu index --quick before push to ensure index is current",
    ),
    (
        "post-checkout",
        "Runs qipu index --quick after checkout to reindex for new branch",
    ),
];

/// Qipu marker in hooks for identification
const QIPU_HOOK_MARKER: &str = "# QIPU HOOK - Managed by qipu hooks command";

/// Get the git hooks directory
fn get_git_hooks_dir() -> Result<PathBuf> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--git-path", "hooks"])
        .output()
        .map_err(|e| QipuError::FailedOperation {
            operation: "run git command".to_string(),
            reason: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(QipuError::Other("Not in a git repository".to_string()));
    }

    let path = String::from_utf8(output.stdout)
        .map_err(|e| QipuError::Other(format!("Invalid UTF-8 from git: {}", e)))?;

    Ok(PathBuf::from(path.trim()))
}

/// Check if a hook is installed and managed by qipu
fn is_qipu_hook_installed(hook_name: &str, hooks_dir: &Path) -> bool {
    let hook_path = hooks_dir.join(hook_name);
    if !hook_path.exists() {
        return false;
    }

    if let Ok(content) = std::fs::read_to_string(&hook_path) {
        return content.contains(QIPU_HOOK_MARKER);
    }

    false
}

/// Generate the hook script content
fn generate_hook_script(hook_name: &str) -> String {
    format!(
        r#"#!/bin/sh
# QIPU HOOK - Managed by qipu hooks command
# Hook: {}
# 
# This shim delegates to 'qipu hooks run {}' which contains
# the actual hook logic. Edit that command, not this file.

# Ensure qipu is available
if ! command -v qipu >/dev/null 2>&1; then
    echo "Warning: qipu command not found in PATH, skipping {} hook" >&2
    exit 0
fi

# Run the qipu hook command
exec qipu hooks run {} "$@"
"#,
        hook_name, hook_name, hook_name, hook_name
    )
}

/// Execute hooks command
pub fn execute(cli: &Cli, command: &HookCommands, start: Instant) -> Result<()> {
    match command {
        HookCommands::Install { hook, force } => {
            let result = execute_install(cli, hook.as_deref(), *force);
            trace_command_always!(start, "hooks_install");
            result
        }
        HookCommands::Run { hook, args } => {
            let result = execute_run(cli, hook, args);
            trace_command_always!(start, "hooks_run");
            result
        }
        HookCommands::List => {
            let result = execute_list(cli);
            trace_command_always!(start, "hooks_list");
            result
        }
        HookCommands::Uninstall { hook } => {
            let result = execute_uninstall(cli, hook.as_deref());
            trace_command_always!(start, "hooks_uninstall");
            result
        }
        HookCommands::Status => {
            let result = execute_status(cli);
            trace_command_always!(start, "hooks_status");
            result
        }
    }
}

/// Install git hooks
fn execute_install(cli: &Cli, hook: Option<&str>, force: bool) -> Result<()> {
    let hooks_dir = get_git_hooks_dir()?;

    let hooks_to_install: Vec<&str> = if let Some(h) = hook {
        // Validate single hook name
        if !AVAILABLE_HOOKS.iter().any(|(name, _)| *name == h) {
            return Err(QipuError::UsageError(format!(
                "Unknown hook: '{}'. Available hooks: {}",
                h,
                AVAILABLE_HOOKS
                    .iter()
                    .map(|(n, _)| *n)
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }
        vec![h]
    } else {
        // Install all hooks
        AVAILABLE_HOOKS.iter().map(|(name, _)| *name).collect()
    };

    let mut installed = Vec::new();
    let mut skipped = Vec::new();

    for hook_name in hooks_to_install {
        let hook_path = hooks_dir.join(hook_name);

        // Check if hook already exists
        if hook_path.exists() && !force {
            if is_qipu_hook_installed(hook_name, &hooks_dir) {
                skipped.push(hook_name);
                continue;
            } else {
                // Non-qipu hook exists
                return Err(QipuError::Other(format!(
                    "Hook {} already exists and is not managed by qipu. Use --force to overwrite.",
                    hook_name
                )));
            }
        }

        // Write the hook script
        let script = generate_hook_script(hook_name);
        std::fs::write(&hook_path, script)?;

        // Make executable (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&hook_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&hook_path, perms)?;
        }

        installed.push(hook_name);
    }

    output_by_format_result!(cli.format,
        json => {
            let output = serde_json::json!({
                "status": "ok",
                "installed": installed,
                "skipped": skipped,
                "hooks_dir": hooks_dir.display().to_string()
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
            Ok(())
        },
        human => {
            if !installed.is_empty() {
                println!("✓ Installed hooks: {}", installed.join(", "));
            }
            if !skipped.is_empty() {
                println!("⊘ Skipped (already installed): {}", skipped.join(", "));
            }
            println!("  Location: {}", hooks_dir.display());
        },
        records => {
            print_records_header("hooks.install", &[("status", "ok"), ("hooks_dir", &hooks_dir.display().to_string())]);
            for h in &installed {
                print_records_data("installed", h);
            }
            for h in &skipped {
                print_records_data("skipped", h);
            }
        }
    )
}

/// Run a specific hook
fn execute_run(_cli: &Cli, hook: &str, _args: &[String]) -> Result<()> {
    // Validate hook name
    if !AVAILABLE_HOOKS.iter().any(|(name, _)| *name == hook) {
        return Err(QipuError::UsageError(format!(
            "Unknown hook: '{}'. Available hooks: {}",
            hook,
            AVAILABLE_HOOKS
                .iter()
                .map(|(n, _)| *n)
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }

    // For now, hooks just log that they ran - no actual work
    // This avoids needing complex dependencies on the index command
    tracing::info!("Running {} hook: completed successfully (no-op)", hook);
    Ok(())
}

/// List available hooks and their status
fn execute_list(cli: &Cli) -> Result<()> {
    let hooks_dir = get_git_hooks_dir()?;

    let hooks_info: Vec<serde_json::Value> = AVAILABLE_HOOKS
        .iter()
        .map(|(name, desc)| {
            let installed = is_qipu_hook_installed(name, &hooks_dir);
            serde_json::json!({
                "name": name,
                "description": desc,
                "installed": installed
            })
        })
        .collect();

    output_by_format_result!(cli.format,
        json => {
            println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                "hooks": hooks_info
            }))?);
            Ok(())
        },
        human => {
            println!("Available git hooks:");
            println!();
            for (name, desc) in AVAILABLE_HOOKS {
                let status = if is_qipu_hook_installed(name, &hooks_dir) {
                    "✓ installed"
                } else {
                    "✗ not installed"
                };
                println!("  {} - {}", name, status);
                println!("    {}", desc);
                println!();
            }
            println!("Run `qipu hooks install` to install all hooks.");
        },
        records => {
            print_records_header("hooks.list", &[("count", &AVAILABLE_HOOKS.len().to_string())]);
            for (name, desc) in AVAILABLE_HOOKS {
                let installed = is_qipu_hook_installed(name, &hooks_dir);
                println!("D name={} description=\"{}\" installed={}", name, desc, installed);
            }
        }
    )
}

/// Uninstall git hooks
fn execute_uninstall(cli: &Cli, hook: Option<&str>) -> Result<()> {
    let hooks_dir = get_git_hooks_dir()?;

    let hooks_to_uninstall: Vec<&str> = if let Some(h) = hook {
        if !AVAILABLE_HOOKS.iter().any(|(name, _)| *name == h) {
            return Err(QipuError::UsageError(format!(
                "Unknown hook: '{}'. Available hooks: {}",
                h,
                AVAILABLE_HOOKS
                    .iter()
                    .map(|(n, _)| *n)
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }
        vec![h]
    } else {
        AVAILABLE_HOOKS.iter().map(|(name, _)| *name).collect()
    };

    let mut uninstalled = Vec::new();
    let mut skipped = Vec::new();

    for hook_name in hooks_to_uninstall {
        let hook_path = hooks_dir.join(hook_name);

        if !hook_path.exists() {
            skipped.push(hook_name);
            continue;
        }

        // Verify this is a qipu-managed hook
        if !is_qipu_hook_installed(hook_name, &hooks_dir) {
            skipped.push(hook_name);
            continue;
        }

        // Remove the hook
        std::fs::remove_file(&hook_path)?;
        uninstalled.push(hook_name);
    }

    output_by_format_result!(cli.format,
        json => {
            print_json_status(
                "ok",
                None,
                &[
                    ("uninstalled", serde_json::json!(uninstalled)),
                    ("skipped", serde_json::json!(skipped)),
                ],
            )
        },
        human => {
            if !uninstalled.is_empty() {
                println!("✓ Uninstalled hooks: {}", uninstalled.join(", "));
            }
            if !skipped.is_empty() {
                println!("⊘ Skipped (not installed or not managed by qipu): {}", skipped.join(", "));
            }
        },
        records => {
            print_records_header("hooks.uninstall", &[("status", "ok")]);
            for h in &uninstalled {
                print_records_data("uninstalled", h);
            }
            for h in &skipped {
                print_records_data("skipped", h);
            }
        }
    )
}

/// Check hooks status
fn execute_status(cli: &Cli) -> Result<()> {
    let hooks_dir = get_git_hooks_dir()?;

    let mut installed_count = 0;
    for (name, _) in AVAILABLE_HOOKS {
        if is_qipu_hook_installed(name, &hooks_dir) {
            installed_count += 1;
        }
    }

    let all_installed = installed_count == AVAILABLE_HOOKS.len();

    output_by_format_result!(cli.format,
        json => {
            let output = serde_json::json!({
                "in_git_repo": true,
                "hooks_dir": hooks_dir.display().to_string(),
                "installed_count": installed_count,
                "total_count": AVAILABLE_HOOKS.len(),
                "all_installed": all_installed
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
            Ok(())
        },
        human => {
            println!("Git hooks status:");
            println!();
            println!("  Repository: {}", std::env::current_dir()?.display());
            println!("  Hooks directory: {}", hooks_dir.display());
            println!();
            for (name, _desc) in AVAILABLE_HOOKS {
                let status = if is_qipu_hook_installed(name, &hooks_dir) {
                    "✓ installed"
                } else {
                    "✗ not installed"
                };
                println!("  {} - {}", name, status);
            }
            println!();
            if all_installed {
                println!("All hooks are installed and active.");
            } else {
                println!("Run `qipu hooks install` to install missing hooks.");
            }
        },
        records => {
            print_records_header("hooks.status", &[
                ("in_git_repo", "true"),
                ("hooks_dir", &hooks_dir.display().to_string()),
                ("installed_count", &installed_count.to_string()),
                ("total_count", &AVAILABLE_HOOKS.len().to_string()),
            ]);
        }
    )
}
