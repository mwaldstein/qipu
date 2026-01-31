//! Link tree command
use std::collections::HashMap;

use qipu_core::compaction::CompactionContext;

/// Bundle of compaction-related context for tree traversal.
///
/// This struct holds all necessary context when building a tree view
/// with compaction support, avoiding complex tuple types.
#[derive(Debug)]
pub struct CompactionContextBundle {
    /// The compaction context if compaction is enabled.
    pub ctx: Option<CompactionContext>,
    /// Map of note IDs to their equivalent note IDs.
    pub equivalence_map: Option<HashMap<String, Vec<String>>>,
    /// The canonical note ID (after resolving aliases/compaction).
    pub canonical_id: String,
}

use crate::cli::{Cli, OutputFormat};
use qipu_core::error::Result;
use qipu_core::graph::TreeResult;
use qipu_core::index::IndexBuilder;
use qipu_core::note::Note;
use qipu_core::store::Store;

use super::{
    human::output_tree_human, json::output_tree_json, records_tree::output_tree_records,
    resolve_note_id, LinkOutputContext, TreeOptions,
};

/// Execute the link tree command
pub fn execute(cli: &Cli, store: &Store, id_or_path: &str, opts: TreeOptions) -> Result<()> {
    use std::time::Instant;
    let start = Instant::now();

    let note_id = resolve_note_id(store, id_or_path)?;
    let index = IndexBuilder::new(store).build()?;

    if cli.verbose {
        tracing::debug!(elapsed = ?start.elapsed(), "load_indexes");
    }

    let all_notes = store.list_notes()?;
    let bundle = build_compaction_context(cli, &all_notes, &note_id)?;

    if !index.contains(&bundle.canonical_id) {
        return Err(qipu_core::error::QipuError::NoteNotFound {
            id: bundle.canonical_id.clone(),
        });
    }

    let result = perform_traversal(
        cli,
        &index,
        store,
        &bundle.canonical_id,
        &opts,
        bundle.ctx.as_ref(),
        bundle.equivalence_map.as_ref(),
    )?;

    let note_map = build_note_map(bundle.ctx.as_ref(), &all_notes);

    match cli.format {
        OutputFormat::Json => {
            output_tree_json(
                cli,
                &result,
                bundle.ctx.as_ref(),
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
                bundle.ctx.as_ref(),
                note_map.as_ref(),
                &all_notes,
            );
        }
        OutputFormat::Records => {
            let ctx = LinkOutputContext::new(
                store,
                &index,
                cli,
                bundle.ctx.as_ref(),
                note_map.as_ref(),
                opts.max_chars,
                &all_notes,
            );
            output_tree_records(&result, &ctx, &opts);
        }
    }

    Ok(())
}

fn build_compaction_context(
    cli: &Cli,
    all_notes: &[Note],
    note_id: &str,
) -> Result<CompactionContextBundle> {
    let ctx = if !cli.no_resolve_compaction {
        Some(CompactionContext::build(all_notes)?)
    } else {
        None
    };

    let equivalence_map = if let Some(ref c) = ctx {
        Some(c.build_equivalence_map(all_notes)?)
    } else {
        None
    };

    let canonical_id = if let Some(ref c) = ctx {
        c.canon(note_id)?
    } else {
        note_id.to_string()
    };

    Ok(CompactionContextBundle {
        ctx,
        equivalence_map,
        canonical_id,
    })
}

fn perform_traversal(
    cli: &Cli,
    index: &qipu_core::index::Index,
    store: &Store,
    canonical_id: &str,
    opts: &TreeOptions,
    compaction_ctx: Option<&CompactionContext>,
    equivalence_map: Option<&HashMap<String, Vec<String>>>,
) -> Result<TreeResult> {
    let mut tree_opts = opts.clone();
    tree_opts.semantic_inversion = !cli.no_semantic_inversion;

    if tree_opts.ignore_value {
        Ok(qipu_core::graph::bfs_traverse(
            index,
            store,
            canonical_id,
            &tree_opts,
            compaction_ctx,
            equivalence_map,
        )?)
    } else {
        Ok(qipu_core::graph::dijkstra_traverse(
            index,
            store,
            canonical_id,
            &tree_opts,
            compaction_ctx,
            equivalence_map,
        )?)
    }
}

fn build_note_map<'a>(
    compaction_ctx: Option<&CompactionContext>,
    all_notes: &'a [Note],
) -> Option<HashMap<&'a str, &'a Note>> {
    if compaction_ctx.is_some() {
        let map: HashMap<&'a str, &'a Note> = all_notes.iter().map(|n| (n.id(), n)).collect();
        Some(map)
    } else {
        None
    }
}
