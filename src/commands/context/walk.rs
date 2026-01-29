//! Graph walk support for context command
//!
//! Implements `qipu context --walk <id>` which traverses the graph from
//! a starting note and bundles all traversed notes.

use crate::cli::Cli;
use qipu_core::compaction::CompactionContext;
use qipu_core::error::{QipuError, Result};
use qipu_core::graph::{Direction, HopCost, TreeOptions};
use qipu_core::index::IndexBuilder;
use qipu_core::store::Store;

/// Perform graph walk from a starting note and return the list of note IDs
#[allow(clippy::too_many_arguments)]
pub fn walk_for_context(
    cli: &Cli,
    store: &Store,
    walk_id: &str,
    walk_direction: &str,
    walk_max_hops: u32,
    walk_type: &[String],
    walk_exclude_type: &[String],
    walk_typed_only: bool,
    walk_inline_only: bool,
    walk_max_nodes: Option<usize>,
    walk_max_edges: Option<usize>,
    walk_max_fanout: Option<usize>,
    walk_min_value: Option<u8>,
    walk_ignore_value: bool,
) -> Result<Vec<String>> {
    // Resolve note ID (reuse link module's resolver)
    let note_id = crate::commands::link::resolve_note_id(store, walk_id)?;

    // Load or build the index
    let index = IndexBuilder::new(store).build()?;

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

    // Verify note exists
    if !index.contains(&canonical_id) {
        return Err(QipuError::NoteNotFound {
            id: canonical_id.clone(),
        });
    }

    // Parse direction
    let direction = match walk_direction.to_lowercase().as_str() {
        "out" => Direction::Out,
        "in" => Direction::In,
        "both" => Direction::Both,
        _ => {
            return Err(QipuError::Other(format!(
                "invalid direction '{}': expected one of: out, in, both",
                walk_direction
            )))
        }
    };

    // Build tree options
    let tree_opts = TreeOptions {
        direction,
        max_hops: HopCost::from(walk_max_hops),
        type_include: walk_type.to_vec(),
        type_exclude: walk_exclude_type.to_vec(),
        typed_only: walk_typed_only,
        inline_only: walk_inline_only,
        max_nodes: walk_max_nodes,
        max_edges: walk_max_edges,
        max_fanout: walk_max_fanout,
        semantic_inversion: !cli.no_semantic_inversion,
        min_value: walk_min_value,
        ignore_value: walk_ignore_value,
        max_chars: None,
    };

    // Perform traversal
    let result = if walk_ignore_value {
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

    // Extract note IDs from traversal result
    let note_ids: Vec<String> = result.notes.into_iter().map(|n| n.id).collect();

    Ok(note_ids)
}
