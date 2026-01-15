//! `qipu dump` command - dump notes to a pack file
//!
//! Per spec (specs/pack.md):
//! - Single-file dump with notes, links, and attachments
//! - Selection: `--note`, `--tag`, `--moc`, `--query` with graph traversal options
//! - Default: dump full store if no selectors
//! - Include attachments by default, `--no-attachments` flag
//! - Output: stdout by default, or `--output <path>` for file

use std::fs::File;
use std::io::Write;
use std::path::Path;

use base64::{engine::general_purpose, Engine as _};

use crate::cli::Cli;
use crate::commands::link::{Direction, TreeOptions};
use crate::lib::compaction::CompactionContext;
use crate::lib::error::{QipuError, Result};
use crate::lib::index::{search, Index, IndexBuilder};
use crate::lib::note::Note;
use crate::lib::store::Store;

/// Options for the dump command
pub struct DumpOptions<'a> {
    pub note_ids: &'a [String],
    pub tag: Option<&'a str>,
    pub moc_id: Option<&'a str>,
    pub query: Option<&'a str>,
    pub direction: Direction,
    pub max_hops: u32,
    pub type_include: Vec<String>,
    pub typed_only: bool,
    pub inline_only: bool,
    pub include_attachments: bool,
    pub output: Option<&'a Path>,
}

/// Pack file header
#[derive(Debug, Clone, serde::Serialize)]
struct PackHeader {
    version: String,
    created: chrono::DateTime<chrono::Utc>,
    store_path: String,
    notes_count: usize,
    attachments_count: usize,
    links_count: usize,
}

/// Pack entry for a note
#[derive(Debug, Clone, serde::Serialize)]
struct PackNote {
    id: String,
    title: String,
    note_type: String,
    tags: Vec<String>,
    created: Option<chrono::DateTime<chrono::Utc>>,
    updated: Option<chrono::DateTime<chrono::Utc>>,
    path: Option<String>,
    content: String,
    sources: Vec<PackSource>,
}

/// Pack entry for a source reference
#[derive(Debug, Clone, serde::Serialize)]
struct PackSource {
    url: String,
    title: Option<String>,
    accessed: Option<String>,
}

/// Pack entry for a link
#[derive(Debug, Clone, serde::Serialize)]
struct PackLink {
    from: String,
    to: String,
    link_type: Option<String>,
    inline: bool,
}

/// Pack entry for an attachment
#[derive(Debug, Clone, serde::Serialize)]
struct PackAttachment {
    path: String,
    name: String,
    data: Vec<u8>,
    content_type: Option<String>,
}

/// Execute the dump command
pub fn execute(cli: &Cli, store: &Store, options: DumpOptions) -> Result<()> {
    use crate::cli::OutputFormat;

    // Build or load index for searching and traversal
    let index = IndexBuilder::new(store).load_existing()?.build()?;

    // Collect notes based on selection criteria and graph traversal
    let selected_notes = collect_notes_with_traversal(store, &index, &options)?;

    if selected_notes.is_empty() {
        if cli.verbose && !cli.quiet {
            eprintln!("warning: no notes selected for dump");
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
    let pack_content = match cli.format {
        OutputFormat::Human | OutputFormat::Json => {
            // For human and JSON output, use a more readable format
            serialize_pack_readable(&selected_notes, &links, &attachments, store)?
        }
        OutputFormat::Records => {
            // For records output, use the compact pack format
            serialize_pack_records(&selected_notes, &links, &attachments, store)?
        }
    };

    // Write output to file or stdout
    if let Some(output_path) = options.output {
        let mut file = File::create(output_path)
            .map_err(|e| QipuError::Other(format!("failed to create pack file: {}", e)))?;
        file.write_all(pack_content.as_bytes())
            .map_err(|e| QipuError::Other(format!("failed to write to pack file: {}", e)))?;

        if cli.verbose && !cli.quiet {
            eprintln!(
                "dumped {} notes, {} links, {} attachments to {}",
                selected_notes.len(),
                links.len(),
                attachments.len(),
                output_path.display()
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
        let results = search(store, index, q, None, None)?;
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
            max_hops: options.max_hops,
            type_include: options.type_include.clone(),
            type_exclude: Vec::new(),
            typed_only: options.typed_only,
            inline_only: options.inline_only,
            max_nodes: None,
            max_edges: None,
            max_fanout: None,
            max_chars: None,
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
    options: &DumpOptions,
) -> Result<Vec<PackLink>> {
    let selected_ids: std::collections::HashSet<_> =
        selected_notes.iter().map(|n| n.id()).collect();
    let mut links = Vec::new();

    for edge in &index.edges {
        // Only include links between selected notes
        if selected_ids.contains(edge.from.as_str()) && selected_ids.contains(edge.to.as_str()) {
            // Apply link type filters
            if !options.type_include.is_empty() && !options.type_include.contains(&edge.link_type) {
                continue;
            }

            // Determine if this is an inline link based on source
            let is_inline = matches!(edge.source, crate::lib::index::LinkSource::Inline);

            if is_inline && options.inline_only {
                continue;
            }
            if !is_inline && options.typed_only {
                continue;
            }

            links.push(PackLink {
                from: edge.from.clone(),
                to: edge.to.clone(),
                link_type: Some(edge.link_type.clone()),
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
    let mut queue: VecDeque<(String, u32)> = VecDeque::new();

    queue.push_back((root.to_string(), 0));
    visited.insert(root.to_string());

    while let Some((current_id, hop)) = queue.pop_front() {
        if hop >= opts.max_hops {
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
                let neighbor_id = if edge.from == current_id {
                    &edge.to
                } else {
                    &edge.from
                };

                if !visited.contains(neighbor_id) {
                    visited.insert(neighbor_id.clone());
                    queue.push_back((neighbor_id.clone(), hop + 1));

                    // Add the note if not already in selection
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
fn collect_attachments(store: &Store, selected_notes: &[Note]) -> Result<Vec<PackAttachment>> {
    let mut attachments = Vec::new();

    for note in selected_notes {
        if let Some(note_path) = &note.path {
            let note_dir = note_path.parent().unwrap_or_else(|| Path::new("."));

            // Look for attachments directory relative to note
            let _attachments_dir = store.root().join("attachments");

            // Check if note has attachments referenced in content
            // For now, we'll implement a simple approach: look for files referenced in note content
            // A more sophisticated implementation could parse markdown links

            // Extract file references from note content (simple approach)
            // This is a placeholder implementation - a full implementation would parse markdown
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

/// Serialize pack in readable format (for human/JSON output)
fn serialize_pack_readable(
    notes: &[Note],
    links: &[PackLink],
    attachments: &[PackAttachment],
    store: &Store,
) -> Result<String> {
    let header = PackHeader {
        version: "1.0".to_string(),
        created: chrono::Utc::now(),
        store_path: store.root().display().to_string(),
        notes_count: notes.len(),
        attachments_count: attachments.len(),
        links_count: links.len(),
    };

    let pack_notes: Vec<PackNote> = notes
        .iter()
        .map(|note| PackNote {
            id: note.id().to_string(),
            title: note.title().to_string(),
            note_type: note.note_type().to_string(),
            tags: note.frontmatter.tags.clone(),
            created: note.frontmatter.created,
            updated: note.frontmatter.updated,
            path: note.path.as_ref().map(|p| p.display().to_string()),
            content: note.body.clone(),
            sources: note
                .frontmatter
                .sources
                .iter()
                .map(|s| PackSource {
                    url: s.url.clone(),
                    title: s.title.clone(),
                    accessed: s.accessed.clone(),
                })
                .collect(),
        })
        .collect();

    let pack_data = serde_json::json!({
        "header": header,
        "notes": pack_notes,
        "links": links,
        "attachments": attachments.iter().map(|att| {
            let mut obj = serde_json::json!({
                "path": att.path,
                "name": att.name,
                "data": general_purpose::STANDARD.encode(&att.data),
            });
            if let Some(content_type) = &att.content_type {
                obj["content_type"] = serde_json::json!(content_type);
            }
            obj
        }).collect::<Vec<_>>(),
    });

    Ok(serde_json::to_string_pretty(&pack_data)?)
}

/// Serialize pack in records format (compact, line-oriented)
fn serialize_pack_records(
    notes: &[Note],
    links: &[PackLink],
    attachments: &[PackAttachment],
    store: &Store,
) -> Result<String> {
    let mut output = String::new();

    // Header line
    output.push_str(&format!(
        "H pack=1 version=1.0 created={} store={} notes={} links={} attachments={}\n",
        chrono::Utc::now().to_rfc3339(),
        store.root().display(),
        notes.len(),
        links.len(),
        attachments.len()
    ));

    // Notes section
    for note in notes {
        let tags_csv = if note.frontmatter.tags.is_empty() {
            "-".to_string()
        } else {
            note.frontmatter.tags.join(",")
        };

        // Note metadata line
        output.push_str(&format!(
            "N {} {} \"{}\" tags={} created={}\n",
            note.id(),
            note.note_type(),
            note.title(),
            tags_csv,
            note.frontmatter
                .created
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| "-".to_string())
        ));

        // Note content line (base64 encoded for safe transport)
        if !note.body.is_empty() {
            let encoded = general_purpose::STANDARD.encode(note.body.as_bytes());
            output.push_str(&format!("C {}\n", encoded));
            output.push_str("C-END\n");
        }

        // Sources
        for source in &note.frontmatter.sources {
            let title = source.title.as_deref().unwrap_or("");
            let accessed = source.accessed.as_deref().unwrap_or("-");
            output.push_str(&format!(
                "S {} url={} title=\"{}\" accessed={}\n",
                note.id(),
                source.url,
                title,
                accessed
            ));
        }
    }

    // Links section
    for link in links {
        let link_type = link.link_type.as_deref().unwrap_or("-");
        output.push_str(&format!(
            "L {} {} type={} inline={}\n",
            link.from, link.to, link_type, link.inline
        ));
    }

    // Attachments section
    for attachment in attachments {
        let content_type = attachment.content_type.as_deref().unwrap_or("-");
        output.push_str(&format!(
            "A {} name={} content_type={}\n",
            attachment.path, attachment.name, content_type
        ));

        // Attachment data (base64 encoded)
        let encoded = general_purpose::STANDARD.encode(&attachment.data);
        output.push_str(&format!("D {}\n", encoded));
        output.push_str("D-END\n");
    }

    // End marker
    output.push_str("END\n");

    Ok(output)
}
