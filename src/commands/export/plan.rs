use super::ExportOptions;
use qipu_core::compaction::CompactionContext;
use qipu_core::error::{QipuError, Result};
use qipu_core::graph::{Direction, HopCost, TreeOptions};
use qipu_core::index::Index;
use qipu_core::note::Note;
use qipu_core::store::Store;
use std::collections::HashSet;

/// Collect notes based on selection criteria
pub fn collect_notes(
    store: &Store,
    index: &Index,
    all_notes: &[Note],
    options: &ExportOptions,
) -> Result<Vec<Note>> {
    let mut selected_notes: Vec<Note> = Vec::new();
    let mut seen_ids = HashSet::new();

    // Selection by explicit note IDs
    for id in options.note_ids {
        if seen_ids.insert(id.clone()) {
            match store.get_note(id) {
                Ok(note) => selected_notes.push(note),
                Err(_) => {
                    return Err(QipuError::NoteNotFound { id: id.clone() });
                }
            }
        }
    }

    // Selection by tag
    if let Some(tag_name) = options.tag {
        for note in all_notes {
            if note.frontmatter.tags.contains(&tag_name.to_string())
                && seen_ids.insert(note.id().to_string())
            {
                selected_notes.push(note.clone());
            }
        }
    }

    // Selection by MOC (same logic as context command)
    if let Some(moc_id) = options.moc_id {
        let linked_notes = get_moc_linked_notes(store, index, moc_id)?;
        for note in linked_notes {
            if seen_ids.insert(note.id().to_string()) {
                selected_notes.push(note);
            }
        }
    }

    // Selection by query
    if let Some(q) = options.query {
        let results =
            store
                .db()
                .search(q, None, None, None, None, 10_000, &store.config().search)?;
        for result in results {
            if seen_ids.insert(result.id.clone()) {
                if let Ok(note) = store.get_note(&result.id) {
                    selected_notes.push(note);
                }
            }
        }
    }

    // If no selection criteria provided, return error
    if options.note_ids.is_empty()
        && options.tag.is_none()
        && options.moc_id.is_none()
        && options.query.is_none()
    {
        return Err(QipuError::Other(
            "no selection criteria provided. Use --note, --tag, --moc, or --query".to_string(),
        ));
    }

    // Graph traversal expansion if requested
    if !selected_notes.is_empty() && options.max_hops > 0 {
        let initial_ids: Vec<String> = selected_notes.iter().map(|n| n.id().to_string()).collect();

        let traversal_options = TreeOptions {
            direction: Direction::Both,
            max_hops: HopCost::from(options.max_hops),
            type_include: &[],
            type_exclude: Vec::new(),
            typed_only: false,
            inline_only: false,
            max_nodes: None,
            max_edges: None,
            max_fanout: None,
            max_chars: None,
            semantic_inversion: true,
            min_value: None,
            ignore_value: false,
        };

        // Build compaction context if needed
        let compaction_ctx = CompactionContext::build(all_notes)?;

        // For each initial note, perform simple traversal and collect discovered notes
        for initial_id in &initial_ids {
            perform_simple_traversal(
                index,
                initial_id,
                &traversal_options,
                Some(&compaction_ctx),
                store,
                &mut selected_notes,
                &mut seen_ids,
            )?;
        }
    }

    Ok(selected_notes)
}

pub fn resolve_compaction_notes(
    store: &Store,
    compaction_ctx: &CompactionContext,
    notes: Vec<Note>,
) -> Result<Vec<Note>> {
    let mut resolved = Vec::new();
    let mut seen_ids = HashSet::new();

    for note in notes {
        let canonical_id = compaction_ctx.canon(note.id())?;
        if seen_ids.insert(canonical_id.clone()) {
            if canonical_id == note.id() {
                resolved.push(note);
            } else {
                resolved.push(store.get_note(&canonical_id)?);
            }
        }
    }

    Ok(resolved)
}

pub fn sort_notes_by_created_id(notes: &mut [Note]) {
    notes.sort_by(|a, b| {
        match (&a.frontmatter.created, &b.frontmatter.created) {
            (Some(a_created), Some(b_created)) => a_created.cmp(b_created),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
        .then_with(|| a.id().cmp(b.id()))
    });
}

/// Get notes linked from a MOC (direct links only, not transitive)
/// Preserves the order in which links appear in the MOC
pub fn get_moc_linked_notes(store: &Store, index: &Index, moc_id: &str) -> Result<Vec<Note>> {
    use regex::Regex;

    let moc = store.get_note(moc_id)?;

    // Extract links in the same order as during index building:
    // 1. Typed links from frontmatter
    // 2. Wiki links from body (in order of appearance)
    // 3. Markdown links from body (in order of appearance)
    let mut linked_notes = Vec::new();
    let mut seen_ids = HashSet::new();

    // 1. Extract typed links from frontmatter first
    for typed_link in &moc.frontmatter.links {
        let to_id = &typed_link.id;
        if !seen_ids.insert(to_id.clone()) {
            continue;
        }

        if index.contains(to_id) {
            if let Ok(note) = store.get_note(to_id) {
                linked_notes.push(note);
            }
        }
    }

    // 2. Extract wiki links: [[id]] or [[id|label]]
    let wiki_link_re = Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]")
        .map_err(|e| QipuError::Other(format!("failed to compile wiki link regex: {}", e)))?;

    for cap in wiki_link_re.captures_iter(&moc.body) {
        let to_id = cap[1].trim().to_string();
        if to_id.is_empty() || !seen_ids.insert(to_id.clone()) {
            continue;
        }

        // Verify the note exists in the index
        if index.contains(&to_id) {
            if let Ok(note) = store.get_note(&to_id) {
                linked_notes.push(note);
            }
        }
    }

    // 3. Extract markdown links: [text](qp-xxxx) or [text](./qp-xxxx-slug.md)
    let md_link_re = Regex::new(r"\[([^\]]*)\]\(([^)]+)\)")
        .map_err(|e| QipuError::Other(format!("failed to compile markdown link regex: {}", e)))?;

    for cap in md_link_re.captures_iter(&moc.body) {
        let target = cap[2].trim();

        // Skip external URLs and anchors
        if target.starts_with("http://")
            || target.starts_with("https://")
            || target.starts_with('#')
        {
            continue;
        }

        // Try to resolve the link to a note ID
        let to_id = if target.starts_with("qp-") {
            // Direct ID reference: [text](qp-xxxx)
            Some(target.split('-').take(2).collect::<Vec<_>>().join("-"))
        } else if target.contains("qp-") {
            // Path reference containing ID: [text](./qp-xxxx-slug.md)
            if let Some(start) = target.find("qp-") {
                let rest = &target[start..];
                let end = rest
                    .find('-')
                    .and_then(|first| rest[first + 1..].find('-').map(|second| first + 1 + second));
                match end {
                    Some(end) => Some(rest[..end].to_string()),
                    None => Some(rest.trim_end_matches(".md").to_string()),
                }
            } else {
                None
            }
        } else {
            None
        };

        if let Some(id) = to_id {
            if id.is_empty() || !id.starts_with("qp-") || !seen_ids.insert(id.clone()) {
                continue;
            }

            // Verify the note exists in the index
            if index.contains(&id) {
                if let Ok(note) = store.get_note(&id) {
                    linked_notes.push(note);
                }
            }
        }
    }

    Ok(linked_notes)
}

/// Perform simple graph traversal for export command
fn perform_simple_traversal<'a>(
    index: &Index,
    root: &str,
    opts: &TreeOptions<'a>,
    _compaction_ctx: Option<&CompactionContext>,
    store: &Store,
    selected_notes: &mut Vec<Note>,
    seen_ids: &mut HashSet<String>,
) -> Result<()> {
    use std::collections::VecDeque;

    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(String, HopCost)> = VecDeque::new();

    queue.push_back((root.to_string(), HopCost::from(0)));
    visited.insert(root.to_string());

    while let Some((current_id, accumulated_cost)) = queue.pop_front() {
        if accumulated_cost.value() >= opts.max_hops.value() {
            continue;
        }

        // Get outbound edges from current note
        for edge in &index.edges {
            let should_follow = match opts.direction {
                Direction::Out => edge.from == current_id,
                Direction::In => edge.to == current_id,
                Direction::Both => edge.from == current_id || edge.to == current_id,
            };

            if should_follow {
                // Determine the neighbor ID
                let neighbor_id = if edge.from == current_id {
                    &edge.to
                } else {
                    &edge.from
                };

                if visited.insert(neighbor_id.clone()) {
                    let next_cost = accumulated_cost + HopCost::from(1);
                    queue.push_back((neighbor_id.clone(), next_cost));

                    // Add to selected notes if not already present
                    if seen_ids.insert(neighbor_id.clone()) {
                        if let Ok(note) = store.get_note(neighbor_id) {
                            selected_notes.push(note);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
