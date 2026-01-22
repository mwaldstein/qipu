//! Human-readable output formatting for search command

use crate::cli::Cli;
use crate::lib::compaction::CompactionContext;
use crate::lib::note::NoteType;
use crate::lib::store::Store;
use std::collections::HashMap;

/// Output in human-readable format
pub fn output_human(
    cli: &Cli,
    _store: &Store,
    results: &[crate::lib::index::SearchResult],
    compaction_ctx: &Option<CompactionContext>,
    notes_cache: &HashMap<String, crate::lib::note::Note>,
    compaction_note_map: &Option<std::collections::HashMap<&str, &crate::lib::note::Note>>,
    query: &str,
) {
    if results.is_empty() {
        if !cli.quiet {
            println!("No results found for '{}'", query);
        }
        return;
    }

    for result in results {
        let type_indicator = match result.note_type {
            NoteType::Fleeting => "F",
            NoteType::Literature => "L",
            NoteType::Permanent => "P",
            NoteType::Moc => "M",
        };

        let mut annotations = String::new();

        if let Some(via) = &result.via {
            annotations.push_str(&format!(" (via {})", via));
        }

        let mut compacts_count = 0;
        if let Some(ref ctx) = compaction_ctx {
            compacts_count = ctx.get_compacts_count(&result.id);
            if compacts_count > 0 {
                annotations.push_str(&format!(" compacts={}", compacts_count));

                if let Some(note) = notes_cache.get(&result.id) {
                    if let Some(ref note_map) = compaction_note_map {
                        if let Some(pct) = ctx.get_compaction_pct(note, note_map) {
                            annotations.push_str(&format!(" compaction={:.0}%", pct));
                        }
                    }
                }
            }
        }

        println!(
            "{} [{}] {}{}",
            result.id, type_indicator, result.title, annotations
        );
        if cli.verbose {
            if let Some(ctx) = &result.match_context {
                println!("    {}", ctx);
            }
        }

        if cli.with_compaction_ids && compacts_count > 0 {
            if let Some(ref ctx) = compaction_ctx {
                let depth = cli.compaction_depth.unwrap_or(1);
                if let Some((ids, truncated)) =
                    ctx.get_compacted_ids(&result.id, depth, cli.compaction_max_nodes)
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
