//! `qipu dump` command - dump notes to a pack file
//!
//! Per spec (specs/pack.md):
//! - Single-file dump with notes, links, and attachments
//! - Selection: `--note`, `--tag`, `--moc`, `--query` with graph traversal options
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
use qipu_core::compaction::CompactionContext;
use qipu_core::error::{QipuError, Result};
use qipu_core::graph::{Direction, TreeOptions};
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
            tracing::info!("no notes selected for dump");
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
            .map_err(|e| QipuError::Other(format!("failed to create pack file: {}", e)))?;
        file.write_all(pack_content.as_bytes())
            .map_err(|e| QipuError::Other(format!("failed to write to pack file: {}", e)))?;

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
    let mut selected_notes: Vec<Note> = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    // Helper function to add notes to selection
    fn add_note_internal(
        store: &Store,
        selected_notes: &mut Vec<Note>,
        seen_ids: &mut std::collections::HashSet<String>,
        id: &str,
    ) -> Result<()> {
        if seen_ids.insert(id.to_string()) {
            match store.get_note(id) {
                Ok(note) => selected_notes.push(note),
                Err(_) => {
                    return Err(QipuError::NoteNotFound { id: id.to_string() });
                }
            }
        }
        Ok(())
    }

    // Selection by explicit note IDs
    for id in options.note_ids {
        add_note_internal(store, &mut selected_notes, &mut seen_ids, id)?;
    }

    // Selection by tag
    if let Some(tag_name) = options.tag {
        let notes = store.list_notes()?;
        for note in notes {
            if note.frontmatter.tags.contains(&tag_name.to_string()) {
                add_note_internal(store, &mut selected_notes, &mut seen_ids, note.id())?;
            }
        }
    }

    // Selection by MOC
    if let Some(moc_id) = options.moc_id {
        // Get notes linked from the MOC
        let edges = index.get_outbound_edges(moc_id);
        for edge in edges {
            add_note_internal(store, &mut selected_notes, &mut seen_ids, &edge.to)?;
        }
    }

    // Selection by query
    if let Some(q) = options.query {
        let results = store
            .db()
            .search(q, None, None, None, None, 200, &store.config().search)?;
        for result in results {
            add_note_internal(store, &mut selected_notes, &mut seen_ids, &result.id)?;
        }
    }

    // If no selection criteria provided, dump all notes
    if options.note_ids.is_empty()
        && options.tag.is_none()
        && options.moc_id.is_none()
        && options.query.is_none()
    {
        let all_notes = store.list_notes()?;
        for note in all_notes {
            add_note_internal(store, &mut selected_notes, &mut seen_ids, note.id())?;
        }
    }

    // Graph traversal expansion if needed
    if !selected_notes.is_empty() && (options.max_hops > 0 || options.direction != Direction::Both)
    {
        let initial_ids: Vec<String> = selected_notes.iter().map(|n| n.id().to_string()).collect();

        let traversal_options = TreeOptions {
            direction: options.direction,
            max_hops: qipu_core::graph::HopCost::from(options.max_hops),
            type_include: options.type_include.clone(),
            type_exclude: Vec::new(),
            typed_only: options.typed_only,
            inline_only: options.inline_only,
            max_nodes: None,
            max_edges: None,
            max_fanout: None,
            max_chars: None,
            semantic_inversion: true,
            min_value: None,
            ignore_value: false,
        };

        // Build compaction context if needed
        let notes = store.list_notes()?;
        let compaction_ctx = Some(CompactionContext::build(&notes)?);

        // For each initial note, perform simple traversal and collect discovered notes
        for initial_id in &initial_ids {
            perform_simple_traversal(
                index,
                initial_id,
                &traversal_options,
                compaction_ctx.as_ref(),
                store,
                &mut selected_notes,
                &mut seen_ids,
            )?;
        }
    }

    // Sort notes deterministically (by created, then by id)
    selected_notes.sort_by(
        |a, b| match (&a.frontmatter.created, &b.frontmatter.created) {
            (Some(a_created), Some(b_created)) => {
                a_created.cmp(b_created).then_with(|| a.id().cmp(b.id()))
            }
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.id().cmp(b.id()),
        },
    );

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

/// Perform simple graph traversal for dump command
fn perform_simple_traversal(
    index: &Index,
    root: &str,
    opts: &TreeOptions,
    _compaction_ctx: Option<&CompactionContext>,
    store: &Store,
    selected_notes: &mut Vec<Note>,
    seen_ids: &mut std::collections::HashSet<String>,
) -> Result<()> {
    use std::collections::{HashSet, VecDeque};

    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(String, qipu_core::graph::HopCost)> = VecDeque::new();

    queue.push_back((root.to_string(), qipu_core::graph::HopCost::from(0)));
    visited.insert(root.to_string());

    while let Some((current_id, accumulated_cost)) = queue.pop_front() {
        if accumulated_cost.value() >= opts.max_hops.value() {
            continue;
        }

        // Get outbound edges from current note
        for edge in &index.edges {
            let should_follow = match opts.direction {
                Direction::Out => edge.from == current_id,
                Direction::In => edge.to == current_id,
                Direction::Both => edge.from == current_id || edge.to == current_id,
            };

            if should_follow {
                // Apply link type filters to determine if we should follow this edge
                if !opts.type_include.is_empty()
                    && !opts
                        .type_include
                        .iter()
                        .any(|t| t == edge.link_type.as_str())
                {
                    continue;
                }

                // Determine if this is an inline link based on source
                let is_inline = matches!(edge.source, qipu_core::index::LinkSource::Inline);

                if !is_inline && opts.inline_only {
                    continue;
                }
                if is_inline && opts.typed_only {
                    continue;
                }

                let neighbor_id = if edge.from == current_id {
                    &edge.to
                } else {
                    &edge.from
                };

                if !visited.contains(neighbor_id) {
                    visited.insert(neighbor_id.clone());
                    let edge_cost = qipu_core::graph::get_link_type_cost(
                        edge.link_type.as_str(),
                        store.config(),
                    );
                    queue.push_back((neighbor_id.clone(), accumulated_cost + edge_cost));

                    // Add note if not already in selection
                    if !seen_ids.contains(neighbor_id) {
                        match store.get_note(neighbor_id) {
                            Ok(note) => {
                                selected_notes.push(note);
                                seen_ids.insert(neighbor_id.clone());
                            }
                            Err(_) => {} // Skip if note not found
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Collect attachments for selected notes
fn collect_attachments(_store: &Store, selected_notes: &[Note]) -> Result<Vec<PackAttachment>> {
    let mut attachments = Vec::new();

    for note in selected_notes {
        if let Some(note_path) = &note.path {
            let note_dir = note_path.parent().unwrap_or_else(|| Path::new("."));

            // Extract file references from note content (simple approach)
            let content_lines = note.body.lines();
            for line in content_lines {
                // Look for patterns like ![alt](path) or [text](path)
                if let Some(start) = line.find('(') {
                    if let Some(end) = line[start..].find(')') {
                        let path_str = &line[start + 1..start + end];
                        // Skip URLs, focus on local files
                        if !path_str.starts_with("http://") && !path_str.starts_with("https://") {
                            let attachment_path = note_dir.join(path_str);
                            if attachment_path.exists() {
                                // Try to read the attachment
                                if let Ok(data) = std::fs::read(&attachment_path) {
                                    let content_type: Option<String> =
                                        mime_guess::from_path(&attachment_path)
                                            .first()
                                            .map(|mime| mime.to_string());

                                    attachments.push(PackAttachment {
                                        path: path_str.to_string(),
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
        }
    }

    // Sort attachments deterministically
    attachments.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(attachments)
}
