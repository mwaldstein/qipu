//! Command trait and context for dispatching commands

use std::path::Path;
use std::time::Instant;

use crate::cli::Cli;
use qipu_core::error::Result;
use qipu_core::store::Store;

/// Discover or open a store based on CLI configuration
pub fn discover_or_open_store(cli: &Cli, root: &Path) -> Result<Store> {
    let base_store = if let Some(path) = &cli.store {
        let resolved = if path.is_absolute() {
            path.clone()
        } else {
            root.join(path)
        };
        Store::open(&resolved)?
    } else {
        Store::discover(root)?
    };

    if let Some(workspace_name) = &cli.workspace {
        let workspace_path = base_store.workspaces_dir().join(workspace_name);
        Store::open(&workspace_path)
    } else {
        Ok(base_store)
    }
}

/// Shared context for command execution
pub struct CommandContext<'a> {
    pub cli: &'a Cli,
    pub root: &'a Path,
    pub start: Instant,
}

impl<'a> CommandContext<'a> {
    pub fn new(cli: &'a Cli, root: &'a Path, start: Instant) -> Self {
        Self { cli, root, start }
    }
}

/// Trait for commands that can be executed
pub trait Command {
    fn execute(&self, ctx: &CommandContext) -> Result<()>;
}

/// No-op command (when no subcommand is provided)
pub struct NoCommand;

impl Command for NoCommand {
    fn execute(&self, _ctx: &CommandContext) -> Result<()> {
        println!("qipu {}", env!("CARGO_PKG_VERSION"));
        println!();
        println!("A Zettelkasten-inspired knowledge management CLI.");
        println!();
        println!("Run `qipu --help` for usage information.");
        Ok(())
    }
}
