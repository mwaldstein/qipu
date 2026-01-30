//! Note selection for context command
//!
//! This module handles selecting notes for context bundles based on various
//! selection criteria like --note, --tag, --moc, --query, --walk.

pub mod expansion;
pub mod filter;
pub mod moc;
pub mod sources;
pub mod state;

pub use filter::filter_and_sort_selected_notes;
pub use state::SelectionState;

use crate::cli::Cli;
use crate::commands::context::types::{ContextOptions, SelectedNote};
use qipu_core::compaction::CompactionContext;
use qipu_core::error::Result;
use qipu_core::note::Note;
use qipu_core::store::Store;
use std::collections::HashMap;

/// Collect all selected notes based on selection criteria and expansion options
pub fn collect_selected_notes<'a>(
    cli: &Cli,
    store: &'a Store,
    options: &ContextOptions<'a>,
    all_notes: &'a [Note],
    compaction_ctx: &'a CompactionContext,
    note_map: &'a HashMap<&'a str, &'a Note>,
) -> Result<(Vec<SelectedNote<'a>>, HashMap<String, String>)> {
    let mut state = SelectionState::new();

    let resolve_id = |id: &str| -> Result<String> {
        if cli.no_resolve_compaction {
            Ok(id.to_string())
        } else {
            compaction_ctx.canon(id)
        }
    };

    // Primary selection sources
    sources::collect_from_walk(&mut state, cli, store, options, note_map, &resolve_id)?;
    sources::collect_from_note_ids(&mut state, options, note_map, &resolve_id)?;
    sources::collect_from_tag(&mut state, store, options, note_map, &resolve_id)?;
    moc::collect_from_moc(&mut state, store, options, note_map, &resolve_id)?;
    sources::collect_from_query(&mut state, cli, store, options, note_map, &resolve_id)?;
    sources::collect_all_notes(&mut state, options, all_notes, note_map, &resolve_id)?;

    // Expansion sources (depend on primary selections)
    expansion::collect_backlinks(&mut state, store, options, note_map, &resolve_id)?;
    expansion::collect_related_notes(&mut state, cli, store, options, note_map, &resolve_id)?;

    state::apply_via_map(&mut state);

    Ok((state.selected_notes, state.via_map))
}
