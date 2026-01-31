//! Handlers for I/O commands (export, dump, load)

use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::cli::Cli;
use crate::commands;
use qipu_core::error::{QipuError, Result};

use super::command::discover_or_open_store;
use super::trace_command;

/// Parameters for the export command handler
pub struct ExportParams<'a> {
    pub cli: &'a Cli,
    pub root: &'a Path,
    pub note_ids: &'a [String],
    pub tag: Option<&'a str>,
    pub moc_id: Option<&'a str>,
    pub query: Option<&'a str>,
    pub output: Option<&'a PathBuf>,
    pub mode: &'a str,
    pub with_attachments: bool,
    pub link_mode: &'a str,
    pub bib_format: &'a str,
    pub max_hops: u32,
    pub pdf: bool,
    pub start: Instant,
}

pub(super) fn handle_export(params: ExportParams) -> Result<()> {
    let store = discover_or_open_store(params.cli, params.root)?;
    trace_command!(params.cli, params.start, "discover_store");
    let export_mode = commands::export::ExportMode::parse(params.mode)?;
    let link_mode = commands::export::LinkMode::parse(params.link_mode)?;
    let bib_format = commands::export::emit::bibliography::BibFormat::parse(params.bib_format)?;
    commands::export::execute(
        params.cli,
        &store,
        commands::export::ExportOptions {
            note_ids: params.note_ids,
            tag: params.tag,
            moc_id: params.moc_id,
            query: params.query,
            output: params.output.map(|p| p.as_path()),
            mode: export_mode,
            with_attachments: params.with_attachments,
            link_mode,
            bib_format,
            max_hops: params.max_hops,
            pdf: params.pdf,
        },
    )?;
    trace_command!(params.cli, params.start, "execute_command");
    Ok(())
}

/// Parameters for the dump command handler
pub struct DumpParams<'a> {
    pub cli: &'a Cli,
    pub root: &'a Path,
    pub file: Option<&'a PathBuf>,
    pub note_ids: &'a [String],
    pub tag: Option<&'a str>,
    pub moc_id: Option<&'a str>,
    pub query: Option<&'a str>,
    pub direction: &'a str,
    pub max_hops: u32,
    pub type_include: &'a [String],
    pub typed_only: bool,
    pub inline_only: bool,
    pub no_attachments: bool,
    pub output: Option<&'a PathBuf>,
    pub start: Instant,
}

pub(super) fn handle_dump(params: DumpParams) -> Result<()> {
    let store = discover_or_open_store(params.cli, params.root)?;
    trace_command!(params.cli, params.start, "discover_store");

    let dir = params
        .direction
        .parse::<commands::link::Direction>()
        .map_err(QipuError::Other)?;

    let resolved_output = match (params.file, params.output) {
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
        params.cli,
        &store,
        commands::dump::DumpOptions {
            note_ids: params.note_ids,
            tag: params.tag,
            moc_id: params.moc_id,
            query: params.query,
            direction: dir,
            max_hops: params.max_hops,
            type_include: params.type_include,
            typed_only: params.typed_only,
            inline_only: params.inline_only,
            include_attachments: !params.no_attachments,
            output: resolved_output,
        },
    )?;
    trace_command!(params.cli, params.start, "execute_command");
    Ok(())
}

/// Parameters for the load command handler
pub struct LoadParams<'a> {
    pub cli: &'a Cli,
    pub root: &'a Path,
    pub pack_file: &'a PathBuf,
    pub strategy: &'a str,
    pub apply_config: bool,
    pub start: Instant,
}

pub(super) fn handle_load(params: LoadParams) -> Result<()> {
    let store = discover_or_open_store(params.cli, params.root)?;
    trace_command!(params.cli, params.start, "discover_store");
    commands::load::execute(
        params.cli,
        &store,
        params.pack_file,
        params.strategy,
        params.apply_config,
    )?;
    trace_command!(params.cli, params.start, "execute_command");
    Ok(())
}

pub(super) fn execute_export<'a>(
    cli: &Cli,
    root: &Path,
    note_ids: &'a [String],
    tag: Option<&'a str>,
    moc_id: Option<&'a str>,
    query: Option<&'a str>,
    output: Option<&'a PathBuf>,
    mode: &'a str,
    with_attachments: bool,
    link_mode: &'a str,
    bib_format: &'a str,
    max_hops: u32,
    pdf: bool,
    start: Instant,
) -> Result<()> {
    handle_export(ExportParams {
        cli,
        root,
        note_ids,
        tag,
        moc_id,
        query,
        output,
        mode,
        with_attachments,
        link_mode,
        bib_format,
        max_hops,
        pdf,
        start,
    })
}

pub(super) fn execute_dump<'a>(
    cli: &Cli,
    root: &Path,
    file: Option<&'a PathBuf>,
    note_ids: &'a [String],
    tag: Option<&'a str>,
    moc_id: Option<&'a str>,
    query: Option<&'a str>,
    direction: &'a str,
    max_hops: u32,
    type_include: &'a [String],
    typed_only: bool,
    inline_only: bool,
    no_attachments: bool,
    output: Option<&'a PathBuf>,
    start: Instant,
) -> Result<()> {
    handle_dump(DumpParams {
        cli,
        root,
        file,
        note_ids,
        tag,
        moc_id,
        query,
        direction,
        max_hops,
        type_include,
        typed_only,
        inline_only,
        no_attachments,
        output,
        start,
    })
}

pub(super) fn execute_load_dispatch<'a>(
    cli: &Cli,
    root: &Path,
    pack_file: &'a PathBuf,
    strategy: &'a str,
    apply_config: bool,
    start: Instant,
) -> Result<()> {
    handle_load(LoadParams {
        cli,
        root,
        pack_file,
        strategy,
        apply_config,
        start,
    })
}
