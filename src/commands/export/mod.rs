//! `qipu export` command - export notes to a single document
//!
//! Per spec (specs/export.md):
//! - Export modes: bundle (concatenate), outline (MOC-first), bibliography (sources only)
//! - Selection: `--note`, `--tag`, `--moc`, `--query`
//! - Deterministic ordering: MOC order or (created_at, id)
//! - Output: stdout by default, or `--output <path>` for file

pub mod emit;
pub mod plan;

use std::fs::File;
use std::io::Write;

use crate::cli::{Cli, OutputFormat};
use crate::lib::compaction::CompactionContext;
use crate::lib::error::{QipuError, Result};
use crate::lib::index::IndexBuilder;
use crate::lib::store::Store;

/// Export mode
#[derive(Debug, Clone, PartialEq)]
pub enum ExportMode {
    /// Bundle export: concatenate notes with metadata headers
    Bundle,
    /// Outline export: use MOC ordering
    Outline,
    /// Bibliography export: extract sources
    Bibliography,
}

impl ExportMode {
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "bundle" => Ok(ExportMode::Bundle),
            "outline" => Ok(ExportMode::Outline),
            "bibliography" | "bib" => Ok(ExportMode::Bibliography),
            _ => Err(QipuError::Other(format!(
                "invalid export mode '{}'. Valid modes: bundle, outline, bibliography",
                s
            ))),
        }
    }
}

/// Options for the export command
pub struct ExportOptions<'a> {
    pub note_ids: &'a [String],
    pub tag: Option<&'a str>,
    pub moc_id: Option<&'a str>,
    pub query: Option<&'a str>,
    pub output: Option<&'a std::path::Path>,
    pub mode: ExportMode,
}

/// Execute the export command
pub fn execute(cli: &Cli, store: &Store, options: ExportOptions) -> Result<()> {
    // Build or load index for searching
    let index = IndexBuilder::new(store).load_existing()?.build()?;

    // Build compaction context for resolved view + annotations
    let all_notes = store.list_notes()?;
    let compaction_ctx = CompactionContext::build(&all_notes)?;

    // Collect notes based on selection criteria
    let mut selected_notes = plan::collect_notes(store, &index, &all_notes, &options)?;

    // Apply compaction resolution unless disabled
    if !cli.no_resolve_compaction {
        selected_notes = plan::resolve_compaction_notes(store, &compaction_ctx, selected_notes)?;
    }

    // Sort notes deterministically (by created, then by id)
    plan::sort_notes_by_created_id(&mut selected_notes);

    if selected_notes.is_empty() {
        if cli.verbose && !cli.quiet {
            eprintln!("warning: no notes selected for export");
        }
        return Ok(());
    }

    // Generate output based on format and mode
    let output_content = match cli.format {
        OutputFormat::Human => {
            // Generate markdown output based on export mode
            match options.mode {
                ExportMode::Bundle => {
                    emit::export_bundle(&selected_notes, store, cli, &compaction_ctx, &all_notes)?
                }
                ExportMode::Outline => emit::export_outline(
                    &selected_notes,
                    store,
                    &index,
                    options.moc_id,
                    cli,
                    &compaction_ctx,
                    !cli.no_resolve_compaction,
                    &all_notes,
                )?,
                ExportMode::Bibliography => emit::export_bibliography(&selected_notes)?,
            }
        }
        OutputFormat::Json => {
            // JSON output: list of notes with metadata
            emit::export_json(
                &selected_notes,
                store,
                &options,
                cli,
                &compaction_ctx,
                &all_notes,
            )?
        }
        OutputFormat::Records => {
            // Records output: low-overhead format
            emit::export_records(
                &selected_notes,
                store,
                &options,
                cli,
                &compaction_ctx,
                &all_notes,
            )?
        }
    };

    // Write output to file or stdout
    if let Some(output_path) = options.output {
        let mut file = File::create(output_path)
            .map_err(|e| QipuError::Other(format!("failed to create output file: {}", e)))?;
        file.write_all(output_content.as_bytes())
            .map_err(|e| QipuError::Other(format!("failed to write to output file: {}", e)))?;

        if cli.verbose && !cli.quiet {
            eprintln!(
                "exported {} notes to {}",
                selected_notes.len(),
                output_path.display()
            );
        }
    } else {
        print!("{}", output_content);
    }

    Ok(())
}
