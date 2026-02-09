//! Git hooks integration for qipu
//!
//! Provides commands to install and manage git hooks that keep
//! the qipu store and index in sync with git operations.

use clap::{Args, Subcommand};

/// Git hook types supported by qipu
#[derive(Subcommand, Debug, Clone)]
pub enum HookCommands {
    /// Install git hooks in the current repository
    Install {
        /// Specific hook to install (installs all if omitted)
        #[arg(value_name = "HOOK")]
        hook: Option<String>,
        /// Force overwrite existing hooks
        #[arg(long)]
        force: bool,
    },
    /// Run a specific hook (called by git hooks)
    Run {
        /// Hook name to run
        #[arg(value_name = "HOOK")]
        hook: String,
        /// Additional arguments passed by git
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// List available hooks and their status
    List,
    /// Uninstall qipu git hooks
    Uninstall {
        /// Specific hook to uninstall (removes all if omitted)
        #[arg(value_name = "HOOK")]
        hook: Option<String>,
    },
    /// Check if hooks are installed and working
    Status,
}

/// Subcommand wrapper for hooks
#[derive(Args, Debug)]
pub struct HooksSubcommand {
    #[command(subcommand)]
    pub command: HookCommands,
}
