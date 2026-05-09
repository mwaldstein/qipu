//! Handlers for link-related commands

use std::path::Path;
use std::time::Instant;

use crate::cli::{Cli, LinkCommands};
use crate::commands;
use qipu_core::error::{QipuError, Result};
use qipu_core::note::LinkType;
use qipu_core::store::Store;

use super::command::discover_or_open_store;
use super::trace_command;

pub(super) fn handle_link(
    cli: &Cli,
    root: &Path,
    command: &LinkCommands,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    trace_command!(cli, start, "discover_store");

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
            trace_command!(cli, start, "execute_command");
            Ok(())
        }
        LinkCommands::Add { from, to, r#type } => {
            commands::link::add::execute(cli, &store, from, to, r#type.clone())?;
            trace_command!(cli, start, "execute_command");
            Ok(())
        }
        LinkCommands::Remove { from, to, r#type } => {
            commands::link::remove::execute(cli, &store, from, to, r#type.clone())?;
            trace_command!(cli, start, "execute_command");
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
                type_include: r#type,
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
            trace_command!(cli, start, "execute_command");
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
                type_include: r#type,
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
            commands::link::path::execute(cli, &store, from, to, opts)?;
            trace_command!(cli, start, "execute_command");
            Ok(())
        }
        LinkCommands::Materialize {
            id_or_path,
            r#type,
            dry_run,
            remove_inline,
        } => {
            commands::link::materialize::execute(
                cli,
                &store,
                id_or_path,
                r#type.clone(),
                *dry_run,
                *remove_inline,
            )?;
            trace_command!(cli, start, "execute_command");
            Ok(())
        }
        LinkCommands::External(args) => {
            let (from, to, link_type) = parse_hidden_link_add(&store, args)?;
            commands::link::add::execute(cli, &store, &from, &to, link_type)?;
            trace_command!(cli, start, "execute_command");
            Ok(())
        }
    }
}

fn parse_hidden_link_add(
    store: &Store,
    args: &[std::ffi::OsString],
) -> Result<(String, String, LinkType)> {
    let values = args
        .iter()
        .map(|arg| {
            arg.to_str().ok_or_else(|| {
                QipuError::UsageError(
                    "link shorthand arguments must be valid UTF-8; use `qipu link add <from> <to> --type <type>`".to_string(),
                )
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let canonical = "Use: qipu link add <from> <to> --type <type>";
    if values.len() != 4 || !matches!(values[2], "--type" | "-T") {
        return Err(QipuError::UsageError(format!(
            "unrecognized link command\n\n{}\nRun `qipu link --help` for full and advanced details.",
            canonical
        )));
    }

    if !looks_like_generated_note_pair(values[0], values[1])
        && !looks_like_existing_note_pair(store, values[0], values[1])
    {
        return Err(QipuError::UsageError(format!(
            "unrecognized link command\n\n{}\nRun `qipu link --help` for full and advanced details.",
            canonical
        )));
    }

    let link_type = values[3].parse::<LinkType>().map_err(|err| {
        QipuError::UsageError(format!(
            "invalid --type '{}': {}\n\n{}",
            values[3], err, canonical
        ))
    })?;

    Ok((values[0].to_string(), values[1].to_string(), link_type))
}

fn looks_like_generated_note_pair(from: &str, to: &str) -> bool {
    from.starts_with("qp-") && to.starts_with("qp-")
}

fn looks_like_existing_note_pair(store: &Store, from: &str, to: &str) -> bool {
    store.note_exists(from) && store.note_exists(to)
}
