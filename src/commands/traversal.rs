//! Shared command-layer graph traversal setup.

use std::collections::HashMap;

use crate::cli::Cli;
use qipu_core::bail_unsupported;
use qipu_core::compaction::CompactionContext;
use qipu_core::error::{QipuError, Result};
use qipu_core::graph::{Direction, TreeOptions, TreeResult};
use qipu_core::index::{Index, IndexBuilder};
use qipu_core::note::Note;
use qipu_core::store::Store;

/// Loaded context needed to run and render a traversal.
pub struct TraversalContext {
    pub index: Index,
    pub all_notes: Vec<Note>,
    pub compaction_ctx: Option<CompactionContext>,
    pub canonical_id: String,
    equivalence_map: Option<HashMap<String, Vec<String>>>,
}

impl TraversalContext {
    pub fn note_map(&self) -> Option<HashMap<&str, &Note>> {
        self.compaction_ctx
            .as_ref()
            .map(|_| self.all_notes.iter().map(|n| (n.id(), n)).collect())
    }

    fn equivalence_map(&self) -> Option<&HashMap<String, Vec<String>>> {
        self.equivalence_map.as_ref()
    }
}

pub fn parse_direction(direction: &str) -> Result<Direction> {
    match direction.to_lowercase().as_str() {
        "out" => Ok(Direction::Out),
        "in" => Ok(Direction::In),
        "both" => Ok(Direction::Both),
        _ => bail_unsupported!("direction", direction, "out, in, both"),
    }
}

pub fn build_context(cli: &Cli, store: &Store, id_or_path: &str) -> Result<TraversalContext> {
    let note_id = crate::commands::link::resolve_note_id(store, id_or_path)?;
    let index = IndexBuilder::new(store).build()?;
    let all_notes = store.list_notes()?;

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

    let canonical_id = if let Some(ref ctx) = compaction_ctx {
        ctx.canon(&note_id)?
    } else {
        note_id
    };

    if !index.contains(&canonical_id) {
        return Err(QipuError::NoteNotFound {
            id: canonical_id.clone(),
        });
    }

    Ok(TraversalContext {
        index,
        all_notes,
        compaction_ctx,
        canonical_id,
        equivalence_map,
    })
}

pub fn run_tree(
    cli: &Cli,
    store: &Store,
    ctx: &TraversalContext,
    opts: &TreeOptions,
) -> Result<TreeResult> {
    let mut tree_opts = opts.clone();
    tree_opts.semantic_inversion = !cli.no_semantic_inversion;

    if tree_opts.ignore_value {
        qipu_core::graph::bfs_traverse(
            &ctx.index,
            store,
            &ctx.canonical_id,
            &tree_opts,
            ctx.compaction_ctx.as_ref(),
            ctx.equivalence_map(),
        )
    } else {
        qipu_core::graph::dijkstra_traverse(
            &ctx.index,
            store,
            &ctx.canonical_id,
            &tree_opts,
            ctx.compaction_ctx.as_ref(),
            ctx.equivalence_map(),
        )
    }
}
