//! Compaction candidate suggestion and clustering

use std::collections::{HashMap, HashSet};

use crate::error::Result;

use super::context::CompactionContext;
use super::estimate_note_size;

/// A compaction candidate cluster
#[derive(Debug, Clone)]
pub struct CompactionCandidate {
    /// Note IDs in this cluster
    pub ids: Vec<String>,
    /// Number of notes in the cluster
    pub node_count: usize,
    /// Number of edges within the cluster
    pub internal_edges: usize,
    /// Number of edges connecting to notes outside the cluster
    pub boundary_edges: usize,
    /// Ratio of boundary edges to total edges (0.0 = fully isolated, 1.0 = fully connected externally)
    pub boundary_ratio: f64,
    /// Cluster cohesion score (1.0 = all internal edges, 0.0 = all boundary edges)
    pub cohesion: f64,
    /// Estimated total size in bytes of all notes in the cluster
    pub estimated_size: usize,
    /// Overall candidate score (higher = better compaction candidate)
    pub score: f64,
}

impl CompactionContext {
    /// Find compaction candidates using graph clustering
    pub fn suggest(
        &self,
        store: &crate::store::Store,
        index: &crate::index::Index,
    ) -> Result<Vec<CompactionCandidate>> {
        // Build adjacency list for clustering
        let mut adjacency: HashMap<String, HashSet<String>> = HashMap::new();

        // Add all notes as nodes
        for note_id in index.metadata.keys() {
            adjacency.entry(note_id.clone()).or_default();
        }

        // Add edges (make undirected for clustering)
        for edge in &index.edges {
            adjacency
                .entry(edge.from.clone())
                .or_default()
                .insert(edge.to.clone());
            adjacency
                .entry(edge.to.clone())
                .or_default()
                .insert(edge.from.clone());
        }

        // Find connected components using DFS
        let mut visited = HashSet::new();
        let mut components = Vec::new();

        for node_id in adjacency.keys() {
            if !visited.contains(node_id) {
                let component = self.find_component(&adjacency, node_id, &mut visited);
                if component.len() >= 3 {
                    // Only consider components with at least 3 nodes
                    components.push(component);
                }
            }
        }

        // Calculate metrics for each component
        let mut candidates = Vec::new();
        for component in components {
            if let Ok(candidate) = self.calculate_candidate_metrics(store, index, &component) {
                // Only include candidates with reasonable cohesion
                if candidate.cohesion >= 0.3 && candidate.node_count >= 3 {
                    candidates.push(candidate);
                }
            }
        }

        // Sort by score (descending)
        candidates.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Return top candidates (max 10)
        candidates.truncate(10);

        Ok(candidates)
    }

    /// Find a connected component starting from a node (DFS)
    fn find_component(
        &self,
        adjacency: &HashMap<String, HashSet<String>>,
        start: &str,
        visited: &mut HashSet<String>,
    ) -> Vec<String> {
        let mut component = Vec::new();
        let mut stack = vec![start.to_string()];

        while let Some(node) = stack.pop() {
            if visited.contains(&node) {
                continue;
            }

            visited.insert(node.clone());
            component.push(node.clone());

            if let Some(neighbors) = adjacency.get(&node) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        stack.push(neighbor.clone());
                    }
                }
            }
        }

        // Sort for deterministic ordering
        component.sort();
        component
    }

    /// Calculate metrics for a compaction candidate
    fn calculate_candidate_metrics(
        &self,
        store: &crate::store::Store,
        index: &crate::index::Index,
        cluster: &[String],
    ) -> Result<CompactionCandidate> {
        let cluster_set: HashSet<_> = cluster.iter().cloned().collect();

        // Count internal and boundary edges
        let mut internal_edges = 0;
        let mut boundary_edges = 0;

        for node_id in cluster {
            let outbound = index.get_outbound_edges(node_id);
            for edge in outbound {
                if cluster_set.contains(&edge.to) {
                    internal_edges += 1;
                } else {
                    boundary_edges += 1;
                }
            }
        }

        // Calculate metrics
        let total_edges = internal_edges + boundary_edges;
        let boundary_ratio = if total_edges > 0 {
            boundary_edges as f64 / total_edges as f64
        } else {
            0.0
        };

        let cohesion = if total_edges > 0 {
            internal_edges as f64 / total_edges as f64
        } else {
            0.0
        };

        // Estimate total size
        let mut estimated_size = 0;
        let mut total_value = 0u32;
        let mut value_count = 0usize;

        for node_id in cluster {
            if let Ok(note) = store.get_note(node_id) {
                estimated_size += estimate_note_size(&note);
                if let Some(v) = note.frontmatter.value {
                    total_value += v as u32;
                    value_count += 1;
                }
            }
        }

        // Calculate average value (default to 50 if no values set)
        let avg_value = if value_count > 0 {
            total_value as f64 / value_count as f64
        } else {
            50.0
        };

        // Value boost: low-value notes are better compaction candidates
        // Per spec: "Notes with `value < 20` are strong candidates for compaction"
        let value_boost = if avg_value < 20.0 {
            15.0
        } else if avg_value < 40.0 {
            7.5
        } else if avg_value < 60.0 {
            0.0
        } else if avg_value < 80.0 {
            -5.0
        } else {
            -10.0
        };

        // Calculate score
        let node_count = cluster.len();
        let size_score = (estimated_size as f64).ln().max(0.0);
        let cohesion_score = cohesion * 10.0;
        let boundary_penalty = boundary_ratio * -5.0;
        let node_score = (node_count as f64).sqrt();

        let score = size_score + cohesion_score + boundary_penalty + node_score + value_boost;

        Ok(CompactionCandidate {
            ids: cluster.to_vec(),
            node_count,
            internal_edges,
            boundary_edges,
            boundary_ratio,
            cohesion,
            estimated_size,
            score,
        })
    }
}
