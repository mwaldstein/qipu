//! Command dispatch logic for qipu

use std::time::Instant;

use crate::cli::paths::resolve_root_path;
use crate::cli::Cli;
use qipu_core::error::Result;
use qipu_core::store::paths::WORKSPACE_FILE;
use qipu_core::store::Store;
use qipu_core::telemetry::{
    get_app_version, init_telemetry, record_command_execution, CommandName,
};
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
    let root = resolve_root_path(cli.root.as_deref());

    debug!(elapsed = ?start.elapsed(), "resolve_root");

    let ctx = CommandContext::new(cli, &root, start);

    // Initialize telemetry
    let telemetry = init_telemetry();

    // Record session stats if telemetry is enabled and store exists
    if telemetry.is_enabled() {
        if let Ok(store) = Store::discover(&root) {
            record_session_stats(&telemetry, &store);
        }
    }

    // Execute command and record telemetry
    match &cli.command {
        None => {
            let result = NoCommand.execute(&ctx);
            record_command_execution(&telemetry, CommandName::List, &result, start);
            result
        }
        Some(cmd) => {
            let command_name = command_to_name(cmd);
            let result = cmd.execute(&ctx);
            record_command_execution(&telemetry, command_name, &result, start);
            result
        }
    }
}

fn record_session_stats(
    telemetry: &std::sync::Arc<qipu_core::telemetry::TelemetryCollector>,
    store: &Store,
) {
    // Count workspaces: primary (1) + valid workspace subdirectories
    let workspaces_dir = store.workspaces_dir();
    let mut workspace_count: usize = 1; // Primary workspace

    if workspaces_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&workspaces_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // A valid workspace has a workspace.toml file
                    if path.join(WORKSPACE_FILE).exists() {
                        workspace_count += 1;
                    }
                }
            }
        }
    }

    // Get note count from database
    let note_count = store.db().get_note_count().unwrap_or(0) as usize;

    telemetry.record_session_stats(workspace_count, note_count, get_app_version());
}

fn command_to_name(cmd: &crate::cli::Commands) -> CommandName {
    use crate::cli::Commands;
    match cmd {
        Commands::Init(_) => CommandName::Init,
        Commands::Create(_) => CommandName::Create,
        Commands::New(_) => CommandName::New,
        Commands::List(_) => CommandName::List,
        Commands::Show(_) => CommandName::Show,
        Commands::Inbox(_) => CommandName::Inbox,
        Commands::Capture(_) => CommandName::Capture,
        Commands::Index(_) => CommandName::Index,
        Commands::Search(_) => CommandName::Search,
        Commands::Edit(_) => CommandName::Edit,
        Commands::Update(_) => CommandName::Update,
        Commands::Context(_) => CommandName::Context,
        Commands::Dump(_) => CommandName::Dump,
        Commands::Export(_) => CommandName::Export,
        Commands::Load(_) => CommandName::Load,
        Commands::Prime(_) => CommandName::Prime,
        Commands::Verify(_) => CommandName::Verify,
        Commands::Value(_) => CommandName::Value,
        Commands::Tags(_) => CommandName::Tags,
        Commands::Custom(_) => CommandName::Custom,
        Commands::Link(_) => CommandName::Link,
        Commands::Onboard => CommandName::Onboard,
        Commands::Setup(_) => CommandName::Setup,
        Commands::Doctor(_) => CommandName::Doctor,
        Commands::Sync(_) => CommandName::Sync,
        Commands::Compact(_) => CommandName::Compact,
        Commands::Workspace(_) => CommandName::Workspace,
        Commands::Merge(_) => CommandName::Merge,
        Commands::Store(_) => CommandName::Store,
        Commands::Ontology(_) => CommandName::Ontology,
        Commands::Telemetry(_) => CommandName::Telemetry,
        Commands::Hooks(_) => CommandName::Hooks,
    }
}
