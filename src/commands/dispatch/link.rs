//! Handlers for link-related commands

use std::path::PathBuf;
use std::time::Instant;

use tracing::debug;

use crate::cli::{Cli, LinkCommands};
use crate::commands;
use qipu_core::error::{QipuError, Result};

use super::command::discover_or_open_store;

pub(super) fn handle_link(
    cli: &Cli,
    root: &PathBuf,
    command: &LinkCommands,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "discover_store");
    }

    match command {
        LinkCommands::List {
            id_or_path,
            direction,
            r#type,
            typed_only,
            inline_only,
            max_chars,
        } => {
            let dir = direction
                .parse::<qipu_core::graph::Direction>()
                .map_err(QipuError::Other)?;
            commands::link::list::execute(
                cli,
                &store,
                id_or_path,
                dir,
                r#type.as_deref(),
                *typed_only,
                *inline_only,
                *max_chars,
            )?;
            if cli.verbose {
                debug!(elapsed = ?start.elapsed(), "execute_command");
            }
            Ok(())
        }
        LinkCommands::Add { from, to, r#type } => {
            commands::link::add::execute(cli, &store, from, to, r#type.clone())?;
            if cli.verbose {
                debug!(elapsed = ?start.elapsed(), "execute_command");
            }
            Ok(())
        }
        LinkCommands::Remove { from, to, r#type } => {
            commands::link::remove::execute(cli, &store, from, to, r#type.clone())?;
            if cli.verbose {
                debug!(elapsed = ?start.elapsed(), "execute_command");
            }
            Ok(())
        }
        LinkCommands::Tree {
            id_or_path,
            direction,
            max_hops,
            r#type,
            exclude_type,
            typed_only,
            inline_only,
            max_nodes,
            max_edges,
            max_fanout,
            max_chars,
            min_value,
            ignore_value,
        } => {
            let dir = direction
                .parse::<qipu_core::graph::Direction>()
                .map_err(|e| {
                    QipuError::UsageError(format!("invalid --direction '{}': {}", direction, e))
                })?;
            let opts = qipu_core::graph::TreeOptions {
                direction: dir,
                max_hops: qipu_core::graph::HopCost::from(*max_hops),
                type_include: r#type.clone(),
                type_exclude: exclude_type.clone(),
                typed_only: *typed_only,
                inline_only: *inline_only,
                max_nodes: *max_nodes,
                max_edges: *max_edges,
                max_fanout: *max_fanout,
                max_chars: *max_chars,
                semantic_inversion: true,
                min_value: *min_value,
                ignore_value: *ignore_value,
            };
            commands::link::tree::execute(cli, &store, id_or_path, opts)?;
            if cli.verbose {
                debug!(elapsed = ?start.elapsed(), "execute_command");
            }
            Ok(())
        }
        LinkCommands::Path {
            from,
            to,
            direction,
            max_hops,
            r#type,
            exclude_type,
            typed_only,
            inline_only,
            max_chars,
            min_value,
            ignore_value,
        } => {
            let dir = direction
                .parse::<qipu_core::graph::Direction>()
                .map_err(|e| {
                    QipuError::UsageError(format!("invalid --direction '{}': {}", direction, e))
                })?;
            let opts = qipu_core::graph::TreeOptions {
                direction: dir,
                max_hops: qipu_core::graph::HopCost::from(*max_hops),
                type_include: r#type.clone(),
                type_exclude: exclude_type.clone(),
                typed_only: *typed_only,
                inline_only: *inline_only,
                max_nodes: None,
                max_edges: None,
                max_fanout: None,
                max_chars: *max_chars,
                semantic_inversion: true,
                min_value: *min_value,
                ignore_value: *ignore_value,
            };
            commands::link::path::execute(cli, &store, from, to, opts)?;
            if cli.verbose {
                debug!(elapsed = ?start.elapsed(), "execute_command");
            }
            Ok(())
        }
    }
}
