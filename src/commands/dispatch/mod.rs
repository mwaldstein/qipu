//! Command dispatch logic for qipu

use std::time::Instant;

use crate::cli::paths::resolve_root_path;
use crate::cli::Cli;
use qipu_core::error::Result;
use tracing::debug;

pub mod command;
mod commands;
pub mod handlers;
mod helpers;
mod io;
mod link;
#[macro_use]
mod macros;
mod maintenance;
mod notes;

pub(crate) use macros::{trace_command, trace_command_always};

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
