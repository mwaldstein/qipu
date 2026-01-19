//! Handlers for link-related commands

use std::path::PathBuf;
use std::time::Instant;

use tracing::debug;

use crate::cli::{Cli, LinkCommands};
use crate::commands;
use crate::lib::error::{QipuError, Result};

use super::discover_or_open_store;

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
                .parse::<crate::lib::graph::Direction>()
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
            )
        }
        LinkCommands::Add { from, to, r#type } => {
            commands::link::add::execute(cli, &store, from, to, r#type.clone())
        }
        LinkCommands::Remove { from, to, r#type } => {
            commands::link::remove::execute(cli, &store, from, to, r#type.clone())
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
        } => {
            let dir = direction
                .parse::<crate::lib::graph::Direction>()
                .map_err(|e| {
                    QipuError::UsageError(format!("invalid --direction '{}': {}", direction, e))
                })?;
            let opts = crate::lib::graph::TreeOptions {
                direction: dir,
                max_hops: crate::lib::graph::HopCost::from(*max_hops),
                type_include: r#type.clone(),
                type_exclude: exclude_type.clone(),
                typed_only: *typed_only,
                inline_only: *inline_only,
                max_nodes: *max_nodes,
                max_edges: *max_edges,
                max_fanout: *max_fanout,
                max_chars: *max_chars,
                semantic_inversion: true,
            };
            commands::link::tree::execute(cli, &store, id_or_path, opts)
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
        } => {
            let dir = direction
                .parse::<crate::lib::graph::Direction>()
                .map_err(|e| {
                    QipuError::UsageError(format!("invalid --direction '{}': {}", direction, e))
                })?;
            let opts = crate::lib::graph::TreeOptions {
                direction: dir,
                max_hops: crate::lib::graph::HopCost::from(*max_hops),
                type_include: r#type.clone(),
                type_exclude: exclude_type.clone(),
                typed_only: *typed_only,
                inline_only: *inline_only,
                max_nodes: None,
                max_edges: None,
                max_fanout: None,
                max_chars: *max_chars,
                semantic_inversion: true,
            };
            commands::link::path::execute(cli, &store, from, to, opts)
        }
    }
}
