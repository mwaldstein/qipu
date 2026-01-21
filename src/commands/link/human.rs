use super::LinkEntry;
use crate::cli::Cli;
use crate::lib::compaction::CompactionContext;
use crate::lib::graph::PathResult;
use crate::lib::note::Note;
use std::collections::HashMap;

/// Output in human-readable format
pub fn output_human(
    cli: &Cli,
    entries: &[LinkEntry],
    display_id: &str,
    compaction_ctx: Option<&CompactionContext>,
    note_map: Option<&HashMap<&str, &Note>>,
) {
    if entries.is_empty() {
        if !cli.quiet {
            println!("No links found for {}", display_id);
        }
    } else {
        for entry in entries {
            let dir_arrow = match entry.direction.as_str() {
                "out" => "->",
                "in" => "<-",
                _ => "--",
            };
            let title_part = entry
                .title
                .as_ref()
                .map(|t| format!(" \"{}\"", t))
                .unwrap_or_default();

            // Build compaction annotations for digest nodes
            // Per spec (specs/compaction.md lines 113-122)
            let mut annotations = String::new();
            if let Some(ctx) = compaction_ctx {
                let compacts_count = ctx.get_compacts_count(&entry.id);
                if compacts_count > 0 {
                    annotations.push_str(&format!(" compacts={}", compacts_count));

                    // Calculate compaction percentage if we have note data
                    if let Some(map) = note_map {
                        if let Some(note) = map.get(entry.id.as_str()) {
                            if let Some(pct) = ctx.get_compaction_pct(note, map) {
                                annotations.push_str(&format!(" compaction={:.0}%", pct));
                            }
                        }
                    }
                }
            }

            println!(
                "{} {}{} [{}] ({})",
                dir_arrow, entry.id, title_part, entry.link_type, entry.source
            );

            if !annotations.is_empty() {
                println!("  {}", annotations.trim_start());
            }

            // Show compacted IDs if --with-compaction-ids is set
            if cli.with_compaction_ids {
                if let Some(ref ctx) = compaction_ctx {
                    let compacts_count = ctx.get_compacts_count(&entry.id);
                    if compacts_count > 0 {
                        let depth = cli.compaction_depth.unwrap_or(1);
                        if let Some((ids, truncated)) =
                            ctx.get_compacted_ids(&entry.id, depth, cli.compaction_max_nodes)
                        {
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
    }
}

/// Output path in human-readable format
pub fn output_path_human(
    cli: &Cli,
    result: &PathResult,
    compaction_ctx: Option<&CompactionContext>,
    note_map: Option<&HashMap<&str, &Note>>,
) {
    if !result.found {
        if !cli.quiet {
            println!("No path found from {} to {}", result.from, result.to);
        }
        return;
    }

    // Print path: node -> node -> node
    for (i, note) in result.notes.iter().enumerate() {
        if i > 0 {
            // Print edge info
            if let Some(link) = result.links.get(i - 1) {
                println!("  |");
                println!("  | [{}] ({})", link.link_type, link.source);
                println!("  v");
            }
        }

        // Build compaction annotations for digest nodes
        // Per spec (specs/compaction.md lines 113-122)
        let mut annotations = String::new();
        if let Some(ctx) = compaction_ctx {
            let compacts_count = ctx.get_compacts_count(&note.id);
            if compacts_count > 0 {
                annotations.push_str(&format!(" compacts={}", compacts_count));

                // Calculate compaction percentage if we have note data
                if let Some(map) = note_map {
                    if let Some(full_note) = map.get(note.id.as_str()) {
                        if let Some(pct) = ctx.get_compaction_pct(full_note, map) {
                            annotations.push_str(&format!(" compaction={:.0}%", pct));
                        }
                    }
                }
            }
        }

        println!("{} \"{}\"", note.id, note.title);

        if !annotations.is_empty() {
            println!("  {}", annotations.trim_start());
        }

        // Show compacted IDs if --with-compaction-ids is set
        if cli.with_compaction_ids {
            if let Some(ctx) = compaction_ctx {
                let compacts_count = ctx.get_compacts_count(&note.id);
                if compacts_count > 0 {
                    let depth = cli.compaction_depth.unwrap_or(1);
                    if let Some((ids, truncated)) =
                        ctx.get_compacted_ids(&note.id, depth, cli.compaction_max_nodes)
                    {
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

    println!();
    println!("Path length: {} hop(s)", result.path_length);
}
