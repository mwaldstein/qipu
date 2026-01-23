//! `qipu show` command - display a note
//!
//! Per spec (specs/cli-interface.md):
//! - `qipu show <id-or-path>` - print note to stdout
//! - `qipu show <id-or-path> --links` - inspect links for a note

use std::fs;
use std::path::Path;

use crate::cli::{Cli, OutputFormat};
use crate::commands::format::{
    add_compaction_to_json, calculate_compaction_info, print_note_records,
};
use crate::commands::link::LinkEntry;
use crate::lib::compaction::CompactionContext;
use crate::lib::error::Result;
use crate::lib::index::IndexBuilder;
use crate::lib::note::Note;
use crate::lib::records::escape_quotes;
use crate::lib::store::Store;

/// Execute the show command
pub fn execute(
    cli: &Cli,
    store: &Store,
    id_or_path: &str,
    show_links: bool,
    show_custom: bool,
) -> Result<()> {
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
                &cli,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use crate::lib::note::NoteType;
    use crate::lib::store::InitOptions;
    use std::fs;
    use std::io::Write;
    use tempfile::tempdir;

    fn make_default_cli() -> Cli {
        Cli {
            root: None,
            store: None,
            format: OutputFormat::Human,
            quiet: false,
            verbose: false,
            log_level: None,
            log_json: false,
            no_resolve_compaction: false,
            with_compaction_ids: false,
            compaction_depth: None,
            compaction_max_nodes: None,
            expand_compaction: false,
            workspace: None,
            no_semantic_inversion: false,
            command: None,
        }
    }

    #[test]
    fn test_show_by_id() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        let note = store
            .create_note("Test Note", None, &["test".to_string()], None)
            .unwrap();
        let id = note.id();

        let cli = make_default_cli();
        let result = execute(&cli, &store, id, false, false);
        assert!(result.is_ok(), "Show by ID should succeed");
    }

    #[test]
    fn test_show_by_file_path() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.md");

        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(
            file,
            "---\nid: qp-external\ntitle: External Note\n---\n\nThis is an external note."
        )
        .unwrap();

        let store = Store::init(dir.path(), InitOptions::default()).unwrap();
        let cli = make_default_cli();

        let result = execute(&cli, &store, file_path.to_str().unwrap(), false, false);
        match result {
            Ok(_) => {}
            Err(e) => panic!("Show by file path failed: {}", e),
        }
    }

    #[test]
    fn test_show_nonexistent_id() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        let cli = make_default_cli();
        let result = execute(&cli, &store, "qp-nonexistent", false, false);
        assert!(result.is_err(), "Show nonexistent ID should fail");
    }

    #[test]
    fn test_show_json_format() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        let note = store
            .create_note(
                "JSON Note",
                Some(NoteType::Permanent),
                &["json".to_string()],
                None,
            )
            .unwrap();
        let id = note.id();

        let mut cli = make_default_cli();
        cli.format = OutputFormat::Json;

        let result = execute(&cli, &store, id, false, false);
        assert!(result.is_ok(), "Show with JSON format should succeed");
    }

    #[test]
    fn test_show_records_format() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        let mut note = store
            .create_note("Records Note", Some(NoteType::Fleeting), &[], None)
            .unwrap();
        note.body = "This is the body content.\nWith multiple lines.".to_string();
        store.save_note(&mut note).unwrap();
        let id = note.id();

        let mut cli = make_default_cli();
        cli.format = OutputFormat::Records;

        let result = execute(&cli, &store, id, false, false);
        assert!(result.is_ok(), "Show with records format should succeed");
    }

    #[test]
    fn test_show_with_compaction_resolution() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        let mut digest_note = store
            .create_note("Digest Note", None, &["digest".to_string()], None)
            .unwrap();
        digest_note.body = "Compacts from qp-source".to_string();
        store.save_note(&mut digest_note).unwrap();

        let source_note = store
            .create_note("Source Note", None, &["source".to_string()], None)
            .unwrap();

        let cli = make_default_cli();
        let result = execute(&cli, &store, source_note.id(), false, false);
        assert!(
            result.is_ok(),
            "Show with compaction resolution should succeed"
        );
    }

    #[test]
    fn test_show_no_resolve_compaction() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        let mut digest_note = store
            .create_note("Digest Note", None, &["digest".to_string()], None)
            .unwrap();
        digest_note.body = "Compacts from qp-source".to_string();
        store.save_note(&mut digest_note).unwrap();

        let source_note = store
            .create_note("Source Note", None, &["source".to_string()], None)
            .unwrap();

        let mut cli = make_default_cli();
        cli.no_resolve_compaction = true;

        let result = execute(&cli, &store, source_note.id(), false, false);
        assert!(
            result.is_ok(),
            "Show with no resolve compaction should succeed"
        );
    }

    #[test]
    fn test_show_links_mode() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        let note = store
            .create_note("Linked Note", None, &["test".to_string()], None)
            .unwrap();
        let id = note.id();

        let cli = make_default_cli();
        let result = execute(&cli, &store, id, true, false);
        assert!(result.is_ok(), "Show links mode should succeed");
    }

    #[test]
    fn test_show_links_json_format() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        let note = store
            .create_note("JSON Links Note", None, &["test".to_string()], None)
            .unwrap();
        let id = note.id();

        let mut cli = make_default_cli();
        cli.format = OutputFormat::Json;

        let result = execute(&cli, &store, id, true, false);
        assert!(result.is_ok(), "Show links with JSON format should succeed");
    }

    #[test]
    fn test_show_links_records_format() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        let note = store
            .create_note("Records Links Note", None, &["test".to_string()], None)
            .unwrap();
        let id = note.id();

        let mut cli = make_default_cli();
        cli.format = OutputFormat::Records;

        let result = execute(&cli, &store, id, true, false);
        assert!(
            result.is_ok(),
            "Show links with records format should succeed"
        );
    }

    #[test]
    fn test_show_with_compaction_ids() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        let mut digest_note = store
            .create_note("Digest with IDs", None, &["digest".to_string()], None)
            .unwrap();
        digest_note.body = "Compacts from qp-source1, qp-source2".to_string();
        store.save_note(&mut digest_note).unwrap();
        let id = digest_note.id();

        let mut cli = make_default_cli();
        cli.format = OutputFormat::Json;
        cli.with_compaction_ids = true;

        let result = execute(&cli, &store, id, false, false);
        assert!(result.is_ok(), "Show with compaction IDs should succeed");
    }

    #[test]
    fn test_show_verbose() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        let note = store
            .create_note("Verbose Note", None, &["test".to_string()], None)
            .unwrap();
        let id = note.id();

        let mut cli = make_default_cli();
        cli.verbose = true;

        let result = execute(&cli, &store, id, false, false);
        assert!(result.is_ok(), "Show with verbose should succeed");
    }
}
