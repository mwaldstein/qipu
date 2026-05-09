use std::collections::HashSet;

use super::ExportOptions;
use crate::commands::note_selection::{
    collect_notes as collect_selected_notes, sort_notes_by_created_id as sort_selected_notes,
    EmptySelection, NoteSelection, TraversalSelection,
};
use qipu_core::compaction::CompactionContext;
use qipu_core::error::Result;
use qipu_core::graph::Direction;
use qipu_core::index::Index;
use qipu_core::note::Note;
use qipu_core::store::Store;

/// Collect notes based on selection criteria.
pub fn collect_notes(
    store: &Store,
    index: &Index,
    all_notes: &[Note],
    options: &ExportOptions,
) -> Result<Vec<Note>> {
    let traversal = (options.max_hops > 0).then_some(TraversalSelection {
        direction: Direction::Both,
        max_hops: options.max_hops,
        type_include: &[],
        typed_only: false,
        inline_only: false,
    });

    collect_selected_notes(
        store,
        index,
        all_notes,
        &NoteSelection {
            note_ids: options.note_ids,
            tag: options.tag,
            moc_id: options.moc_id,
            query: options.query,
            query_limit: 10_000,
            empty_selection: EmptySelection::Error,
            traversal,
        },
    )
}

pub fn resolve_compaction_notes(
    store: &Store,
    compaction_ctx: &CompactionContext,
    notes: Vec<Note>,
) -> Result<Vec<Note>> {
    let mut resolved = Vec::new();
    let mut seen_ids = HashSet::new();

    for note in notes {
        let canonical_id = compaction_ctx.canon(note.id())?;
        if seen_ids.insert(canonical_id.clone()) {
            if canonical_id == note.id() {
                resolved.push(note);
            } else {
                resolved.push(store.get_note(&canonical_id)?);
            }
        }
    }

    Ok(resolved)
}

pub fn sort_notes_by_created_id(notes: &mut [Note]) {
    sort_selected_notes(notes);
}
