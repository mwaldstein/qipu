use crate::compaction::CompactionContext;
use crate::error::Result;
use crate::graph::types::{
    filter_edge, get_edge_cost, get_link_type_cost, Direction, HopCost, SpanningTreeEntry,
    TreeLink, TreeNote, TreeOptions, TreeResult, DIRECTION_BOTH, DIRECTION_IN, DIRECTION_OUT,
};
use crate::graph::GraphProvider;
use crate::index::Edge;
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
        if let Some(max) = opts.max_nodes {
            if self.visited.len() >= max {
                self.truncated = true;
                self.truncation_reason = Some("max_nodes".to_string());
                return false;
            }
        }

        if let Some(max) = opts.max_edges {
            if self.links.len() >= max {
                self.truncated = true;
                self.truncation_reason = Some("max_edges".to_string());
                return false;
            }
        }

        true
    }
}

/// Check if root passes min_value filter
fn root_passes_filter(provider: &dyn GraphProvider, root: &str, opts: &TreeOptions) -> bool {
    if let Some(meta) = provider.get_metadata(root) {
        let value = meta.value.unwrap_or(50);
        opts.min_value.is_none_or(|min| value >= min)
    } else {
        false
    }
}

/// Build empty result when root is filtered out
fn build_filtered_result(root: &str, opts: &TreeOptions) -> TreeResult {
    TreeResult {
        root: root.to_string(),
        direction: match opts.direction {
            Direction::Out => DIRECTION_OUT.to_string(),
            Direction::In => DIRECTION_IN.to_string(),
            Direction::Both => DIRECTION_BOTH.to_string(),
        },
        max_hops: opts.max_hops.as_u32_for_display(),
        truncated: false,
        truncation_reason: Some("min_value filter excluded root".to_string()),
        notes: vec![],
        links: vec![],
        spanning_tree: vec![],
    }
}

/// Get source IDs considering equivalence map
fn get_source_ids<'a>(
    current_id: &'a str,
    equivalence_map: Option<&'a HashMap<String, Vec<String>>>,
) -> Vec<&'a str> {
    match equivalence_map.and_then(|map| map.get(current_id)) {
        Some(ids) if !ids.is_empty() => ids.iter().map(|s| s.as_str()).collect(),
        _ => vec![current_id],
    }
}

/// Check if there are unexpanded neighbors at max_hops
fn has_unexpanded_neighbors(
    provider: &dyn GraphProvider,
    source_ids: &[&str],
    opts: &TreeOptions,
) -> bool {
    let has_outbound = if opts.direction == Direction::Out || opts.direction == Direction::Both {
        source_ids.iter().any(|id| {
            provider
                .get_outbound_edges(id)
                .iter()
                .any(|e| filter_edge(e, opts))
        })
    } else {
        false
    };

    let has_inbound = if opts.direction == Direction::In || opts.direction == Direction::Both {
        source_ids.iter().any(|id| {
            provider
                .get_inbound_edges(id)
                .iter()
                .any(|e| filter_edge(e, opts))
        })
    } else {
        false
    };

    has_outbound || has_inbound
}

/// Collect all neighbors from outbound edges
fn collect_outbound_neighbors(
    provider: &dyn GraphProvider,
    source_ids: &[&str],
    opts: &TreeOptions,
) -> Vec<(String, Edge)> {
    let mut neighbors = Vec::new();
    for source_id in source_ids {
        for edge in provider.get_outbound_edges(source_id) {
            if filter_edge(&edge, opts) {
                let to = edge.to.clone();
                neighbors.push((to, edge));
            }
        }
    }
    neighbors
}

/// Collect all neighbors from inbound edges (with optional semantic inversion)
fn collect_inbound_neighbors(
    provider: &dyn GraphProvider,
    store: &Store,
    source_ids: &[&str],
    opts: &TreeOptions,
) -> Vec<(String, Edge)> {
    let mut neighbors = Vec::new();
    for source_id in source_ids {
        for edge in provider.get_inbound_edges(source_id) {
            if opts.semantic_inversion {
                let virtual_edge = edge.invert(store.config());
                if filter_edge(&virtual_edge, opts) {
                    let to = virtual_edge.to.clone();
                    neighbors.push((to, virtual_edge));
                }
            } else if filter_edge(&edge, opts) {
                let from = edge.from.clone();
                neighbors.push((from, edge));
            }
        }
    }
    neighbors
}

/// Sort and apply max_fanout to neighbors
fn prepare_neighbors(
    mut neighbors: Vec<(String, Edge)>,
    opts: &TreeOptions,
    truncated: &mut bool,
    truncation_reason: &mut Option<String>,
) -> Vec<(String, Edge)> {
    neighbors.sort_by(|a, b| {
        a.1.link_type
            .cmp(&b.1.link_type)
            .then_with(|| a.0.cmp(&b.0))
    });

    if let Some(max_fanout) = opts.max_fanout {
        if neighbors.len() > max_fanout {
            *truncated = true;
            *truncation_reason = Some("max_fanout".to_string());
        }
        neighbors.into_iter().take(max_fanout).collect()
    } else {
        neighbors
    }
}

/// Check if neighbor passes min_value filter
fn neighbor_passes_filter(
    provider: &dyn GraphProvider,
    canonical_neighbor: &str,
    visited: &HashSet<String>,
    opts: &TreeOptions,
) -> bool {
    if visited.contains(canonical_neighbor) {
        return true;
    }
    if let Some(meta) = provider.get_metadata(canonical_neighbor) {
        let value = meta.value.unwrap_or(50);
        opts.min_value.is_none_or(|min| value >= min)
    } else {
        false
    }
}

/// Calculate edge cost based on options
fn calculate_edge_cost(
    edge: &Edge,
    canonical_neighbor: &str,
    provider: &dyn GraphProvider,
    store: &Store,
    opts: &TreeOptions,
) -> HopCost {
    if opts.ignore_value {
        get_link_type_cost(edge.link_type.as_str(), store.config())
    } else if let Some(meta) = provider.get_metadata(canonical_neighbor) {
        let value = meta.value.unwrap_or(50);
        get_edge_cost(edge.link_type.as_str(), value, store.config())
    } else {
        get_link_type_cost(edge.link_type.as_str(), store.config())
    }
}

/// Context for processing a neighbor
struct NeighborContext<'a> {
    current_id: &'a str,
    accumulated_cost: HopCost,
    provider: &'a dyn GraphProvider,
    store: &'a Store,
    opts: &'a TreeOptions,
    compaction_ctx: Option<&'a CompactionContext>,
}

/// Process a single neighbor, updating state
fn process_neighbor_dijkstra(
    neighbor_id: String,
    edge: Edge,
    state: &mut DijkstraState,
    ctx: &NeighborContext<'_>,
) -> Result<()> {
    // Canonicalize edge endpoints
    let (canonical_from, canonical_to) = if let Some(compact_ctx) = ctx.compaction_ctx {
        (compact_ctx.canon(&edge.from)?, compact_ctx.canon(&edge.to)?)
    } else {
        (edge.from.clone(), edge.to.clone())
    };

    // Skip self-loops
    if canonical_from == canonical_to {
        return Ok(());
    }

    // Canonicalize neighbor ID
    let canonical_neighbor = if let Some(compact_ctx) = ctx.compaction_ctx {
        compact_ctx.canon(&neighbor_id)?
    } else {
        neighbor_id.clone()
    };

    // Check filter
    if !neighbor_passes_filter(ctx.provider, &canonical_neighbor, &state.visited, ctx.opts) {
        return Ok(());
    }

    // Check max_edges again
    if let Some(max) = ctx.opts.max_edges {
        if state.links.len() >= max {
            state.truncated = true;
            state.truncation_reason = Some("max_edges".to_string());
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
                state.truncated = true;
                state.truncation_reason = Some("max_nodes".to_string());
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
                note_type: meta.note_type,
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

/// Sort all result collections for determinism
fn sort_results(state: &mut DijkstraState) {
    state.notes.sort_by(|a, b| a.id.cmp(&b.id));
    state.links.sort_by(|a, b| {
        a.from
            .cmp(&b.from)
            .then_with(|| a.link_type.cmp(&b.link_type))
            .then_with(|| a.to.cmp(&b.to))
    });
    state.spanning_tree.sort_by(|a, b| {
        a.hop
            .cmp(&b.hop)
            .then_with(|| a.link_type.cmp(&b.link_type))
            .then_with(|| a.to.cmp(&b.to))
    });
}

/// Build final TreeResult from state
fn build_result(root: &str, opts: &TreeOptions, state: DijkstraState) -> TreeResult {
    TreeResult {
        root: root.to_string(),
        direction: match opts.direction {
            Direction::Out => DIRECTION_OUT.to_string(),
            Direction::In => DIRECTION_IN.to_string(),
            Direction::Both => DIRECTION_BOTH.to_string(),
        },
        max_hops: opts.max_hops.as_u32_for_display(),
        truncated: state.truncated,
        truncation_reason: state.truncation_reason,
        notes: state.notes,
        links: state.links,
        spanning_tree: state.spanning_tree,
    }
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
            note_type: meta.note_type,
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
                state.truncated = true;
                if state.truncation_reason.is_none() {
                    state.truncation_reason = Some("max_hops".to_string());
                }
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

    sort_results(&mut state);
    Ok(build_result(root, opts, state))
}

#[cfg(test)]
mod tests;
