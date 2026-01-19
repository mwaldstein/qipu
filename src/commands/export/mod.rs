//! `qipu export` command - export notes to a single document
//!
//! Per spec (specs/export.md):
//! - Export modes: bundle (concatenate), outline (MOC-first), bibliography (sources only)
//! - Selection: `--note`, `--tag`, `--moc`, `--query`
//! - Deterministic ordering: MOC order or (created_at, id)
//! - Output: stdout by default, or `--output <path>` for file
//! - Link handling: preserve, markdown, anchors

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

/// Export link handling behavior
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LinkMode {
    /// Preserve original links
    Preserve,
    /// Rewrite wiki links to markdown file links
    Markdown,
    /// Rewrite note links to section anchors in bundle output
    Anchors,
}

impl LinkMode {
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "preserve" => Ok(LinkMode::Preserve),
            "markdown" => Ok(LinkMode::Markdown),
            "anchors" | "anchor" => Ok(LinkMode::Anchors),
            _ => Err(QipuError::Other(format!(
                "invalid link mode '{}'. Valid modes: preserve, markdown, anchors",
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
    pub with_attachments: bool,
    pub link_mode: LinkMode,
}

/// Execute the export command
pub fn execute(cli: &Cli, store: &Store, options: ExportOptions) -> Result<()> {
    use std::time::Instant;
    let start = Instant::now();

    // Build or load index for searching
    let index = IndexBuilder::new(store).build()?;

    if cli.verbose {
        tracing::debug!(elapsed = ?start.elapsed(), "load_indexes");
    }

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
    // Skip sorting for MOC-driven exports to preserve MOC ordering
    if options.moc_id.is_none() {
        plan::sort_notes_by_created_id(&mut selected_notes);
    }

    if selected_notes.is_empty() {
        if cli.verbose && !cli.quiet {
            tracing::info!("no notes selected for export");
        }
        return Ok(());
    }

    // Generate output based on format and mode
    let output_content = match cli.format {
        OutputFormat::Human => {
            // Generate markdown output based on export mode
            match options.mode {
                ExportMode::Bundle => emit::export_bundle(
                    &selected_notes,
                    store,
                    &options,
                    cli,
                    &compaction_ctx,
                    &all_notes,
                )?,
                ExportMode::Outline => emit::export_outline(
                    &selected_notes,
                    store,
                    &index,
                    &options,
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

    // Handle attachment copying and link rewriting if requested
    let output_content = if options.with_attachments {
        if let Some(output_path) = options.output {
            let output_dir = output_path.parent().unwrap_or(std::path::Path::new("."));
            let attachments_target_dir = output_dir.join("attachments");
            copy_attachments(store, &selected_notes, &attachments_target_dir, cli)?;
            // Rewrite attachment links from ../attachments/ to ./attachments/
            rewrite_attachment_links(&output_content)
        } else {
            if cli.verbose && !cli.quiet {
                tracing::info!("--with-attachments ignored when exporting to stdout");
            }
            output_content
        }
    } else {
        output_content
    };

    // Write output to file or stdout
    if let Some(output_path) = options.output {
        let mut file = File::create(output_path)
            .map_err(|e| QipuError::Other(format!("failed to create output file: {}", e)))?;
        file.write_all(output_content.as_bytes())
            .map_err(|e| QipuError::Other(format!("failed to write to output file: {}", e)))?;

        if cli.verbose && !cli.quiet {
            tracing::info!(
                count = selected_notes.len(),
                path = %output_path.display(),
                "exported notes"
            );
        }
    } else {
        print!("{}", output_content);
    }

    Ok(())
}

/// Rewrite attachment links from ../attachments/ to ./attachments/
fn rewrite_attachment_links(content: &str) -> String {
    use regex::Regex;
    let re = Regex::new(r"\.\./attachments/").expect("valid regex");
    re.replace_all(content, "./attachments/").into_owned()
}

/// Copy referenced attachments to the target directory
fn copy_attachments(
    store: &Store,
    notes: &[crate::lib::note::Note],
    target_dir: &std::path::Path,
    cli: &Cli,
) -> Result<()> {
    use regex::Regex;
    use std::fs;

    // Pattern for ../attachments/filename.ext
    let re = Regex::new(r"(\.\./attachments/([^)\s\n]+))")
        .map_err(|e| QipuError::Other(format!("failed to compile regex: {}", e)))?;

    let mut copied_count = 0;
    let mut seen_attachments = std::collections::HashSet::new();

    for note in notes {
        for cap in re.captures_iter(&note.body) {
            let filename = &cap[2];
            if seen_attachments.insert(filename.to_string()) {
                let source_path = store.root().join("attachments").join(filename);
                if source_path.exists() {
                    if !target_dir.exists() {
                        fs::create_dir_all(target_dir)?;
                    }
                    let target_path = target_dir.join(filename);
                    fs::copy(&source_path, &target_path)?;
                    copied_count += 1;
                } else if cli.verbose && !cli.quiet {
                    tracing::info!(filename, note_id = note.id(), "attachment not found");
                }
            }
        }
    }

    if cli.verbose && !cli.quiet && copied_count > 0 {
        tracing::info!(
            count = copied_count,
            target = %target_dir.display(),
            "copied attachments"
        );
    }

    Ok(())
}
