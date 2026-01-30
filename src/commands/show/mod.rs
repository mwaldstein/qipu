//! `qipu show` command - display a note
//!
//! Per spec (specs/cli-interface.md):
//! - `qipu show <id-or-path>` - print note to stdout
//! - `qipu show <id-or-path> --links` - inspect links for a note

use crate::cli::{Cli, OutputFormat};
use crate::commands::format::{
    add_compaction_to_json, calculate_compaction_info, print_note_records,
};
use crate::commands::link::LinkEntry;
use qipu_core::compaction::CompactionContext;
use qipu_core::error::Result;
use qipu_core::index::IndexBuilder;
use qipu_core::note::Note;

use qipu_core::store::Store;

/// Execute the show command
pub fn execute(
    cli: &Cli,
    store: &Store,
    id_or_path: &str,
    show_links: bool,
    show_custom: bool,
) -> Result<()> {
    // Load note by ID or path
    let mut note = store.load_note_by_id_or_path(id_or_path)?;

    // Build compaction context for annotations and resolution
    // Per spec (specs/compaction.md lines 116-119)
    let all_notes = store.list_notes()?;
    let compaction_ctx = CompactionContext::build(&all_notes)?;

    // Build note map for efficient lookups (avoid O(n²) when calculating compaction pct)
    let note_map = CompactionContext::build_note_map(&all_notes);

    // Resolve compaction unless disabled
    let mut via = None;
    if !cli.no_resolve_compaction {
        let canonical_id = compaction_ctx.canon(note.id())?;
        if canonical_id != note.id() {
            via = Some(note.id().to_string());
            note = store.get_note(&canonical_id)?;
        }
    }

    if show_links {
        // Show links mode - similar to `qipu link list` but integrated into show
        let compaction_ctx = if cli.no_resolve_compaction {
            None
        } else {
            Some(&compaction_ctx)
        };
        return execute_show_links(cli, store, &note, compaction_ctx, &all_notes);
    }

    match cli.format {
        OutputFormat::Json => {
            let mut output = serde_json::json!({
                "id": note.id(),
                "title": note.title(),
                "type": note.note_type().to_string(),
                "tags": note.frontmatter.tags,

                "created": note.frontmatter.created,
                "updated": note.frontmatter.updated,
                "value": note.frontmatter.value,
                "sources": note.frontmatter.sources,
                "links": note.frontmatter.links,
                "source": note.frontmatter.source,
                "author": note.frontmatter.author,
                "generated_by": note.frontmatter.generated_by,
                "prompt_hash": note.frontmatter.prompt_hash,
                "verified": note.frontmatter.verified,
                "body": note.body,
            });

            if let Some(via_id) = &via {
                if let Some(obj) = output.as_object_mut() {
                    obj.insert("via".to_string(), serde_json::json!(via_id));
                }
            }

            // Add custom metadata if requested (opt-in)
            if show_custom && !note.frontmatter.custom.is_empty() {
                if let Some(obj) = output.as_object_mut() {
                    obj.insert(
                        "custom".to_string(),
                        serde_json::to_value(&note.frontmatter.custom)
                            .unwrap_or(serde_json::json!({})),
                    );
                }
            }

            // Add compaction annotations for digest notes
            let compaction_info = calculate_compaction_info(cli, &note, &note_map, &compaction_ctx);

            if let Some(obj) = output.as_object_mut() {
                add_compaction_to_json(
                    obj,
                    compaction_info.count,
                    compaction_info.percentage,
                    Some(compaction_info.compacted_ids),
                    compaction_info.truncated,
                );
            }

            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Human => {
            // Print the raw markdown content
            let content = note.to_markdown()?;
            print!("{}", content);
        }
        OutputFormat::Records => {
            println!(
                "H qipu=1 records=1 store={} mode=show id={}",
                store.root().display(),
                note.id()
            );

            let compaction_info = calculate_compaction_info(cli, &note, &note_map, &compaction_ctx);
            print_note_records(
                cli,
                &note,
                store,
                via.as_deref(),
                show_custom,
                compaction_info,
            );
        }
    }

    Ok(())
}

/// Execute show with --links flag
/// Shows inline + typed links, both directions
fn execute_show_links(
    cli: &Cli,
    store: &Store,
    note: &Note,
    compaction_ctx: Option<&CompactionContext>,
    all_notes: &[Note],
) -> Result<()> {
    use std::time::Instant;
    let start = Instant::now();
    let note_id = note.id().to_string();

    // Load or build the index to get backlinks
    let index = IndexBuilder::new(store).build()?;

    if cli.verbose {
        tracing::debug!(elapsed = ?start.elapsed(), "load_indexes");
    }

    let equivalence_map = if let Some(ctx) = compaction_ctx {
        Some(ctx.build_equivalence_map(all_notes)?)
    } else {
        None
    };

    let source_ids = equivalence_map
        .as_ref()
        .and_then(|map| map.get(&note_id).cloned())
        .unwrap_or_else(|| vec![note_id.clone()]);

    // Collect links - both directions (consistent with spec for --links)
    let mut entries: Vec<LinkEntry> = Vec::new();

    // Outbound edges (links FROM this note or any compacted source)
    for source_id in &source_ids {
        for edge in index.get_outbound_edges(source_id) {
            let mut entry = LinkEntry {
                direction: "out".to_string(),
                id: edge.to.clone(),
                title: index.get_metadata(&edge.to).map(|m| m.title.clone()),
                link_type: edge.link_type.to_string(),
                source: edge.source.to_string(),
                via: None,
            };

            if let Some(ctx) = compaction_ctx {
                let original_id = entry.id.clone();
                entry.id = ctx.canon(&entry.id)?;
                if entry.id == note_id {
                    continue;
                }
                // Set via annotation if ID changed due to canonicalization
                if entry.id != original_id {
                    entry.via = Some(original_id);
                }
                entry.title = index.get_metadata(&entry.id).map(|m| m.title.clone());
            }

            entries.push(entry);
        }
    }

    // Inbound edges (backlinks TO this note or any compacted source)
    for source_id in &source_ids {
        for edge in index.get_inbound_edges(source_id) {
            let mut entry = if !cli.no_semantic_inversion {
                // Semantic inversion enabled: show virtual outbound link with inverted type
                let virtual_edge = edge.invert(store.config());
                LinkEntry {
                    direction: "out".to_string(),
                    id: virtual_edge.to.clone(),
                    title: index
                        .get_metadata(&virtual_edge.to)
                        .map(|m| m.title.clone()),
                    link_type: virtual_edge.link_type.to_string(),
                    source: virtual_edge.source.to_string(),
                    via: None,
                }
            } else {
                // Semantic inversion disabled: show raw backlink with original type
                LinkEntry {
                    direction: "in".to_string(),
                    id: edge.from.clone(),
                    title: index.get_metadata(&edge.from).map(|m| m.title.clone()),
                    link_type: edge.link_type.to_string(),
                    source: edge.source.to_string(),
                    via: None,
                }
            };

            if let Some(ctx) = compaction_ctx {
                let original_id = entry.id.clone();
                entry.id = ctx.canon(&entry.id)?;
                if entry.id == note_id {
                    continue;
                }
                // Set via annotation if ID changed due to canonicalization
                if entry.id != original_id {
                    entry.via = Some(original_id);
                }
                entry.title = index.get_metadata(&entry.id).map(|m| m.title.clone());
            }

            entries.push(entry);
        }
    }

    // Sort for determinism: direction, then type, then id
    entries.sort_by(|a, b| {
        a.direction
            .cmp(&b.direction)
            .then_with(|| a.link_type.cmp(&b.link_type))
            .then_with(|| a.id.cmp(&b.id))
    });

    // Output - consistent with `qipu link list` schema
    match cli.format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "id": note_id,
                "title": note.title(),
                "links": entries,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Human => {
            println!("Links for {} \"{}\":", note_id, note.title());
            println!();

            if entries.is_empty() {
                if !cli.quiet {
                    println!("No links found");
                }
            } else {
                // Group by direction for clearer output
                let outbound: Vec<_> = entries.iter().filter(|e| e.direction == "out").collect();
                let inbound: Vec<_> = entries.iter().filter(|e| e.direction == "in").collect();

                if !outbound.is_empty() {
                    println!("Outbound links ({}):", outbound.len());
                    for entry in &outbound {
                        let title_part = entry
                            .title
                            .as_ref()
                            .map(|t| format!(" \"{}\"", t))
                            .unwrap_or_default();
                        println!(
                            "  -> {} {} [{}] ({})",
                            entry.id, title_part, entry.link_type, entry.source
                        );
                    }
                }

                if !inbound.is_empty() {
                    if !outbound.is_empty() {
                        println!();
                    }
                    println!("Inbound links ({}):", inbound.len());
                    for entry in &inbound {
                        let title_part = entry
                            .title
                            .as_ref()
                            .map(|t| format!(" \"{}\"", t))
                            .unwrap_or_default();
                        println!(
                            "  <- {} {} [{}] ({})",
                            entry.id, title_part, entry.link_type, entry.source
                        );
                    }
                }
            }
        }
        OutputFormat::Records => {
            // Header line per spec (specs/records-output.md)
            // mode=show.links to distinguish from regular show
            println!(
                "H qipu=1 records=1 store={} mode=show.links id={} direction=both",
                store.root().display(),
                note_id
            );

            // Edge lines - consistent with link list format
            for entry in &entries {
                let (from, to) = match entry.direction.as_str() {
                    "out" => (note_id.clone(), entry.id.clone()),
                    "in" => (entry.id.clone(), note_id.clone()),
                    _ => (note_id.clone(), entry.id.clone()),
                };
                println!("E {} {} {} {}", from, entry.link_type, to, entry.source);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
