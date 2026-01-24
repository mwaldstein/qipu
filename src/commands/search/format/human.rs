//! Human-readable output formatting for search command

use crate::cli::Cli;
use crate::lib::compaction::CompactionContext;
use crate::lib::format::build_compaction_annotations;
use crate::lib::format::output_compaction_ids;
use crate::lib::note::NoteType;
use crate::lib::store::Store;
use std::collections::HashMap;

/// Output in human-readable format
pub fn output_human(
    cli: &Cli,
    _store: &Store,
    results: &[crate::lib::index::SearchResult],
    compaction_ctx: &Option<CompactionContext>,
    _notes_cache: &HashMap<String, crate::lib::note::Note>,
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
            output_compaction_ids(cli, &result.id, ctx);
        }
    }
}
