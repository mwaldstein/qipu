//! `qipu list` command - list notes
//!
//! Per spec (specs/cli-interface.md):
//! - `--tag` filter
//! - `--type` filter
//! - `--since` filter
//! - Deterministic ordering (by created, then id)
//! - Compaction visibility (specs/compaction.md): hide compacted notes by default
//! - `--interactive` - fzf-style picker for selecting a note

pub mod format;

use chrono::{DateTime, Utc};

use crate::cli::{Cli, OutputFormat};
use crate::commands::picker::{pick_single, PickerItem};
use qipu_core::compaction::CompactionContext;
use qipu_core::error::Result;
use qipu_core::note::NoteType;
use qipu_core::query::NoteFilter;
use qipu_core::store::Store;

use self::format::{output_human, output_json, output_records};

/// Execute the list command
#[allow(clippy::too_many_arguments)]
pub fn execute(
    cli: &Cli,
    store: &Store,
    tag: Option<&str>,
    note_type: Option<NoteType>,
    since: Option<DateTime<Utc>>,
    min_value: Option<u8>,
    custom: Option<&str>,
    show_custom: bool,
    interactive: bool,
) -> Result<()> {
    let all_notes = store.list_notes()?;

    let compaction_ctx = CompactionContext::build(&all_notes)?;
    let note_map = CompactionContext::build_note_map(&all_notes);

    // Resolve tag aliases for filtering
    let equivalent_tags = tag.map(|t| store.config().get_equivalent_tags(t));

    let filter = NoteFilter::new()
        .with_tag(tag)
        .with_equivalent_tags(equivalent_tags)
        .with_type(note_type)
        .with_since(since)
        .with_min_value(min_value)
        .with_custom(custom)
        .with_hide_compacted(!cli.no_resolve_compaction);

    let notes: Vec<_> = all_notes
        .iter()
        .filter(|n| filter.matches(n, &compaction_ctx))
        .cloned()
        .collect();

    // Handle interactive picker mode
    if interactive {
        if notes.is_empty() {
            if !cli.quiet {
                println!("No notes found");
            }
            return Ok(());
        }

        let items: Vec<PickerItem> = notes.iter().map(PickerItem::from_note).collect();

        if let Some(selected_id) = pick_single(&items, "Select a note")? {
            // Output just the selected ID for piping to other commands
            println!("{}", selected_id);
        }
        return Ok(());
    }

    match cli.format {
        OutputFormat::Json => {
            output_json(cli, store, &notes, &compaction_ctx, &note_map, show_custom)?
        }
        OutputFormat::Human => {
            output_human(cli, store, &notes, &compaction_ctx, &note_map, show_custom)
        }
        OutputFormat::Records => {
            output_records(cli, store, &notes, &compaction_ctx, &note_map, show_custom)
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
