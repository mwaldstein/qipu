//! Link path command
use crate::cli::{Cli, OutputFormat};
use crate::lib::compaction::CompactionContext;
use crate::lib::error::Result;
use crate::lib::index::IndexBuilder;
use crate::lib::store::Store;

use super::{human, json, records, resolve_note_id, TreeOptions};

/// Execute the link path command
pub fn execute(
    cli: &Cli,
    store: &Store,
    from_id: &str,
    to_id: &str,
    opts: TreeOptions,
) -> Result<()> {
    use std::time::Instant;
    let start = Instant::now();

    // Resolve note IDs
    let from_resolved = resolve_note_id(store, from_id)?;
    let to_resolved = resolve_note_id(store, to_id)?;

    // Load or build the index
    let index = IndexBuilder::new(store).build()?;

    if cli.verbose {
        tracing::debug!(elapsed = ?start.elapsed(), "load_indexes");
    }

    let all_notes = store.list_notes()?;

    // Build compaction context if needed
    let compaction_ctx = if !cli.no_resolve_compaction {
        Some(CompactionContext::build(&all_notes)?)
    } else {
        None
    };

    let equivalence_map = if let Some(ref ctx) = compaction_ctx {
        Some(ctx.build_equivalence_map(&all_notes)?)
    } else {
        None
    };

    // Canonicalize the note IDs
    let canonical_from = if let Some(ref ctx) = compaction_ctx {
        ctx.canon(&from_resolved)?
    } else {
        from_resolved.clone()
    };
    let canonical_to = if let Some(ref ctx) = compaction_ctx {
        ctx.canon(&to_resolved)?
    } else {
        to_resolved.clone()
    };

    // Verify both notes exist (check canonical IDs)
    if !index.contains(&canonical_from) {
        return Err(crate::lib::error::QipuError::NoteNotFound {
            id: canonical_from.clone(),
        });
    }
    if !index.contains(&canonical_to) {
        return Err(crate::lib::error::QipuError::NoteNotFound {
            id: canonical_to.clone(),
        });
    }

    // Find path using weighted or unweighted traversal with compaction context
    let mut tree_opts = opts.clone();
    tree_opts.semantic_inversion = !cli.no_semantic_inversion;

    // bfs_find_path now handles both weighted (Dijkstra) and unweighted (BFS) based on ignore_value flag
    let result = crate::lib::graph::bfs_find_path(
        &index,
        store,
        &canonical_from,
        &canonical_to,
        &tree_opts,
        compaction_ctx.as_ref(),
        equivalence_map.as_ref(),
    )?;

    // Build note map for compaction percentage calculation
    // Per spec (specs/compaction.md lines 104-109)
    let note_map = if compaction_ctx.is_some() {
        let map: std::collections::HashMap<&str, &crate::lib::note::Note> =
            all_notes.iter().map(|n| (n.id(), n)).collect();
        Some(map)
    } else {
        None
    };

    // Output
    match cli.format {
        OutputFormat::Json => {
            json::output_path_json(cli, &result, compaction_ctx.as_ref(), note_map.as_ref())?;
        }
        OutputFormat::Human => {
            human::output_path_human(cli, &result, compaction_ctx.as_ref(), note_map.as_ref());
        }
        OutputFormat::Records => {
            records::output_path_records(
                &result,
                store,
                &opts,
                cli,
                compaction_ctx.as_ref(),
                note_map.as_ref(),
            );
        }
    }

    Ok(())
}
