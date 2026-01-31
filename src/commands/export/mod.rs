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
use qipu_core::compaction::CompactionContext;
use qipu_core::error::{QipuError, Result};
use qipu_core::index::IndexBuilder;
use qipu_core::store::Store;

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
            _ => Err(QipuError::unsupported(
                "export mode",
                s,
                "bundle, outline, bibliography",
            )),
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
            _ => Err(QipuError::unsupported(
                "link mode",
                s,
                "preserve, markdown, anchors",
            )),
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
    pub bib_format: emit::bibliography::BibFormat,
    pub max_hops: u32,
    pub pdf: bool,
}

fn build_export_context(
    store: &Store,
) -> Result<(
    qipu_core::index::Index,
    CompactionContext,
    Vec<qipu_core::note::Note>,
)> {
    let index = IndexBuilder::new(store).build()?;
    let all_notes = store.list_notes()?;
    let compaction_ctx = CompactionContext::build(&all_notes)?;
    Ok((index, compaction_ctx, all_notes))
}

fn prepare_export_notes(
    store: &Store,
    index: &qipu_core::index::Index,
    all_notes: &[qipu_core::note::Note],
    compaction_ctx: &CompactionContext,
    options: &ExportOptions,
    no_resolve: bool,
) -> Result<Vec<qipu_core::note::Note>> {
    let mut selected_notes = plan::collect_notes(store, index, all_notes, options)?;

    if !no_resolve {
        selected_notes = plan::resolve_compaction_notes(store, compaction_ctx, selected_notes)?;
    }

    if options.moc_id.is_none() {
        plan::sort_notes_by_created_id(&mut selected_notes);
    }

    Ok(selected_notes)
}

fn generate_output(
    cli: &Cli,
    selected_notes: &[qipu_core::note::Note],
    store: &Store,
    index: &qipu_core::index::Index,
    options: &ExportOptions,
    compaction_ctx: &CompactionContext,
    all_notes: &[qipu_core::note::Note],
) -> Result<String> {
    match cli.format {
        OutputFormat::Human => match options.mode {
            ExportMode::Bundle => emit::export_bundle(
                selected_notes,
                store,
                options,
                cli,
                compaction_ctx,
                all_notes,
            ),
            ExportMode::Outline => emit::export_outline(
                selected_notes,
                store,
                index,
                options,
                cli,
                compaction_ctx,
                !cli.no_resolve_compaction,
                all_notes,
            ),
            ExportMode::Bibliography => {
                emit::export_bibliography(selected_notes, options.bib_format)
            }
        },
        OutputFormat::Json => emit::export_json(
            selected_notes,
            store,
            options,
            cli,
            compaction_ctx,
            all_notes,
        ),
        OutputFormat::Records => emit::export_records(
            selected_notes,
            store,
            options,
            cli,
            compaction_ctx,
            all_notes,
        ),
    }
}

fn write_output(
    cli: &Cli,
    output_content: &str,
    output_path: Option<&std::path::Path>,
    pdf: bool,
    note_count: usize,
) -> Result<()> {
    if let Some(output_path) = output_path {
        if pdf {
            convert_to_pdf(output_content, output_path, cli)?;
        } else {
            let mut file = File::create(output_path)
                .map_err(|e| QipuError::io_operation("create", "output file", e))?;
            file.write_all(output_content.as_bytes())
                .map_err(|e| QipuError::io_operation("write to", "output file", e))?;
        }

        if cli.verbose && !cli.quiet {
            tracing::info!(
                count = note_count,
                path = %output_path.display(),
                format = if pdf { "pdf" } else { "markdown" },
                "exported notes"
            );
        }
    } else {
        if pdf {
            return Err(QipuError::UsageError(
                "--pdf requires --output (PDF cannot be written to stdout)".to_string(),
            ));
        }
        print!("{}", output_content);
    }

    Ok(())
}

/// Execute the export command
pub fn execute(cli: &Cli, store: &Store, options: ExportOptions) -> Result<()> {
    use std::time::Instant;
    let start = Instant::now();

    let (index, compaction_ctx, all_notes) = build_export_context(store)?;

    if cli.verbose {
        tracing::debug!(elapsed = ?start.elapsed(), "load_indexes");
    }

    let selected_notes = prepare_export_notes(
        store,
        &index,
        &all_notes,
        &compaction_ctx,
        &options,
        cli.no_resolve_compaction,
    )?;

    if selected_notes.is_empty() {
        if cli.verbose && !cli.quiet {
            tracing::info!("No notes selected for export");
        }
        return Ok(());
    }

    let output_content = generate_output(
        cli,
        &selected_notes,
        store,
        &index,
        &options,
        &compaction_ctx,
        &all_notes,
    )?;

    let output_content = if options.with_attachments {
        if let Some(output_path) = options.output {
            let output_dir = output_path.parent().unwrap_or(std::path::Path::new("."));
            let attachments_target_dir = output_dir.join("attachments");
            copy_attachments(store, &selected_notes, &attachments_target_dir, cli)?;
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

    write_output(
        cli,
        &output_content,
        options.output,
        options.pdf,
        selected_notes.len(),
    )
}

/// Rewrite attachment links from ../attachments/ to ./attachments/
fn rewrite_attachment_links(content: &str) -> String {
    use regex::Regex;
    let re = Regex::new(r"\.\./attachments/").expect("valid regex");
    re.replace_all(content, "./attachments/").into_owned()
}

/// Convert markdown content to PDF using pandoc
fn convert_to_pdf(content: &str, output_path: &std::path::Path, cli: &Cli) -> Result<()> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    // Check if pandoc is available
    let pandoc_check = Command::new("pandoc")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match pandoc_check {
        Err(_) => {
            return Err(QipuError::Other(
                "pandoc not found. Please install pandoc to use --pdf flag. \
                 Visit https://pandoc.org/installing.html for installation instructions."
                    .to_string(),
            ));
        }
        Ok(status) if !status.success() => {
            return Err(QipuError::Other(
                "pandoc executable found but returned error. \
                 Please verify pandoc installation."
                    .to_string(),
            ));
        }
        Ok(_) => {
            // pandoc is available, proceed with conversion
        }
    }

    if cli.verbose && !cli.quiet {
        tracing::debug!(path = %output_path.display(), "converting to PDF via pandoc");
    }

    // Run pandoc to convert markdown to PDF
    let mut child = Command::new("pandoc")
        .arg("-f")
        .arg("markdown")
        .arg("-t")
        .arg("pdf")
        .arg("-o")
        .arg(output_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| QipuError::FailedOperation {
            operation: "spawn pandoc".to_string(),
            reason: e.to_string(),
        })?;

    // Write markdown content to pandoc's stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(content.as_bytes())
            .map_err(|e| QipuError::FailedOperation {
                operation: "write to pandoc stdin".to_string(),
                reason: e.to_string(),
            })?;
    }

    // Wait for pandoc to finish
    let output = child
        .wait_with_output()
        .map_err(|e| QipuError::FailedOperation {
            operation: "wait for pandoc".to_string(),
            reason: e.to_string(),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(QipuError::FailedOperation {
            operation: "pandoc conversion".to_string(),
            reason: stderr.to_string(),
        });
    }

    Ok(())
}

/// Copy referenced attachments to the target directory
fn copy_attachments(
    store: &Store,
    notes: &[qipu_core::note::Note],
    target_dir: &std::path::Path,
    cli: &Cli,
) -> Result<()> {
    use regex::Regex;
    use std::fs;

    // Pattern for ../attachments/filename.ext
    let re =
        Regex::new(r"(\.\./attachments/([^)\s\n]+))").map_err(|e| QipuError::FailedOperation {
            operation: "compile regex".to_string(),
            reason: e.to_string(),
        })?;

    let mut copied_count = 0;
    let mut seen_attachments = std::collections::HashSet::new();

    for note in notes {
        for cap in re.captures_iter(&note.body) {
            let filename = &cap[2];
            if seen_attachments.insert(filename.to_string()) {
                let source_path = store.attachments_dir().join(filename);
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
