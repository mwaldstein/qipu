//! `qipu list` command - list notes
//!
//! Per spec (specs/cli-interface.md):
//! - `--tag` filter
//! - `--type` filter
//! - `--since` filter
//! - Deterministic ordering (by created, then id)
//! - Compaction visibility (specs/compaction.md): hide compacted notes by default

use chrono::{DateTime, Utc};

use crate::cli::{Cli, OutputFormat};
use crate::lib::compaction::CompactionContext;
use crate::lib::error::Result;
use crate::lib::note::NoteType;
use crate::lib::store::Store;

/// Execute the list command
pub fn execute(
    cli: &Cli,
    store: &Store,
    tag: Option<&str>,
    note_type: Option<NoteType>,
    since: Option<DateTime<Utc>>,
) -> Result<()> {
    let all_notes = store.list_notes()?;
    let mut notes = all_notes.clone();

    // Build compaction context for both filtering and annotations
    // Per spec (specs/compaction.md line 101): hide notes with a compactor by default
    let compaction_ctx = CompactionContext::build(&all_notes)?;

    if !cli.no_resolve_compaction {
        notes.retain(|n| !compaction_ctx.is_compacted(&n.frontmatter.id));
    }

    // Apply filters
    if let Some(tag) = tag {
        notes.retain(|n| n.frontmatter.tags.iter().any(|t| t == tag));
    }

    if let Some(nt) = note_type {
        notes.retain(|n| n.note_type() == nt);
    }

    if let Some(since) = since {
        notes.retain(|n| {
            n.frontmatter
                .created
                .is_some_and(|created| created >= since)
        });
    }

    match cli.format {
        OutputFormat::Json => {
            let output: Vec<_> = notes
                .iter()
                .map(|n| {
                    let mut json = serde_json::json!({
                        "id": n.id(),
                        "title": n.title(),
                        "type": n.note_type().to_string(),
                        "tags": n.frontmatter.tags,
                        "path": n.path.as_ref().map(|p| p.display().to_string()),
                        "created": n.frontmatter.created,
                        "updated": n.frontmatter.updated,
                    });

                    // Add compaction annotations for digest notes
                    // Per spec (specs/compaction.md lines 116-119)
                    let compacts_count = compaction_ctx.get_compacts_count(&n.frontmatter.id);
                    if compacts_count > 0 {
                        if let Some(obj) = json.as_object_mut() {
                            obj.insert("compacts".to_string(), serde_json::json!(compacts_count));

                            if let Some(pct) = compaction_ctx.get_compaction_pct(n, &all_notes) {
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
                                    &n.frontmatter.id,
                                    depth,
                                    cli.compaction_max_nodes,
                                ) {
                                    obj.insert("compacted_ids".to_string(), serde_json::json!(ids));
                                }
                            }
                        }
                    }

                    json
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Human => {
            if notes.is_empty() {
                if !cli.quiet {
                    println!("No notes found");
                }
            } else {
                for note in &notes {
                    let type_indicator = match note.note_type() {
                        NoteType::Fleeting => "F",
                        NoteType::Literature => "L",
                        NoteType::Permanent => "P",
                        NoteType::Moc => "M",
                    };

                    // Build compaction annotations for digest notes
                    // Per spec (specs/compaction.md lines 116-119)
                    let mut annotations = String::new();
                    let compacts_count = compaction_ctx.get_compacts_count(&note.frontmatter.id);
                    if compacts_count > 0 {
                        annotations.push_str(&format!(" compacts={}", compacts_count));

                        if let Some(pct) = compaction_ctx.get_compaction_pct(note, &all_notes) {
                            annotations.push_str(&format!(" compaction={:.0}%", pct));
                        }
                    }

                    println!(
                        "{} [{}] {}{}",
                        note.id(),
                        type_indicator,
                        note.title(),
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
                            let ids_str = ids.join(", ");
                            let suffix = if truncated {
                                let max = cli.compaction_max_nodes.unwrap_or(ids.len());
                                format!(" (truncated, showing {} of {})", max, compacts_count)
                            } else {
                                String::new()
                            };
                            println!("  Compacted: {}{}", ids_str, suffix);
                        }
                    }
                }
            }
        }
        OutputFormat::Records => {
            // Header line per spec (specs/records-output.md)
            println!(
                "H qipu=1 records=1 store={} mode=list notes={}",
                store.root().display(),
                notes.len()
            );
            for note in &notes {
                let tags_csv = if note.frontmatter.tags.is_empty() {
                    "-".to_string()
                } else {
                    note.frontmatter.tags.join(",")
                };

                // Build compaction annotations for digest notes
                // Per spec (specs/compaction.md lines 116-119, 125)
                let mut annotations = String::new();
                let compacts_count = compaction_ctx.get_compacts_count(&note.frontmatter.id);
                if compacts_count > 0 {
                    annotations.push_str(&format!(" compacts={}", compacts_count));

                    if let Some(pct) = compaction_ctx.get_compaction_pct(note, &all_notes) {
                        annotations.push_str(&format!(" compaction={:.0}%", pct));
                    }
                }

                println!(
                    "N {} {} \"{}\" tags={}{}",
                    note.id(),
                    note.note_type(),
                    note.title(),
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
            }
        }
    }

    Ok(())
}
