//! Command dispatch logic for qipu

use std::time::Instant;

use crate::cli::paths::resolve_root_path;
use crate::cli::Cli;
use qipu_core::error::Result;
use tracing::debug;

mod command;
mod commands;
pub mod handlers;
mod io;
mod link;
mod maintenance;
mod notes;

use command::{Command, CommandContext, NoCommand};

pub fn run(cli: &Cli, start: Instant) -> Result<()> {
    // Determine the root directory
    let root = resolve_root_path(cli.root.clone());

    debug!(elapsed = ?start.elapsed(), "resolve_root");

    let ctx = CommandContext::new(cli, &root, start);

    // Execute command
    match &cli.command {
        None => NoCommand.execute(&ctx),
        Some(cmd) => cmd.execute(&ctx),
    }
}
