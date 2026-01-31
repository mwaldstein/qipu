use super::state::SelectionState;
use crate::cli::Cli;
use crate::commands::context::types::ContextOptions;
use crate::commands::context::walk;
use qipu_core::error::{QipuError, Result};
use qipu_core::note::Note;
use qipu_core::store::Store;
use std::collections::HashMap;

/// Collect notes from a graph walk starting at walk_id
pub fn collect_from_walk<'a>(
    state: &mut SelectionState<'a>,
    cli: &Cli,
    store: &'a Store,
    options: &ContextOptions<'a>,
    note_map: &'a HashMap<&'a str, &'a Note>,
    resolve_id: &dyn Fn(&str) -> Result<String>,
) -> Result<()> {
    let Some(walk_id) = options.walk.id else {
        return Ok(());
    };

    let walked_ids = walk::walk_for_context(
        cli,
        store,
        walk_id,
        options.walk.direction,
        options.walk.max_hops,
        options.walk.type_include,
        options.walk.type_exclude,
        options.walk.typed_only,
        options.walk.inline_only,
        options.walk.max_nodes,
        options.walk.max_edges,
        options.walk.max_fanout,
        options.walk.min_value,
        options.walk.ignore_value,
    )?;

    for id in &walked_ids {
        let resolved_id = resolve_id(id)?;
        state.add_note(
            id,
            resolved_id,
            note_map,
            Some(format!("walk:{}", walk_id)),
            None,
        )?;
    }

    Ok(())
}

/// Collect notes from explicit note IDs
pub fn collect_from_note_ids<'a>(
    state: &mut SelectionState<'a>,
    options: &ContextOptions<'a>,
    note_map: &'a HashMap<&'a str, &'a Note>,
    resolve_id: &dyn Fn(&str) -> Result<String>,
) -> Result<()> {
    for id in options.selection.note_ids {
        let resolved_id = resolve_id(id)?;
        state.add_note(id, resolved_id, note_map, None, None)?;
    }
    Ok(())
}

/// Collect notes that have a specific tag
pub fn collect_from_tag<'a>(
    state: &mut SelectionState<'a>,
    store: &'a Store,
    options: &ContextOptions<'a>,
    note_map: &'a HashMap<&'a str, &'a Note>,
    resolve_id: &dyn Fn(&str) -> Result<String>,
) -> Result<()> {
    let Some(tag_name) = options.selection.tag else {
        return Ok(());
    };

    let notes_with_tag = store.db().list_notes(None, Some(tag_name), None)?;
    for note in &notes_with_tag {
        let resolved_id = resolve_id(&note.id)?;
        state.add_note(&note.id, resolved_id, note_map, None, None)?;
    }

    Ok(())
}

/// Collect notes from a search query
pub fn collect_from_query<'a>(
    state: &mut SelectionState<'a>,
    cli: &Cli,
    store: &'a Store,
    options: &ContextOptions<'a>,
    note_map: &'a HashMap<&'a str, &'a Note>,
    resolve_id: &dyn Fn(&str) -> Result<String>,
) -> Result<()> {
    let Some(q) = options.selection.query else {
        return Ok(());
    };

    let results = store
        .db()
        .search(q, None, None, None, None, 100, &store.config().search)?;

    for result in results {
        let resolved_id = resolve_id(&result.id)?;
        let via_source = if !cli.no_resolve_compaction && resolved_id != result.id {
            Some(result.id.clone())
        } else {
            None
        };

        if let Some(via_id) = via_source {
            state.via_map.entry(resolved_id.clone()).or_insert(via_id);
        }

        state.add_note(&result.id, resolved_id, note_map, None, None)?;
    }

    Ok(())
}

/// Collect all notes when no specific criteria provided (with filters)
pub fn collect_all_notes<'a>(
    state: &mut SelectionState<'a>,
    options: &ContextOptions<'_>,
    all_notes: &'a [Note],
    note_map: &'a HashMap<&'a str, &'a Note>,
    resolve_id: &dyn Fn(&str) -> Result<String>,
) -> Result<()> {
    if !options.selection.note_ids.is_empty()
        || options.selection.tag.is_some()
        || options.selection.moc_id.is_some()
        || options.selection.query.is_some()
        || options.walk.id.is_some()
    {
        return Ok(());
    }

    if options.selection.min_value.is_none() && options.selection.custom_filter.is_empty() {
        return Err(QipuError::UsageError(
            "no selection criteria provided. Use --note, --tag, --moc, --query, --walk, --min-value, or --custom-filter"
                .to_string(),
        ));
    }

    for note in all_notes {
        let resolved_id = resolve_id(note.id())?;
        state.add_note(note.id(), resolved_id, note_map, None, None)?;
    }

    Ok(())
}
