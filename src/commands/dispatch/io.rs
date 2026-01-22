//! Handlers for I/O commands (export, dump, load)

#![allow(clippy::ptr_arg)]

use std::path::PathBuf;
use std::time::Instant;

use tracing::debug;

use crate::cli::Cli;
use crate::commands;
use crate::lib::error::{QipuError, Result};

use super::discover_or_open_store;

#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_export(
    cli: &Cli,
    root: &PathBuf,
    note_ids: &[String],
    tag: Option<&str>,
    moc_id: Option<&str>,
    query: Option<&str>,
    output: Option<&PathBuf>,
    mode: &str,
    with_attachments: bool,
    link_mode: &str,
    bib_format: &str,
    max_hops: u32,
    pdf: bool,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "discover_store");
    }
    let export_mode = commands::export::ExportMode::parse(mode)?;
    let link_mode = commands::export::LinkMode::parse(link_mode)?;
    let bib_format = commands::export::emit::bibliography::BibFormat::parse(bib_format)?;
    commands::export::execute(
        cli,
        &store,
        commands::export::ExportOptions {
            note_ids,
            tag,
            moc_id,
            query,
            output: output.map(|p| p.as_path()),
            mode: export_mode,
            with_attachments,
            link_mode,
            bib_format,
            max_hops,
            pdf,
        },
    )?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "execute_command");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_dump(
    cli: &Cli,
    root: &PathBuf,
    file: Option<&PathBuf>,
    note_ids: &[String],
    tag: Option<&str>,
    moc_id: Option<&str>,
    query: Option<&str>,
    direction: &str,
    max_hops: u32,
    type_include: Vec<String>,
    typed_only: bool,
    inline_only: bool,
    no_attachments: bool,
    output: Option<&PathBuf>,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "discover_store");
    }

    let dir = direction
        .parse::<commands::link::Direction>()
        .map_err(QipuError::Other)?;

    let resolved_output = match (file, output) {
        (Some(_), Some(_)) => {
            return Err(QipuError::Other(
                "both positional file and --output were provided; use one".to_string(),
            ))
        }
        (Some(file_path), None) => Some(file_path.as_path()),
        (None, Some(output_path)) => Some(output_path.as_path()),
        (None, None) => None,
    };

    commands::dump::execute(
        cli,
        &store,
        commands::dump::DumpOptions {
            note_ids,
            tag,
            moc_id,
            query,
            direction: dir,
            max_hops,
            type_include,
            typed_only,
            inline_only,
            include_attachments: !no_attachments,
            output: resolved_output,
        },
    )?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "execute_command");
    }
    Ok(())
}

pub(super) fn handle_load(
    cli: &Cli,
    root: &PathBuf,
    pack_file: &PathBuf,
    strategy: &str,
    start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "discover_store");
    }
    commands::load::execute(cli, &store, pack_file, strategy)?;
    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "execute_command");
    }
    Ok(())
}
