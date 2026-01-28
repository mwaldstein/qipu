//! Human-readable output formatting for list command

use crate::cli::Cli;
use crate::commands::format::status::format_custom_value;
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
    notes: &[crate::lib::note::Note],
    compaction_ctx: &CompactionContext,
    note_map: &HashMap<&str, &crate::lib::note::Note>,
    show_custom: bool,
) {
    if notes.is_empty() {
        if !cli.quiet {
            println!("No notes found");
        }
        return;
    }

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

        output_compaction_ids(cli, note.id(), compaction_ctx);
    }
}
