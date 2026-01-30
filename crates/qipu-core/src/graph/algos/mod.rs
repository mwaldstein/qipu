//! Graph algorithm implementations
//!
//! Contains concrete implementations of graph algorithms:
//! - `bfs`: Breadth-first search for tree traversal
//! - `dijkstra`: Weighted shortest path finding
//! - `shared`: Common utilities used by multiple algorithms

pub mod bfs;
pub mod dijkstra;
pub mod shared;

pub use bfs::bfs_traverse;
pub use dijkstra::dijkstra_traverse;
pub use shared::{
    build_filtered_result, build_result, calculate_edge_cost, canonicalize_edge, canonicalize_node,
    check_limits, collect_inbound_neighbors, collect_outbound_neighbors, get_source_ids,
    has_unexpanded_neighbors, neighbor_passes_filter, prepare_neighbors, root_passes_filter,
    sort_results, NeighborContext,
};
