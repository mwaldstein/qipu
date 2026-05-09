//! Graph walk support for context command
//!
//! Implements `qipu context --walk <id>` which traverses the graph from
//! a starting note and bundles all traversed notes.

use crate::cli::Cli;
use crate::commands::traversal;
use qipu_core::error::Result;
use qipu_core::graph::{HopCost, TreeOptions};
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
    let tree_opts = TreeOptions {
        direction: traversal::parse_direction(walk_direction)?,
        max_hops: HopCost::from(walk_max_hops),
        type_include: walk_type,
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

    let traversal_ctx = traversal::build_context(cli, store, walk_id)?;
    let result = traversal::run_tree(cli, store, &traversal_ctx, &tree_opts)?;

    let note_ids: Vec<String> = result.notes.into_iter().map(|n| n.id).collect();
    Ok(note_ids)
}
