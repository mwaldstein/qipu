use crate::compaction::CompactionContext;
use crate::error::Result;
use crate::graph::algos::shared::{
    build_filtered_result, build_result, calculate_edge_cost, canonicalize_edge, canonicalize_node,
    collect_inbound_neighbors, collect_outbound_neighbors, get_source_ids,
    has_unexpanded_neighbors, neighbor_passes_filter, prepare_neighbors, root_passes_filter,
    set_truncation, set_truncation_if_unset, sort_results, NeighborContext,
};
use crate::graph::types::{
    Direction, HopCost, SpanningTreeEntry, TreeLink, TreeNote, TreeOptions, TreeResult,
};
use crate::graph::GraphProvider;
use crate::store::Store;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// Wrapper for BinaryHeap to use as min-heap (ordered by accumulated cost)
#[derive(Debug, Clone)]
pub struct HeapEntry {
    pub node_id: String,
    pub accumulated_cost: HopCost,
}

impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.node_id == other.node_id
            && self.accumulated_cost.value() == other.accumulated_cost.value()
    }
}

impl Eq for HeapEntry {}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.accumulated_cost
            .value()
            .partial_cmp(&other.accumulated_cost.value())
            .unwrap()
    }
}

/// State tracked during Dijkstra traversal
struct DijkstraState {
    visited: HashSet<String>,
    heap: BinaryHeap<Reverse<HeapEntry>>,
    notes: Vec<TreeNote>,
    links: Vec<TreeLink>,
    spanning_tree: Vec<SpanningTreeEntry>,
    truncated: bool,
    truncation_reason: Option<String>,
}

impl DijkstraState {
    fn new() -> Self {
        Self {
            visited: HashSet::new(),
            heap: BinaryHeap::new(),
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
fn process_neighbor_dijkstra(
    neighbor_id: String,
    edge: crate::index::Edge,
    state: &mut DijkstraState,
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
        link_type: link_type_str.clone(),
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
            link_type: link_type_str,
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

        // Add to heap for expansion
        state.heap.push(Reverse(HeapEntry {
            node_id: canonical_neighbor,
            accumulated_cost: new_cost,
        }));
    }

    Ok(())
}

/// Perform Dijkstra traversal from a root node (weighted by note value)
/// Default behavior: weighted traversal using `get_edge_cost()` which
/// applies penalties for low-value notes (value 0-100).
/// With `--ignore-value`: unweighted BFS (all edges cost 1.0).
#[tracing::instrument(skip(provider, store, opts, compaction_ctx, equivalence_map), fields(root = %root, direction = ?opts.direction, max_hops = %opts.max_hops.value(), max_nodes, max_edges, max_fanout, ignore_value = %opts.ignore_value))]
pub fn dijkstra_traverse(
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
    let mut state = DijkstraState::new();
    let root_owned = root.to_string();
    state.heap.push(Reverse(HeapEntry {
        node_id: root_owned.clone(),
        accumulated_cost: HopCost::from(0),
    }));
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

    // Main Dijkstra loop
    while let Some(Reverse(HeapEntry {
        node_id: current_id,
        accumulated_cost,
    })) = state.heap.pop()
    {
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

        // Collect neighbors based on direction
        let source_ids = get_source_ids(&current_id, equivalence_map);
        let mut neighbors = Vec::new();

        if opts.direction == Direction::Out || opts.direction == Direction::Both {
            neighbors.extend(collect_outbound_neighbors(provider, &source_ids, opts));
        }

        if opts.direction == Direction::In || opts.direction == Direction::Both {
            neighbors.extend(collect_inbound_neighbors(
                provider,
                store,
                &source_ids,
                opts,
            ));
        }

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
            process_neighbor_dijkstra(neighbor_id, edge, &mut state, &neighbor_ctx)?;
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
mod tests;
