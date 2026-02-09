//! `qipu search` command - search notes
//!
//! Per spec (specs/cli-interface.md, specs/indexing-search.md):
//! - `qipu search <query>` - search titles + bodies
//! - `--type` filter
//! - `--tag` filter
//! - Result ranking: title > exact tag > body, recency boost
//! - Compaction resolution (specs/compaction.md): show canonical digests with via= annotations
//! - `--interactive` - fzf-style picker for selecting from results

pub mod format;

use std::time::Instant;

use tracing::debug;

use crate::cli::Cli;
use crate::commands::picker::{pick_single, PickerItem};
use qipu_core::compaction::CompactionContext;
use qipu_core::error::Result;
use qipu_core::note::NoteType;
use qipu_core::search;
use qipu_core::store::Store;

use self::format::{output_human, output_json, output_records};

/// Execute the search command
#[allow(clippy::too_many_arguments)]
pub fn execute(
    cli: &Cli,
    store: &Store,
    query: &str,
    type_filter: Option<NoteType>,
    tag_filter: Option<&str>,
    exclude_mocs: bool,
    min_value: Option<u8>,
    sort: Option<&str>,
    interactive: bool,
) -> Result<()> {
    let start = Instant::now();

    // Resolve tag aliases for filtering
    let equivalent_tags = tag_filter.map(|t| store.config().get_equivalent_tags(t));

    if cli.verbose {
        debug!(
            query,
            ?type_filter,
            ?tag_filter,
            ?equivalent_tags,
            exclude_mocs,
            ?min_value,
            ?sort,
            "search_params"
        );
    }

    let results = store.db().search(
        query,
        type_filter,
        tag_filter,
        min_value,
        equivalent_tags.as_deref(),
        200,
        &store.config().search,
    )?;

    if cli.verbose {
        debug!(result_count = results.len(), elapsed = ?start.elapsed(), "search");
    }

    let needs_compaction = !cli.no_resolve_compaction
        || cli.with_compaction_ids
        || cli.compaction_depth.is_some()
        || cli.compaction_max_nodes.is_some();

    let all_notes = if needs_compaction {
        store.list_notes()?
    } else {
        Vec::new()
    };

    let compaction_ctx = if needs_compaction {
        if cli.verbose {
            debug!(note_count = all_notes.len(), "build_compaction_context");
        }
        Some(CompactionContext::build(&all_notes)?)
    } else {
        None
    };

    let compaction_note_map = if needs_compaction {
        Some(CompactionContext::build_note_map(&all_notes))
    } else {
        None
    };

    let (results, notes_cache, _compacts_count) = search::process_search_results(
        results,
        !cli.no_resolve_compaction,
        store,
        &compaction_ctx,
        &compaction_note_map,
        exclude_mocs,
        sort,
    );

    // Handle interactive picker mode
    if interactive {
        let items: Vec<PickerItem> = results.iter().map(PickerItem::from_search_result).collect();

        if items.is_empty() {
            if !cli.quiet {
                println!("No results found for '{}'", query);
            }
            return Ok(());
        }

        let prompt = format!("Select a note for '{}'", query);
        if let Some(selected_id) = pick_single(&items, &prompt)? {
            // Output just the selected ID for piping to other commands
            println!("{}", selected_id);
        }
        return Ok(());
    }

    match cli.format {
        crate::cli::OutputFormat::Json => {
            output_json(
                cli,
                store,
                &results,
                &compaction_ctx,
                &notes_cache,
                &compaction_note_map,
            )?;
        }
        crate::cli::OutputFormat::Human => {
            output_human(
                cli,
                store,
                &results,
                &compaction_ctx,
                &notes_cache,
                &compaction_note_map,
                query,
            );
        }
        crate::cli::OutputFormat::Records => {
            output_records(
                cli,
                store,
                &results,
                &compaction_ctx,
                &notes_cache,
                &compaction_note_map,
                query,
            );
        }
    }

    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "execute_command");
    }

    Ok(())
}

#[cfg(test)]
mod tests;
