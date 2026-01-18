//! `qipu show` command - display a note
//!
//! Per spec (specs/cli-interface.md):
//! - `qipu show <id-or-path>` - print note to stdout
//! - `qipu show <id-or-path> --links` - inspect links for a note

use std::fs;
use std::path::Path;

use crate::cli::{Cli, OutputFormat};
use crate::commands::link::LinkEntry;
use crate::lib::compaction::CompactionContext;
use crate::lib::error::Result;
use crate::lib::index::IndexBuilder;
use crate::lib::note::Note;
use crate::lib::records::escape_quotes;
use crate::lib::store::Store;

/// Execute the show command
pub fn execute(cli: &Cli, store: &Store, id_or_path: &str, show_links: bool) -> Result<()> {
    // Try to interpret as path first
    let mut note = if Path::new(id_or_path).exists() {
        let content = fs::read_to_string(id_or_path)?;
        Note::parse(&content, Some(id_or_path.into()))?
    } else {
        // Treat as ID
        store.get_note(id_or_path)?
    };

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
                "path": note.path.as_ref().map(|p| p.display().to_string()),
                "created": note.frontmatter.created,
                "updated": note.frontmatter.updated,
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

            // Add compaction annotations for digest notes
            let compacts_count = compaction_ctx.get_compacts_count(&note.frontmatter.id);
            if compacts_count > 0 {
                if let Some(obj) = output.as_object_mut() {
                    obj.insert("compacts".to_string(), serde_json::json!(compacts_count));

                    if let Some(pct) = compaction_ctx.get_compaction_pct(&note, &note_map) {
                        obj.insert(
                            "compaction_pct".to_string(),
                            serde_json::json!(format!("{:.1}", pct)),
                        );
                    }

                    // Add compacted IDs if --with-compaction-ids is set
                    // Per spec (specs/compaction.md line 131)
                    if cli.with_compaction_ids {
                        let depth = cli.compaction_depth.unwrap_or(1);
                        if let Some((ids, _truncated)) = compaction_ctx.get_compacted_ids(
                            &note.frontmatter.id,
                            depth,
                            cli.compaction_max_nodes,
                        ) {
                            obj.insert("compacted_ids".to_string(), serde_json::json!(ids));
                        }
                    }
                }
            }

            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Human => {
            // Print the raw markdown content
            let content = note.to_markdown()?;
            print!("{}", content);
        }
        OutputFormat::Records => {
            // Header line per spec (specs/records-output.md)
            println!(
                "H qipu=1 records=1 store={} mode=show id={}",
                store.root().display(),
                note.id()
            );

            // Note metadata line with compaction annotations
            let tags_csv = if note.frontmatter.tags.is_empty() {
                "-".to_string()
            } else {
                note.frontmatter.tags.join(",")
            };

            // Build compaction annotations for digest notes
            let mut annotations = String::new();
            if let Some(via_id) = &via {
                annotations.push_str(&format!(" via={}", via_id));
            }
            let compacts_count = compaction_ctx.get_compacts_count(&note.frontmatter.id);
            if compacts_count > 0 {
                annotations.push_str(&format!(" compacts={}", compacts_count));

                if let Some(pct) = compaction_ctx.get_compaction_pct(&note, &note_map) {
                    annotations.push_str(&format!(" compaction={:.0}%", pct));
                }
            }

            println!(
                "N {} {} \"{}\" tags={}{}",
                note.id(),
                note.note_type(),
                escape_quotes(note.title()),
                tags_csv,
                annotations
            );

            // Show compacted IDs if --with-compaction-ids is set
            // Per spec (specs/compaction.md line 131)
            if cli.with_compaction_ids && compacts_count > 0 {
                let depth = cli.compaction_depth.unwrap_or(1);
                if let Some((ids, truncated)) = compaction_ctx.get_compacted_ids(
                    &note.frontmatter.id,
                    depth,
                    cli.compaction_max_nodes,
                ) {
                    for id in &ids {
                        println!("D compacted {} from={}", id, note.id());
                    }
                    if truncated {
                        println!(
                            "D compacted_truncated max={} total={}",
                            cli.compaction_max_nodes.unwrap_or(ids.len()),
                            compacts_count
                        );
                    }
                }
            }

            // Summary line
            let summary = note.summary();
            if !summary.is_empty() {
                // Only output first line of summary per spec
                let first_line = summary.lines().next().unwrap_or(&summary);
                println!("S {} {}", note.id(), first_line);
            }

            // Body lines with terminator
            println!("B {}", note.id());
            for line in note.body.lines() {
                println!("{}", line);
            }
            println!("B-END");
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
    let note_id = note.id().to_string();

    // Load or build the index to get backlinks
    let index = IndexBuilder::new(store).load_existing()?.build()?;

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
            };

            if let Some(ctx) = compaction_ctx {
                entry.id = ctx.canon(&entry.id)?;
                if entry.id == note_id {
                    continue;
                }
                entry.title = index.get_metadata(&entry.id).map(|m| m.title.clone());
            }

            entries.push(entry);
        }
    }

    // Inbound edges (backlinks TO this note or any compacted source)
    for source_id in &source_ids {
        for edge in index.get_inbound_edges(source_id) {
            let mut entry = LinkEntry {
                direction: "in".to_string(),
                id: edge.from.clone(),
                title: index.get_metadata(&edge.from).map(|m| m.title.clone()),
                link_type: edge.link_type.to_string(),
                source: edge.source.to_string(),
            };

            if let Some(ctx) = compaction_ctx {
                entry.id = ctx.canon(&entry.id)?;
                if entry.id == note_id {
                    continue;
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
