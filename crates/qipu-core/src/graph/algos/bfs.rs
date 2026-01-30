use crate::compaction::CompactionContext;
use crate::error::Result;
use crate::graph::algos::shared::{
    build_filtered_result, build_result, calculate_edge_cost, canonicalize_edge, canonicalize_node,
    collect_inbound_neighbors, collect_outbound_neighbors, get_source_ids,
    has_unexpanded_neighbors, neighbor_passes_filter, prepare_neighbors, root_passes_filter,
    set_truncation, set_truncation_if_unset, sort_results, NeighborContext,
};
use crate::graph::types::{
    HopCost, SpanningTreeEntry, TreeLink, TreeNote, TreeOptions, TreeResult,
};
use crate::graph::GraphProvider;
use crate::store::Store;
use std::collections::{HashMap, HashSet, VecDeque};

/// State tracked during BFS traversal
struct BfsState {
    visited: HashSet<String>,
    queue: VecDeque<(String, HopCost)>,
    notes: Vec<TreeNote>,
    links: Vec<TreeLink>,
    spanning_tree: Vec<SpanningTreeEntry>,
    truncated: bool,
    truncation_reason: Option<String>,
}

impl BfsState {
    fn new() -> Self {
        Self {
            visited: HashSet::new(),
            queue: VecDeque::new(),
            notes: Vec::new(),
            links: Vec::new(),
            spanning_tree: Vec::new(),
            truncated: false,
            truncation_reason: None,
        }
    }

    fn check_limits(&mut self, opts: &TreeOptions) -> bool {
        super::shared::check_limits(
            self.visited.len(),
            self.links.len(),
            &mut self.truncated,
            &mut self.truncation_reason,
            opts,
        )
    }
}

/// Process a single neighbor, updating state
fn process_neighbor(
    neighbor_id: String,
    edge: crate::index::Edge,
    state: &mut BfsState,
    ctx: &NeighborContext<'_>,
) -> Result<()> {
    // Canonicalize edge endpoints
    let (canonical_from, canonical_to) = canonicalize_edge(&edge, ctx.compaction_ctx)?;

    // Skip self-loops
    if canonical_from == canonical_to {
        return Ok(());
    }

    // Canonicalize neighbor ID
    let canonical_neighbor = canonicalize_node(&neighbor_id, ctx.compaction_ctx)?;

    // Check filter
    if !neighbor_passes_filter(ctx.provider, &canonical_neighbor, &state.visited, ctx.opts) {
        return Ok(());
    }

    // Check max_edges again
    if let Some(max) = ctx.opts.max_edges {
        if state.links.len() >= max {
            set_truncation(
                &mut state.truncated,
                &mut state.truncation_reason,
                "max_edges",
            );
            return Ok(());
        }
    }

    // Track via if canonicalized
    let via_for_link = if neighbor_id != canonical_neighbor {
        Some(neighbor_id)
    } else {
        None
    };

    // Add edge
    let link_type_str = edge.link_type.to_string();
    let source_str = edge.source.to_string();
    state.links.push(TreeLink {
        from: canonical_from,
        to: canonical_to,
        link_type: link_type_str,
        source: source_str,
        via: via_for_link.clone(),
    });

    // Process new neighbor
    if !state.visited.contains(&canonical_neighbor) {
        // Check max_nodes
        if let Some(max) = ctx.opts.max_nodes {
            if state.visited.len() >= max {
                set_truncation(
                    &mut state.truncated,
                    &mut state.truncation_reason,
                    "max_nodes",
                );
                return Ok(());
            }
        }

        state.visited.insert(canonical_neighbor.clone());

        // Calculate cost
        let edge_cost = calculate_edge_cost(
            &edge,
            &canonical_neighbor,
            ctx.provider,
            ctx.store,
            ctx.opts,
        );
        let new_cost = ctx.accumulated_cost + edge_cost;

        // Add to spanning tree
        state.spanning_tree.push(SpanningTreeEntry {
            from: ctx.current_id.to_string(),
            to: canonical_neighbor.clone(),
            hop: new_cost.as_u32_for_display(),
            link_type: edge.link_type.to_string(),
        });

        // Add note metadata
        if let Some(meta) = ctx.provider.get_metadata(&canonical_neighbor) {
            state.notes.push(TreeNote {
                id: meta.id.clone(),
                title: meta.title.clone(),
                note_type: meta.note_type.clone(),
                tags: meta.tags.clone(),
                path: meta.path.clone(),
                via: via_for_link,
            });
        }

        // Queue for expansion
        state.queue.push_back((canonical_neighbor, new_cost));
    }

    Ok(())
}

/// Perform BFS traversal from a root node
#[tracing::instrument(skip(provider, store, opts, compaction_ctx, equivalence_map), fields(root = %root, direction = ?opts.direction, max_hops = %opts.max_hops.value(), max_nodes, max_edges, max_fanout))]
pub fn bfs_traverse(
    provider: &dyn GraphProvider,
    store: &Store,
    root: &str,
    opts: &TreeOptions,
    compaction_ctx: Option<&CompactionContext>,
    equivalence_map: Option<&HashMap<String, Vec<String>>>,
) -> Result<TreeResult> {
    // Check root filter
    if !root_passes_filter(provider, root, opts) {
        return Ok(build_filtered_result(root, opts));
    }

    // Initialize state
    let mut state = BfsState::new();
    let root_owned = root.to_string();
    state
        .queue
        .push_back((root_owned.clone(), HopCost::from(0)));
    state.visited.insert(root_owned);

    // Add root note
    if let Some(meta) = provider.get_metadata(root) {
        state.notes.push(TreeNote {
            id: meta.id.clone(),
            title: meta.title.clone(),
            note_type: meta.note_type.clone(),
            tags: meta.tags.clone(),
            path: meta.path.clone(),
            via: None,
        });
    }

    // Main BFS loop
    while let Some((current_id, accumulated_cost)) = state.queue.pop_front() {
        if !state.check_limits(opts) {
            break;
        }

        // Handle max_hops reached
        if accumulated_cost.value() >= opts.max_hops.value() {
            let source_ids = get_source_ids(&current_id, equivalence_map);
            if has_unexpanded_neighbors(provider, &source_ids, opts) {
                set_truncation_if_unset(
                    &mut state.truncated,
                    &mut state.truncation_reason,
                    "max_hops",
                );
            }
            continue;
        }

        // Collect neighbors
        let source_ids = get_source_ids(&current_id, equivalence_map);
        let mut neighbors = collect_outbound_neighbors(provider, &source_ids, opts);
        neighbors.extend(collect_inbound_neighbors(
            provider,
            store,
            &source_ids,
            opts,
        ));

        // Prepare and process neighbors
        let neighbors = prepare_neighbors(
            neighbors,
            opts,
            &mut state.truncated,
            &mut state.truncation_reason,
        );

        let neighbor_ctx = NeighborContext {
            current_id: &current_id,
            accumulated_cost,
            provider,
            store,
            opts,
            compaction_ctx,
        };

        for (neighbor_id, edge) in neighbors {
            process_neighbor(neighbor_id, edge, &mut state, &neighbor_ctx)?;
        }
    }

    sort_results(&mut state.notes, &mut state.links, &mut state.spanning_tree);
    Ok(build_result(
        root,
        opts,
        state.truncated,
        state.truncation_reason,
        state.notes,
        state.links,
        state.spanning_tree,
    ))
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::graph::types::{HopCost, TreeOptions};
    use crate::index::IndexBuilder;
    use crate::store::Store;
    use tempfile::tempdir;

    /// Test that bfs_traverse works correctly
    #[test]
    fn test_bfs_traverse_basic() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), crate::store::InitOptions::default()).unwrap();

        let root = store
            .create_note("Root Note", None, &["root".to_string()], None)
            .unwrap();
        let mid = store
            .create_note("Mid Note", None, &["mid".to_string()], None)
            .unwrap();
        let leaf = store
            .create_note("Leaf Note", None, &["leaf".to_string()], None)
            .unwrap();

        // Create links: root -> mid -> leaf
        let mut root_note = store.get_note(root.id()).unwrap();
        root_note.frontmatter.links.push(crate::note::TypedLink {
            link_type: crate::note::LinkType::from("supports"),
            id: mid.id().to_string(),
        });
        store.save_note(&mut root_note).unwrap();

        let mut mid_note = store.get_note(mid.id()).unwrap();
        mid_note.frontmatter.links.push(crate::note::TypedLink {
            link_type: crate::note::LinkType::from("supports"),
            id: leaf.id().to_string(),
        });
        store.save_note(&mut mid_note).unwrap();

        let index = IndexBuilder::new(&store).build().unwrap();

        let opts = TreeOptions {
            ignore_value: true,
            max_hops: HopCost::from(5),
            ..Default::default()
        };

        let result = bfs_traverse(&index, &store, root.id(), &opts, None, None).unwrap();

        assert_eq!(result.root, root.id());
        assert!(!result.truncated);
        assert_eq!(result.notes.len(), 3); // root + mid + leaf
    }
}
