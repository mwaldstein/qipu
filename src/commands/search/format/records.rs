//! Records output formatting for search command

use crate::cli::Cli;
use crate::lib::compaction::CompactionContext;
use crate::lib::format::build_compaction_annotations;
use crate::lib::records::escape_quotes;
use crate::lib::store::Store;
use std::collections::HashMap;

/// Output in records format
pub fn output_records(
    cli: &Cli,
    store: &Store,
    results: &[crate::lib::index::SearchResult],
    compaction_ctx: &Option<CompactionContext>,
    _notes_cache: &HashMap<String, crate::lib::note::Note>,
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

        if let Some(ref ctx) = compaction_ctx {
            if let Some(ref note_map) = compaction_note_map {
                annotations.push_str(&build_compaction_annotations(&result.id, ctx, note_map));
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

        if let Some(ref ctx) = compaction_ctx {
            let compacts_count = ctx.get_compacts_count(&result.id);
            if cli.with_compaction_ids && compacts_count > 0 {
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
