use std::path::PathBuf;

use crate::cli::Cli;
use crate::lib::error::Result;
use crate::lib::index::Index;
use crate::lib::store::Store;

use super::utils::estimate_size;

/// A compaction candidate cluster
#[derive(Debug, Clone)]
struct CompactionCandidate {
    ids: Vec<String>,
    node_count: usize,
    internal_edges: usize,
    boundary_edges: usize,
    boundary_ratio: f64,
    cohesion: f64,
    estimated_size: usize,
    score: f64,
}

/// Execute `qipu compact suggest`
pub fn execute(cli: &Cli) -> Result<()> {
    let root = cli
        .root
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let store = if let Some(path) = &cli.store {
        let resolved = if path.is_absolute() {
            path.clone()
        } else {
            root.join(path)
        };
        Store::open(&resolved)?
    } else {
        Store::discover(&root)?
    };

    // Build index for graph analysis
    let index = crate::lib::index::IndexBuilder::new(&store)
        .load_existing()?
        .build()?;

    // Find compaction candidates
    let candidates = find_compaction_candidates(&store, &index)?;

    // Output
    match cli.format {
        crate::lib::format::OutputFormat::Human => {
            if candidates.is_empty() {
                println!("No compaction candidates found.");
                println!();
                println!(
                    "Try creating more interconnected notes or adjusting clustering parameters."
                );
                return Ok(());
            }

            println!("Compaction Candidates");
            println!("====================");
            println!();

            for (i, candidate) in candidates.iter().enumerate() {
                println!("Candidate {} (score: {:.1})", i + 1, candidate.score);
                println!(
                    "  Notes: {} ({} chars total)",
                    candidate.node_count, candidate.estimated_size
                );
                println!(
                    "  Cohesion: {:.0}% ({} internal / {} boundary edges)",
                    candidate.cohesion * 100.0,
                    candidate.internal_edges,
                    candidate.boundary_edges
                );
                println!("  IDs: {}", candidate.ids.join(", "));
                println!();
                println!("  Next step:");
                let note_flags = candidate
                    .ids
                    .iter()
                    .map(|id| format!("--note {}", id))
                    .collect::<Vec<_>>()
                    .join(" ");
                println!(
                    "    qipu create \"Digest of {} notes\" --type permanent",
                    candidate.node_count
                );
                println!("    qipu compact apply <digest-id> {}", note_flags);
                println!();
            }
        }
        crate::lib::format::OutputFormat::Json => {
            let output: Vec<_> = candidates.iter().map(|c| {
                let note_flags = c.ids.iter()
                    .map(|id| format!("--note {}", id))
                    .collect::<Vec<_>>()
                    .join(" ");

                serde_json::json!({
                    "ids": c.ids,
                    "node_count": c.node_count,
                    "internal_edges": c.internal_edges,
                    "boundary_edges": c.boundary_edges,
                    "boundary_ratio": format!("{:.2}", c.boundary_ratio),
                    "cohesion": format!("{:.2}", c.cohesion),
                    "estimated_size": c.estimated_size,
                    "score": format!("{:.1}", c.score),
                    "suggested_command": format!("qipu compact apply <digest-id> {}", note_flags),
                })
            }).collect();

            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        crate::lib::format::OutputFormat::Records => {
            println!(
                "H qipu=1 records=1 mode=compact.suggest candidates={}",
                candidates.len()
            );
            for candidate in &candidates {
                println!(
                    "D candidate nodes={} internal={} boundary={} cohesion={:.2} size={} score={:.1}",
                    candidate.node_count,
                    candidate.internal_edges,
                    candidate.boundary_edges,
                    candidate.cohesion,
                    candidate.estimated_size,
                    candidate.score
                );
                for id in &candidate.ids {
                    println!("D candidate_id {}", id);
                }
            }
        }
    }

    Ok(())
}

/// Find compaction candidates using graph clustering
fn find_compaction_candidates(store: &Store, index: &Index) -> Result<Vec<CompactionCandidate>> {
    // Build adjacency list for clustering
    let mut adjacency: std::collections::HashMap<String, std::collections::HashSet<String>> =
        std::collections::HashMap::new();

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
    let mut visited = std::collections::HashSet::new();
    let mut components = Vec::new();

    for node_id in adjacency.keys() {
        if !visited.contains(node_id) {
            let component = find_component(&adjacency, node_id, &mut visited);
            if component.len() >= 3 {
                // Only consider components with at least 3 nodes
                components.push(component);
            }
        }
    }

    // Calculate metrics for each component
    let mut candidates = Vec::new();
    for component in components {
        if let Ok(candidate) = calculate_candidate_metrics(store, index, &component) {
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
    adjacency: &std::collections::HashMap<String, std::collections::HashSet<String>>,
    start: &str,
    visited: &mut std::collections::HashSet<String>,
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
    store: &Store,
    index: &Index,
    cluster: &[String],
) -> Result<CompactionCandidate> {
    let cluster_set: std::collections::HashSet<_> = cluster.iter().cloned().collect();

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
    for node_id in cluster {
        if let Ok(note) = store.get_note(node_id) {
            estimated_size += estimate_size(&note);
        }
    }

    // Calculate score
    let node_count = cluster.len();
    let size_score = (estimated_size as f64).ln().max(0.0);
    let cohesion_score = cohesion * 10.0;
    let boundary_penalty = boundary_ratio * -5.0;
    let node_score = (node_count as f64).sqrt();

    let score = size_score + cohesion_score + boundary_penalty + node_score;

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
