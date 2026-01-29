//! Human-readable output formatting for list command

use crate::cli::Cli;
use crate::commands::format::status::format_custom_value;
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
    notes: &[qipu_core::note::Note],
    compaction_ctx: &CompactionContext,
    note_map: &HashMap<&str, &qipu_core::note::Note>,
    show_custom: bool,
) {
    if notes.is_empty() {
        if !cli.quiet {
            println!("No notes found");
        }
        return;
    }

    let opts = CompactionOutputOptions {
        with_compaction_ids: cli.with_compaction_ids,
        compaction_depth: cli.compaction_depth,
        compaction_max_nodes: cli.compaction_max_nodes,
    };

    for note in notes {
        let type_indicator = match note.note_type().as_str() {
            NoteType::FLEETING => "F",
            NoteType::LITERATURE => "L",
            NoteType::PERMANENT => "P",
            NoteType::MOC => "M",
            _ => "F",
        };

        let annotations = build_compaction_annotations(note.id(), compaction_ctx, note_map);

        println!(
            "{} [{}] {}{}",
            note.id(),
            type_indicator,
            note.title(),
            annotations
        );

        if show_custom && !note.frontmatter.custom.is_empty() {
            for (key, value) in &note.frontmatter.custom {
                println!("  {}={}", key, format_custom_value(value));
            }
        }

        output_compaction_ids(&opts, note.id(), compaction_ctx);
    }
}
