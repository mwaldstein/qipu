use crate::cli::Cli;
use crate::commands::context::types::{ContextOptions, SelectedNote};
use crate::commands::context::walk;
use qipu_core::compaction::CompactionContext;
use qipu_core::error::{QipuError, Result};
use qipu_core::index::IndexBuilder;
use qipu_core::note::Note;
use qipu_core::similarity::SimilarityEngine;
use qipu_core::store::Store;

use qipu_core::db::Database;
use qipu_core::note::LinkType;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;
use tracing::debug;

type CustomFilter = Arc<dyn Fn(&HashMap<String, serde_yaml::Value>) -> bool>;

/// Get note IDs linked from a MOC (including the MOC itself) with their link types
/// Returns (note_id, link_type) pairs. For the MOC itself, link_type is None.
pub fn get_moc_linked_ids(
    db: &Database,
    moc_id: &str,
    transitive: bool,
) -> Result<Vec<(String, Option<LinkType>)>> {
    let start = Instant::now();

    debug!(moc_id, transitive, "get_moc_linked_ids");

    let mut result = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back((moc_id.to_string(), None));

    visited.insert(moc_id.to_string());
    result.push((moc_id.to_string(), None));

    while let Some((current_id, _)) = queue.pop_front() {
        // Get outbound edges from current note
        let edges = db.get_outbound_edges(&current_id)?;

        for edge in edges {
            if visited.insert(edge.to.clone()) {
                let link_type = edge.link_type.clone();
                result.push((edge.to.clone(), Some(link_type.clone())));

                // If transitive and target is a MOC, add to queue for further traversal
                if transitive {
                    if let Some(meta) = db.get_note_metadata(&edge.to)? {
                        if meta.note_type.is_moc() {
                            queue.push_back((edge.to.clone(), Some(link_type)));
                        }
                    }
                }
            }
        }
    }

    debug!(
        result_count = result.len(),
        elapsed = ?start.elapsed(),
        "get_moc_linked_ids_complete"
    );

    Ok(result)
}

/// Collect all selected notes based on selection criteria and expansion options
pub fn collect_selected_notes<'a>(
    cli: &Cli,
    store: &'a Store,
    options: &ContextOptions<'a>,
    all_notes: &'a [Note],
    compaction_ctx: &'a CompactionContext,
    note_map: &'a HashMap<&'a str, &'a Note>,
) -> Result<(Vec<SelectedNote<'a>>, HashMap<String, String>)> {
    let mut selected_notes: Vec<SelectedNote<'a>> = Vec::new();
    let mut seen_ids: HashSet<String> = HashSet::new();
    let mut via_map: HashMap<String, String> = HashMap::new();

    let resolve_id = |id: &str| -> Result<String> {
        if cli.no_resolve_compaction {
            Ok(id.to_string())
        } else {
            compaction_ctx.canon(id)
        }
    };

    if let Some(walk_id) = options.walk_id {
        let walked_ids = walk::walk_for_context(
            cli,
            store,
            walk_id,
            options.walk_direction,
            options.walk_max_hops,
            options.walk_type,
            options.walk_exclude_type,
            options.walk_typed_only,
            options.walk_inline_only,
            options.walk_max_nodes,
            options.walk_max_edges,
            options.walk_max_fanout,
            options.walk_min_value,
            options.walk_ignore_value,
        )?;

        for id in &walked_ids {
            let resolved_id = resolve_id(id)?;
            if seen_ids.insert(resolved_id.clone()) {
                let note =
                    note_map
                        .get(resolved_id.as_str())
                        .ok_or_else(|| QipuError::NoteNotFound {
                            id: resolved_id.clone(),
                        })?;
                selected_notes.push(SelectedNote {
                    note,
                    via: Some(format!("walk:{}", walk_id)),
                    link_type: None,
                });
            }
        }
    }

    for id in options.note_ids {
        let resolved_id = resolve_id(id)?;
        if seen_ids.insert(resolved_id.clone()) {
            let note =
                note_map
                    .get(resolved_id.as_str())
                    .ok_or_else(|| QipuError::NoteNotFound {
                        id: resolved_id.clone(),
                    })?;
            selected_notes.push(SelectedNote {
                note,
                via: None,
                link_type: None,
            });
        }
    }

    if let Some(tag_name) = options.tag {
        let notes_with_tag = store.db().list_notes(None, Some(tag_name), None)?;
        for note in &notes_with_tag {
            let resolved_id = resolve_id(&note.id)?;
            if seen_ids.insert(resolved_id.clone()) {
                let note =
                    note_map
                        .get(resolved_id.as_str())
                        .ok_or_else(|| QipuError::NoteNotFound {
                            id: resolved_id.clone(),
                        })?;
                selected_notes.push(SelectedNote {
                    note,
                    via: None,
                    link_type: None,
                });
            }
        }
    }

    if let Some(moc) = options.moc_id {
        let linked_ids = get_moc_linked_ids(store.db(), moc, options.transitive)?;
        for (id, link_type) in linked_ids {
            let resolved_id = resolve_id(&id)?;
            if seen_ids.insert(resolved_id.clone()) {
                let note =
                    note_map
                        .get(resolved_id.as_str())
                        .ok_or_else(|| QipuError::NoteNotFound {
                            id: resolved_id.clone(),
                        })?;
                selected_notes.push(SelectedNote {
                    note,
                    via: None,
                    link_type,
                });
            }
        }
    }

    if let Some(q) = options.query {
        let results = store
            .db()
            .search(q, None, None, None, None, 100, &store.config().search)?;
        for result in results {
            let resolved_id = resolve_id(&result.id)?;
            let via_source = if !cli.no_resolve_compaction && resolved_id != result.id {
                Some(result.id.clone())
            } else {
                None
            };

            if let Some(via_id) = via_source {
                via_map.entry(resolved_id.clone()).or_insert(via_id);
            }

            if seen_ids.insert(resolved_id.clone()) {
                let note =
                    note_map
                        .get(resolved_id.as_str())
                        .ok_or_else(|| QipuError::NoteNotFound {
                            id: resolved_id.clone(),
                        })?;
                selected_notes.push(SelectedNote {
                    note,
                    via: None,
                    link_type: None,
                });
            }
        }
    }

    if options.backlinks {
        let mut backlink_notes: Vec<(String, String, LinkType)> = Vec::new();

        for selected_note in &selected_notes {
            let note_id = selected_note.note.id();
            let backlinks = store.db().get_backlinks(note_id)?;

            for backlink in backlinks {
                if !seen_ids.contains(&backlink.from) {
                    backlink_notes.push((backlink.from, note_id.to_string(), backlink.link_type));
                }
            }
        }

        for (backlink_id, source_id, link_type) in backlink_notes {
            let resolved_id = resolve_id(&backlink_id)?;
            via_map
                .entry(resolved_id.clone())
                .or_insert_with(|| format!("backlink:{}", source_id));
            if seen_ids.insert(resolved_id.clone()) {
                let note =
                    note_map
                        .get(resolved_id.as_str())
                        .ok_or_else(|| QipuError::NoteNotFound {
                            id: resolved_id.clone(),
                        })?;
                selected_notes.push(SelectedNote {
                    note,
                    via: None,
                    link_type: Some(link_type),
                });
            }
        }
    }

    if let Some(threshold) = options.related_threshold {
        let start = Instant::now();
        let index = IndexBuilder::new(store).build()?;

        if cli.verbose {
            debug!(elapsed = ?start.elapsed(), "load_indexes");
        }

        let engine = SimilarityEngine::new(&index);

        let mut linked_ids: HashSet<String> = HashSet::new();
        for selected_note in &selected_notes {
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

        for selected_note in &selected_notes {
            let note_id = selected_note.note.id();

            let similar = engine.find_similar(note_id, 100, threshold);
            for sim in similar {
                if !seen_ids.contains(&sim.id) && !linked_ids.contains(&sim.id) {
                    related_notes.push((sim.id, sim.score, "similarity".to_string()));
                }
            }

            let shared_tags = engine.find_by_shared_tags(note_id, 100);
            for sim in shared_tags {
                if !seen_ids.contains(&sim.id) && !linked_ids.contains(&sim.id) {
                    related_notes.push((sim.id, sim.score, "shared-tags".to_string()));
                }
            }

            let two_hop = engine.find_by_2hop_neighborhood(note_id, 100);
            for sim in two_hop {
                if !seen_ids.contains(&sim.id) && !linked_ids.contains(&sim.id) {
                    related_notes.push((sim.id, sim.score, "2hop".to_string()));
                }
            }
        }

        related_notes.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.partial_cmp(&a.1).unwrap()));
        related_notes.dedup_by(|a, b| a.0 == b.0);
        related_notes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        for (related_id, score, method) in related_notes {
            if seen_ids.contains(&related_id) {
                continue;
            }
            let resolved_id = resolve_id(&related_id)?;
            via_map
                .entry(resolved_id.clone())
                .or_insert_with(|| format!("{}:{:.2}", method, score));
            if seen_ids.insert(resolved_id.clone()) {
                let note =
                    note_map
                        .get(resolved_id.as_str())
                        .ok_or_else(|| QipuError::NoteNotFound {
                            id: resolved_id.clone(),
                        })?;
                selected_notes.push(SelectedNote {
                    note,
                    via: Some(related_id.clone()),
                    link_type: None,
                });
            }
        }
    }

    for selected in &mut selected_notes {
        if let Some(via) = via_map.get(selected.note.id()) {
            selected.via = Some(via.clone());
        }
    }

    if options.note_ids.is_empty()
        && options.tag.is_none()
        && options.moc_id.is_none()
        && options.query.is_none()
        && options.walk_id.is_none()
    {
        if options.min_value.is_none() && options.custom_filter.is_empty() {
            return Err(QipuError::UsageError(
                "no selection criteria provided. Use --note, --tag, --moc, --query, --walk, --min-value, or --custom-filter"
                    .to_string(),
            ));
        }

        for note in all_notes {
            let resolved_id = resolve_id(note.id())?;

            if seen_ids.insert(resolved_id.clone()) {
                let note_from_map =
                    note_map
                        .get(resolved_id.as_str())
                        .ok_or_else(|| QipuError::NoteNotFound {
                            id: resolved_id.clone(),
                        })?;
                selected_notes.push(SelectedNote {
                    note: note_from_map,
                    via: None,
                    link_type: None,
                });
            }
        }
    }

    Ok((selected_notes, via_map))
}

/// Filter and sort selected notes based on min-value, custom filters, and sorting criteria
pub fn filter_and_sort_selected_notes(
    cli: &Cli,
    selected_notes: &mut Vec<SelectedNote<'_>>,
    options: &ContextOptions<'_>,
) {
    if let Some(min_value) = options.min_value {
        let before_count = selected_notes.len();
        selected_notes.retain(|selected| {
            let note_value = selected.note.frontmatter.value.unwrap_or(50);
            note_value >= min_value
        });
        let after_count = selected_notes.len();

        if cli.verbose && before_count > after_count {
            debug!(
                min_value,
                before_count,
                after_count,
                filtered = before_count - after_count,
                "min_value_filter"
            );
        }
    }

    if !options.custom_filter.is_empty() {
        let before_count = selected_notes.len();

        let filters: Vec<CustomFilter> = options
            .custom_filter
            .iter()
            .map(|filter_expr| {
                crate::commands::context::filter::parse_custom_filter_expression(filter_expr)
            })
            .collect::<Result<_>>()
            .unwrap();

        selected_notes.retain(|selected| {
            filters
                .iter()
                .all(|filter| filter(&selected.note.frontmatter.custom))
        });

        let after_count = selected_notes.len();

        if cli.verbose && before_count > after_count {
            debug!(
                filter_count = options.custom_filter.len(),
                before_count,
                after_count,
                filtered = before_count - after_count,
                "custom_filters"
            );
        }
    }

    selected_notes.sort_by(|a, b| {
        let a_verified = a.note.frontmatter.verified.unwrap_or(false);
        let b_verified = b.note.frontmatter.verified.unwrap_or(false);

        let link_priority = |link_type: &Option<LinkType>| -> u8 {
            match link_type {
                Some(lt) if lt.as_str() == "part-of" || lt.as_str() == "supports" => 0,
                Some(lt) if lt.as_str() != "related" => 1,
                Some(_) => 2,
                None => 1,
            }
        };

        let a_priority = link_priority(&a.link_type);
        let b_priority = link_priority(&b.link_type);

        b_verified
            .cmp(&a_verified)
            .then_with(|| a_priority.cmp(&b_priority))
            .then_with(
                || match (&a.note.frontmatter.created, &b.note.frontmatter.created) {
                    (Some(a_created), Some(b_created)) => a_created.cmp(b_created),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                },
            )
            .then_with(|| a.note.id().cmp(b.note.id()))
    });
}
