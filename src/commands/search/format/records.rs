//! Records output formatting for search command

use crate::cli::Cli;
use crate::lib::compaction::CompactionContext;
use crate::lib::records::escape_quotes;
use crate::lib::store::Store;
use std::collections::HashMap;

/// Output in records format
pub fn output_records(
    cli: &Cli,
    store: &Store,
    results: &[crate::lib::index::SearchResult],
    compaction_ctx: &Option<CompactionContext>,
    notes_cache: &HashMap<String, crate::lib::note::Note>,
    compaction_note_map: &Option<std::collections::HashMap<&str, &crate::lib::note::Note>>,
    query: &str,
) {
    println!(
        "H qipu=1 records=1 store={} mode=search query=\"{}\" results={}",
        store.root().display(),
        query.replace('"', "\\\""),
        results.len()
    );

    for result in results {
        let tags_csv = if result.tags.is_empty() {
            "-".to_string()
        } else {
            result.tags.join(",")
        };

        let mut annotations = String::new();

        if let Some(via) = &result.via {
            annotations.push_str(&format!(" via={}", via));
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
            "N {} {} \"{}\" tags={}{}",
            result.id,
            result.note_type,
            escape_quotes(&result.title),
            tags_csv,
            annotations
        );
        if let Some(ctx) = &result.match_context {
            println!("S {} {}", result.id, ctx);
        }

        if cli.with_compaction_ids && compacts_count > 0 {
            if let Some(ref ctx) = compaction_ctx {
                let depth = cli.compaction_depth.unwrap_or(1);
                if let Some((ids, truncated)) =
                    ctx.get_compacted_ids(&result.id, depth, cli.compaction_max_nodes)
                {
                    for id in &ids {
                        println!("D compacted {} from={}", id, result.id);
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
