//! `qipu show` command - display a note
//!
//! Per spec (specs/cli-interface.md):
//! - `qipu show <id-or-path>` - print note to stdout
//! - `qipu show <id-or-path> --links` - inspect links for a note

use std::fs;
use std::path::Path;

use crate::cli::{Cli, OutputFormat};
use crate::commands::link::LinkEntry;
use crate::lib::error::Result;
use crate::lib::index::IndexBuilder;
use crate::lib::note::Note;
use crate::lib::store::Store;

/// Execute the show command
pub fn execute(cli: &Cli, store: &Store, id_or_path: &str, show_links: bool) -> Result<()> {
    // Try to interpret as path first
    let note = if Path::new(id_or_path).exists() {
        let content = fs::read_to_string(id_or_path)?;
        Note::parse(&content, Some(id_or_path.into()))?
    } else {
        // Treat as ID
        store.get_note(id_or_path)?
    };

    if show_links {
        // Show links mode - similar to `qipu link list` but integrated into show
        return execute_show_links(cli, store, &note);
    }

    match cli.format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "id": note.id(),
                "title": note.title(),
                "type": note.note_type().to_string(),
                "tags": note.frontmatter.tags,
                "path": note.path.as_ref().map(|p| p.display().to_string()),
                "created": note.frontmatter.created,
                "updated": note.frontmatter.updated,
                "sources": note.frontmatter.sources,
                "links": note.frontmatter.links,
                "body": note.body,
            });
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

            // Note metadata line
            let tags_csv = if note.frontmatter.tags.is_empty() {
                "-".to_string()
            } else {
                note.frontmatter.tags.join(",")
            };
            println!(
                "N {} {} \"{}\" tags={}",
                note.id(),
                note.note_type(),
                note.title(),
                tags_csv
            );

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
fn execute_show_links(cli: &Cli, store: &Store, note: &Note) -> Result<()> {
    let note_id = note.id().to_string();

    // Load or build the index to get backlinks
    let index = IndexBuilder::new(store).load_existing()?.build()?;

    // Collect links - both directions (consistent with spec for --links)
    let mut entries: Vec<LinkEntry> = Vec::new();

    // Outbound edges (links FROM this note)
    for edge in index.get_outbound_edges(&note_id) {
        let title = index.get_metadata(&edge.to).map(|m| m.title.clone());
        entries.push(LinkEntry {
            direction: "out".to_string(),
            id: edge.to.clone(),
            title,
            link_type: edge.link_type.clone(),
            source: edge.source.to_string(),
        });
    }

    // Inbound edges (backlinks TO this note)
    for edge in index.get_inbound_edges(&note_id) {
        let title = index.get_metadata(&edge.from).map(|m| m.title.clone());
        entries.push(LinkEntry {
            direction: "in".to_string(),
            id: edge.from.clone(),
            title,
            link_type: edge.link_type.clone(),
            source: edge.source.to_string(),
        });
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
