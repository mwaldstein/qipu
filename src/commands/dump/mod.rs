//! `qipu dump` command - dump notes to a pack file
//!
//! Per spec (specs/pack.md):
//! - Single-file dump with notes, links, and attachments
//! - Selection: `--note`, `--tag`, `--moc`/`--collection-root`, `--query` with graph traversal options
//! - Default: dump full store if no selectors
//! - Include attachments by default, `--no-attachments` flag
//! - Output: stdout by default, or `--output <path>` for file

#![allow(clippy::single_match)]

pub mod model;
pub mod serialize;

use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::cli::Cli;
use qipu_core::error::{QipuError, Result};
use qipu_core::index::{Index, IndexBuilder};
use qipu_core::note::Note;
use qipu_core::store::Store;

pub use model::{DumpOptions, PackAttachment, PackLink};

/// Execute the dump command
pub fn execute(cli: &Cli, store: &Store, options: DumpOptions) -> Result<()> {
    use std::time::Instant;
    let start = Instant::now();

    // Build index for searching and traversal
    let index = IndexBuilder::new(store).build()?;

    if cli.verbose {
        tracing::debug!(elapsed = ?start.elapsed(), "load_indexes");
    }

    // Collect notes based on selection criteria and graph traversal
    let selected_notes = collect_notes_with_traversal(store, &index, &options)?;

    if selected_notes.is_empty() {
        if cli.verbose && !cli.quiet {
            tracing::info!("No notes selected for dump");
        }
        return Ok(());
    }

    // Collect all links between selected notes
    let links = collect_links(&index, &selected_notes, &options)?;

    // Collect attachments if requested
    let attachments = if options.include_attachments {
        collect_attachments(store, &selected_notes)?
    } else {
        Vec::new()
    };

    // Generate pack content
    // Per spec: --format flags do not alter pack contents (specs/pack.md:63-64)
    // Always use records format for pack files
    let pack_content =
        serialize::serialize_pack_records(&selected_notes, &links, &attachments, store)?;

    // Write output to file or stdout
    if let Some(output_path) = options.output {
        let mut file = File::create(output_path)
            .map_err(|e| QipuError::io_operation("create", "pack file", e))?;
        file.write_all(pack_content.as_bytes())
            .map_err(|e| QipuError::io_operation("write to", "pack file", e))?;

        if cli.verbose && !cli.quiet {
            tracing::info!(
                notes = selected_notes.len(),
                links = links.len(),
                attachments = attachments.len(),
                path = %output_path.display(),
                "dumped notes"
            );
        }
    } else {
        print!("{}", pack_content);
    }

    Ok(())
}

/// Collect notes with graph traversal support
fn collect_notes_with_traversal(
    store: &Store,
    index: &Index,
    options: &DumpOptions,
) -> Result<Vec<Note>> {
    let all_notes = store.list_notes()?;
    let traversal =
        (options.max_hops > 0).then_some(crate::commands::note_selection::TraversalSelection {
            direction: options.direction,
            max_hops: options.max_hops,
            type_include: options.type_include,
            typed_only: options.typed_only,
            inline_only: options.inline_only,
        });

    let mut selected_notes = crate::commands::note_selection::collect_notes(
        store,
        index,
        &all_notes,
        &crate::commands::note_selection::NoteSelection {
            note_ids: options.note_ids,
            tag: options.tag,
            moc_id: options.moc_id,
            query: options.query,
            query_limit: 200,
            empty_selection: crate::commands::note_selection::EmptySelection::FullStore,
            traversal,
        },
    )?;

    // Sort notes deterministically (by created, then by id)
    crate::commands::note_selection::sort_notes_by_created_id(&mut selected_notes);

    Ok(selected_notes)
}

/// Collect all links between selected notes
fn collect_links(
    index: &Index,
    selected_notes: &[Note],
    _options: &DumpOptions,
) -> Result<Vec<PackLink>> {
    let selected_ids: std::collections::HashSet<_> =
        selected_notes.iter().map(|n| n.id()).collect();
    let mut links = Vec::new();

    for edge in &index.edges {
        // Only include links between selected notes
        if selected_ids.contains(edge.from.as_str()) && selected_ids.contains(edge.to.as_str()) {
            // Determine if this is an inline link based on source
            let is_inline = matches!(edge.source, qipu_core::index::LinkSource::Inline);

            links.push(PackLink {
                from: edge.from.clone(),
                to: edge.to.clone(),
                link_type: Some(edge.link_type.to_string()),
                inline: is_inline,
            });
        }
    }

    // Sort links deterministically
    links.sort_by(|a, b| a.from.cmp(&b.from).then(a.to.cmp(&b.to)));

    Ok(links)
}

/// Collect attachments for selected notes
fn collect_attachments(_store: &Store, selected_notes: &[Note]) -> Result<Vec<PackAttachment>> {
    use qipu_core::text::markdown::{is_external_or_anchor_target, markdown_links};

    let mut attachments = Vec::new();

    for note in selected_notes {
        if let Some(note_path) = &note.path {
            let note_dir = note_path.parent().unwrap_or_else(|| Path::new("."));

            for line in note.body.lines() {
                if let Some(link) = markdown_links(line).into_iter().next() {
                    let path_str = link.target;
                    if is_external_or_anchor_target(&path_str) {
                        continue;
                    }

                    let attachment_path = note_dir.join(&path_str);
                    if attachment_path.exists() {
                        if let Ok(data) = std::fs::read(&attachment_path) {
                            let content_type: Option<String> =
                                mime_guess::from_path(&attachment_path)
                                    .first()
                                    .map(|mime| mime.to_string());

                            attachments.push(PackAttachment {
                                path: path_str,
                                name: attachment_path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("unknown")
                                    .to_string(),
                                data,
                                content_type,
                            });
                        }
                    }
                }
            }
        }
    }

    // Sort attachments deterministically
    attachments.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(attachments)
}
