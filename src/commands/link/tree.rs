//! Link tree command
use std::collections::HashMap;

use crate::cli::{Cli, OutputFormat};
use qipu_core::compaction::CompactionContext;
use qipu_core::error::Result;
use qipu_core::index::IndexBuilder;
use qipu_core::note::Note;
use qipu_core::store::Store;

use super::{
    human::output_tree_human, json::output_tree_json, records::output_tree_records,
    resolve_note_id, TreeOptions,
};

/// Execute the link tree command
pub fn execute(cli: &Cli, store: &Store, id_or_path: &str, opts: TreeOptions) -> Result<()> {
    use std::time::Instant;
    let start = Instant::now();

    // Resolve note ID
    let note_id = resolve_note_id(store, id_or_path)?;

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

    // Canonicalize the root note ID
    let canonical_id = if let Some(ref ctx) = compaction_ctx {
        ctx.canon(&note_id)?
    } else {
        note_id.clone()
    };

    // Verify note exists (check canonical ID)
    if !index.contains(&canonical_id) {
        return Err(qipu_core::error::QipuError::NoteNotFound {
            id: canonical_id.clone(),
        });
    }

    // Perform BFS traversal with compaction context
    let mut tree_opts = opts.clone();
    tree_opts.semantic_inversion = !cli.no_semantic_inversion;

    let result = if tree_opts.ignore_value {
        qipu_core::graph::bfs_traverse(
            &index,
            store,
            &canonical_id,
            &tree_opts,
            compaction_ctx.as_ref(),
            equivalence_map.as_ref(),
        )?
    } else {
        qipu_core::graph::dijkstra_traverse(
            &index,
            store,
            &canonical_id,
            &tree_opts,
            compaction_ctx.as_ref(),
            equivalence_map.as_ref(),
        )?
    };

    // Build note map for compaction percentage calculation
    // Per spec (specs/compaction.md lines 104-109)
    let note_map = if compaction_ctx.is_some() {
        let map: HashMap<&str, &Note> = all_notes.iter().map(|n| (n.id(), n)).collect();
        Some(map)
    } else {
        None
    };

    // Output
    match cli.format {
        OutputFormat::Json => {
            output_tree_json(
                cli,
                &result,
                compaction_ctx.as_ref(),
                note_map.as_ref(),
                &all_notes,
            )?;
        }
        OutputFormat::Human => {
            output_tree_human(
                cli,
                &result,
                &index,
                store,
                compaction_ctx.as_ref(),
                note_map.as_ref(),
                &all_notes,
            );
        }
        OutputFormat::Records => {
            output_tree_records(
                &result,
                store,
                &opts,
                cli,
                compaction_ctx.as_ref(),
                note_map.as_ref(),
                &all_notes,
            );
        }
    }

    Ok(())
}
