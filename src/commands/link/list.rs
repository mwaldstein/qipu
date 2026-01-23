//! Link list command
use crate::cli::{Cli, OutputFormat};
use crate::lib::compaction::CompactionContext;
use crate::lib::error::Result;
use crate::lib::index::IndexBuilder;
use crate::lib::store::Store;

use super::{
    filter_and_convert, filter_and_convert_inbound, human, json, records, resolve_note_id,
    Direction,
};

/// Execute the link list command
///
/// Lists all links for a note, with optional direction and type filters.
#[allow(clippy::too_many_arguments)]
pub fn execute(
    cli: &Cli,
    store: &Store,
    id_or_path: &str,
    direction: Direction,
    type_filter: Option<&str>,
    typed_only: bool,
    inline_only: bool,
    max_chars: Option<usize>,
) -> Result<()> {
    use std::time::Instant;
    let start = Instant::now();

    // Resolve note ID
    let note_id = resolve_note_id(store, id_or_path)?;

    // Load or build of index
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

    // Canonicalize the note ID to get which note's links we should show
    let canonical_id = if let Some(ref ctx) = compaction_ctx {
        ctx.canon(&note_id)?
    } else {
        note_id.clone()
    };

    let display_id = if compaction_ctx.is_some() {
        canonical_id.clone()
    } else {
        note_id.clone()
    };

    // Verify canonical note exists
    if !index.contains(&canonical_id) {
        return Err(crate::lib::error::QipuError::NoteNotFound {
            id: canonical_id.clone(),
        });
    }

    // Collect all raw IDs that map to this canonical ID (for gathering edges)
    let source_ids = equivalence_map
        .as_ref()
        .and_then(|map| map.get(&canonical_id).cloned())
        .unwrap_or_else(|| vec![canonical_id.clone()]);

    // Collect links based on direction
    let mut entries = Vec::new();

    // Outbound edges (links FROM this note or any note it compacts)
    if direction == Direction::Out || direction == Direction::Both {
        for source_id in &source_ids {
            for edge in index.get_outbound_edges(source_id) {
                if let Some(mut entry) =
                    filter_and_convert(edge, "out", &index, type_filter, typed_only, inline_only)
                {
                    // Canonicalize the target ID if compaction is enabled
                    if let Some(ref ctx) = compaction_ctx {
                        entry.id = ctx.canon(&entry.id)?;
                        if entry.id == canonical_id {
                            continue;
                        }
                        // Update title if it changed due to canonicalization
                        if let Some(meta) = index.get_metadata(&entry.id) {
                            entry.title = Some(meta.title.clone());
                        }
                    }
                    entries.push(entry);
                }
            }
        }
    }

    // Inbound edges (backlinks TO this note or any note it compacts)
    if direction == Direction::In || direction == Direction::Both {
        for source_id in &source_ids {
            for edge in index.get_inbound_edges(source_id) {
                if let Some(mut entry) = filter_and_convert_inbound(
                    edge,
                    &index,
                    store,
                    type_filter,
                    typed_only,
                    inline_only,
                    !cli.no_semantic_inversion,
                ) {
                    // Canonicalize the source ID if compaction is enabled
                    if let Some(ref ctx) = compaction_ctx {
                        entry.id = ctx.canon(&entry.id)?;
                        if entry.id == canonical_id {
                            continue;
                        }
                        // Update title if it changed due to canonicalization
                        if let Some(meta) = index.get_metadata(&entry.id) {
                            entry.title = Some(meta.title.clone());
                        }
                    }
                    entries.push(entry);
                }
            }
        }
    }

    // Remove duplicates that may have been created by canonicalization
    entries.sort_by(|a, b| {
        a.direction
            .cmp(&b.direction)
            .then_with(|| a.link_type.cmp(&b.link_type))
            .then_with(|| a.id.cmp(&b.id))
    });
    entries
        .dedup_by(|a, b| a.direction == b.direction && a.link_type == b.link_type && a.id == b.id);

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
            json::output_json(cli, &entries, compaction_ctx.as_ref(), note_map.as_ref())?;
        }
        OutputFormat::Human => {
            human::output_human(
                cli,
                &entries,
                &display_id,
                compaction_ctx.as_ref(),
                note_map.as_ref(),
            );
        }
        OutputFormat::Records => {
            records::output_records(
                &entries,
                store,
                &index,
                &display_id,
                direction,
                cli,
                compaction_ctx.as_ref(),
                note_map.as_ref(),
                max_chars,
            );
        }
    }

    Ok(())
}
