use super::state::SelectionState;
use crate::cli::Cli;
use crate::commands::context::types::ContextOptions;
use qipu_core::error::Result;
use qipu_core::index::IndexBuilder;
use qipu_core::note::{LinkType, Note};
use qipu_core::similarity::SimilarityEngine;
use qipu_core::store::Store;
use std::collections::{HashMap, HashSet};
use std::time::Instant;
use tracing::debug;

/// Collect backlinks to currently selected notes
pub fn collect_backlinks<'a>(
    state: &mut SelectionState<'a>,
    store: &'a Store,
    options: &ContextOptions<'_>,
    note_map: &'a HashMap<&'a str, &'a Note>,
    resolve_id: &dyn Fn(&str) -> Result<String>,
) -> Result<()> {
    if !options.expansion.backlinks {
        return Ok(());
    }

    let mut backlink_notes: Vec<(String, String, LinkType)> = Vec::new();

    for selected_note in &state.selected_notes {
        let note_id = selected_note.note.id();
        let backlinks = store.db().get_backlinks(note_id)?;

        for backlink in backlinks {
            if !state.seen_ids.contains(&backlink.from) {
                backlink_notes.push((backlink.from, note_id.to_string(), backlink.link_type));
            }
        }
    }

    for (backlink_id, source_id, link_type) in backlink_notes {
        let resolved_id = resolve_id(&backlink_id)?;
        let via = format!("backlink:{}", source_id);
        state
            .via_map
            .entry(resolved_id.clone())
            .or_insert_with(|| via.clone());
        state.add_note(
            &backlink_id,
            resolved_id,
            note_map,
            Some(via),
            Some(link_type),
        )?;
    }

    Ok(())
}

/// Collect related notes based on similarity
pub fn collect_related_notes<'a>(
    state: &mut SelectionState<'a>,
    cli: &Cli,
    store: &'a Store,
    options: &ContextOptions<'_>,
    note_map: &'a HashMap<&'a str, &'a Note>,
    resolve_id: &dyn Fn(&str) -> Result<String>,
) -> Result<()> {
    let Some(threshold) = options.expansion.related_threshold else {
        return Ok(());
    };

    let start = Instant::now();
    let index = IndexBuilder::new(store).build()?;

    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "load_indexes");
    }

    let engine = SimilarityEngine::new(&index);

    let mut linked_ids: HashSet<String> = HashSet::new();
    for selected_note in &state.selected_notes {
        let note_id = selected_note.note.id();
        let outbound_edges = index.get_outbound_edges(note_id);
        for edge in outbound_edges {
            linked_ids.insert(edge.to.clone());
        }
        let inbound_edges = index.get_inbound_edges(note_id);
        for edge in inbound_edges {
            linked_ids.insert(edge.from.clone());
        }
    }

    let mut related_notes: Vec<(String, f64, String)> = Vec::new();

    for selected_note in &state.selected_notes {
        let note_id = selected_note.note.id();

        let similar = engine.find_similar(note_id, 100, threshold);
        for sim in similar {
            if !state.seen_ids.contains(&sim.id) && !linked_ids.contains(&sim.id) {
                related_notes.push((sim.id, sim.score, "similarity".to_string()));
            }
        }

        let shared_tags = engine.find_by_shared_tags(note_id, 100);
        for sim in shared_tags {
            if !state.seen_ids.contains(&sim.id) && !linked_ids.contains(&sim.id) {
                related_notes.push((sim.id, sim.score, "shared-tags".to_string()));
            }
        }

        let two_hop = engine.find_by_2hop_neighborhood(note_id, 100);
        for sim in two_hop {
            if !state.seen_ids.contains(&sim.id) && !linked_ids.contains(&sim.id) {
                related_notes.push((sim.id, sim.score, "2hop".to_string()));
            }
        }
    }

    related_notes.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.partial_cmp(&a.1).unwrap()));
    related_notes.dedup_by(|a, b| a.0 == b.0);
    related_notes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    for (related_id, score, method) in related_notes {
        if state.seen_ids.contains(&related_id) {
            continue;
        }
        let resolved_id = resolve_id(&related_id)?;
        state
            .via_map
            .entry(resolved_id.clone())
            .or_insert_with(|| format!("{}:{:.2}", method, score));
        state.add_note(
            &related_id.clone(),
            resolved_id,
            note_map,
            Some(related_id),
            None,
        )?;
    }

    Ok(())
}
