//! Human-readable output formatting for search command

use crate::cli::Cli;
use qipu_core::compaction::CompactionContext;
use qipu_core::format::build_compaction_annotations;
use qipu_core::format::output_compaction_ids;
use qipu_core::format::CompactionOutputOptions;
use qipu_core::note::NoteType;
use qipu_core::store::Store;
use std::collections::HashMap;

/// Output in human-readable format
pub fn output_human(
    cli: &Cli,
    _store: &Store,
    results: &[qipu_core::index::SearchResult],
    compaction_ctx: &Option<CompactionContext>,
    _notes_cache: &HashMap<String, qipu_core::note::Note>,
    compaction_note_map: &Option<std::collections::HashMap<&str, &qipu_core::note::Note>>,
    query: &str,
) {
    if results.is_empty() {
        if !cli.quiet {
            println!("No results found for '{}'", query);
        }
        return;
    }

    let opts = CompactionOutputOptions {
        with_compaction_ids: cli.with_compaction_ids,
        compaction_depth: cli.compaction_depth,
        compaction_max_nodes: cli.compaction_max_nodes,
    };

    for result in results {
        let type_indicator = match result.note_type.as_str() {
            NoteType::FLEETING => "F",
            NoteType::LITERATURE => "L",
            NoteType::PERMANENT => "P",
            NoteType::MOC => "M",
            _ => "F",
        };

        let mut annotations = String::new();

        if let Some(via) = &result.via {
            annotations.push_str(&format!(" (via {})", via));
        }

        if let Some(ref ctx) = compaction_ctx {
            if let Some(ref note_map) = compaction_note_map {
                annotations.push_str(&build_compaction_annotations(&result.id, ctx, note_map));
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

        if let Some(ref ctx) = compaction_ctx {
            output_compaction_ids(&opts, &result.id, ctx);
        }
    }
}
