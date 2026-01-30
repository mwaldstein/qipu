use crate::compaction::CompactionContext;
use crate::error::Result;
use crate::graph::types::{
    filter_edge, get_edge_cost, get_link_type_cost, Direction, HopCost, SpanningTreeEntry,
    TreeLink, TreeNote, TreeOptions, TreeResult, DIRECTION_BOTH, DIRECTION_IN, DIRECTION_OUT,
};
use crate::graph::GraphProvider;
use crate::index::Edge;
use crate::store::Store;
use std::collections::{HashMap, HashSet};

/// Trait for traversal state that supports limit checking
trait TraversalState {
    fn visited_count(&self) -> usize;
    fn links_count(&self) -> usize;
}

/// Check limits and return false if traversal should stop
pub fn check_limits(
    visited_len: usize,
    links_len: usize,
    truncated: &mut bool,
    truncation_reason: &mut Option<String>,
    opts: &TreeOptions,
) -> bool {
    if let Some(max) = opts.max_nodes {
        if visited_len >= max {
            *truncated = true;
            *truncation_reason = Some("max_nodes".to_string());
            return false;
        }
    }

    if let Some(max) = opts.max_edges {
        if links_len >= max {
            *truncated = true;
            *truncation_reason = Some("max_edges".to_string());
            return false;
        }
    }

    true
}

/// Check if root passes min_value filter
pub fn root_passes_filter(provider: &dyn GraphProvider, root: &str, opts: &TreeOptions) -> bool {
    if let Some(meta) = provider.get_metadata(root) {
        let value = meta.value.unwrap_or(50);
        opts.min_value.is_none_or(|min| value >= min)
    } else {
        false
    }
}

/// Build empty result when root is filtered out
pub fn build_filtered_result(root: &str, opts: &TreeOptions) -> TreeResult {
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
pub fn get_source_ids<'a>(
    current_id: &'a str,
    equivalence_map: Option<&'a HashMap<String, Vec<String>>>,
) -> Vec<&'a str> {
    match equivalence_map.and_then(|map| map.get(current_id)) {
        Some(ids) if !ids.is_empty() => ids.iter().map(|s| s.as_str()).collect(),
        _ => vec![current_id],
    }
}

/// Check if there are unexpanded neighbors at max_hops
pub fn has_unexpanded_neighbors(
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
pub fn collect_outbound_neighbors(
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
pub fn collect_inbound_neighbors(
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
pub fn prepare_neighbors(
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
pub fn neighbor_passes_filter(
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
pub fn calculate_edge_cost(
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
pub struct NeighborContext<'a> {
    pub current_id: &'a str,
    pub accumulated_cost: HopCost,
    pub provider: &'a dyn GraphProvider,
    pub store: &'a Store,
    pub opts: &'a TreeOptions,
    pub compaction_ctx: Option<&'a CompactionContext>,
}

/// Canonicalize edge endpoints
pub fn canonicalize_edge(
    edge: &Edge,
    compaction_ctx: Option<&CompactionContext>,
) -> Result<(String, String)> {
    if let Some(ctx) = compaction_ctx {
        Ok((ctx.canon(&edge.from)?, ctx.canon(&edge.to)?))
    } else {
        Ok((edge.from.clone(), edge.to.clone()))
    }
}

/// Canonicalize a node ID
pub fn canonicalize_node(
    node_id: &str,
    compaction_ctx: Option<&CompactionContext>,
) -> Result<String> {
    if let Some(ctx) = compaction_ctx {
        Ok(ctx.canon(node_id)?)
    } else {
        Ok(node_id.to_string())
    }
}

/// Sort all result collections for determinism
pub fn sort_results(
    notes: &mut [TreeNote],
    links: &mut [TreeLink],
    spanning_tree: &mut [SpanningTreeEntry],
) {
    notes.sort_by(|a, b| a.id.cmp(&b.id));
    links.sort_by(|a, b| {
        a.from
            .cmp(&b.from)
            .then_with(|| a.link_type.cmp(&b.link_type))
            .then_with(|| a.to.cmp(&b.to))
    });
    spanning_tree.sort_by(|a, b| {
        a.hop
            .cmp(&b.hop)
            .then_with(|| a.link_type.cmp(&b.link_type))
            .then_with(|| a.to.cmp(&b.to))
    });
}

/// Build final TreeResult
pub fn build_result(
    root: &str,
    opts: &TreeOptions,
    truncated: bool,
    truncation_reason: Option<String>,
    notes: Vec<TreeNote>,
    links: Vec<TreeLink>,
    spanning_tree: Vec<SpanningTreeEntry>,
) -> TreeResult {
    TreeResult {
        root: root.to_string(),
        direction: match opts.direction {
            Direction::Out => DIRECTION_OUT.to_string(),
            Direction::In => DIRECTION_IN.to_string(),
            Direction::Both => DIRECTION_BOTH.to_string(),
        },
        max_hops: opts.max_hops.as_u32_for_display(),
        truncated,
        truncation_reason,
        notes,
        links,
        spanning_tree,
    }
}
