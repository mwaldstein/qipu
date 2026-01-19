use std::path::PathBuf;

use crate::cli::Cli;
use crate::lib::compaction::CompactionContext;
use crate::lib::error::Result;
use crate::lib::store::Store;

use super::utils::estimate_size;

/// Execute `qipu compact report`
pub fn execute(cli: &Cli, digest_id: &str) -> Result<()> {
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

    let all_notes = store.list_notes()?;
    let ctx = CompactionContext::build(&all_notes)?;

    // Build index for edge analysis
    let index = crate::lib::index::IndexBuilder::new(&store).build()?;

    // Get direct compacted notes
    let direct_compacts = ctx
        .get_compacted_notes(digest_id)
        .cloned()
        .unwrap_or_default();

    if direct_compacts.is_empty() {
        return Err(crate::lib::error::QipuError::Other(format!(
            "Note {} does not compact any notes",
            digest_id
        )));
    }

    // 1. Direct compaction count
    let compacts_direct_count = direct_compacts.len();

    // 2. Compaction percentage
    let digest_note = store.get_note(digest_id)?;
    let digest_size = estimate_size(&digest_note);
    let mut expanded_size = 0;
    for source_id in &direct_compacts {
        if let Ok(note) = store.get_note(source_id) {
            expanded_size += estimate_size(&note);
        }
    }
    let compaction_pct = if expanded_size > 0 {
        100.0 * (1.0 - (digest_size as f64 / expanded_size as f64))
    } else {
        0.0
    };

    // 3. Boundary edge ratio
    // Count edges from compacted notes that point outside the compaction set
    let compacted_set: std::collections::HashSet<_> = direct_compacts.iter().cloned().collect();
    let mut internal_edges = 0;
    let mut boundary_edges = 0;

    for source_id in &direct_compacts {
        let outbound_edges = index.get_outbound_edges(source_id);
        for edge in outbound_edges {
            if compacted_set.contains(&edge.to) {
                internal_edges += 1;
            } else {
                boundary_edges += 1;
            }
        }
    }

    let total_edges = internal_edges + boundary_edges;
    let boundary_edge_ratio = if total_edges > 0 {
        (boundary_edges as f64) / (total_edges as f64)
    } else {
        0.0
    };

    // 4. Staleness indicator
    // Check if any source note was updated after the digest
    let digest_updated = digest_note.frontmatter.updated;
    let mut stale_sources = Vec::new();

    for source_id in &direct_compacts {
        if let Ok(note) = store.get_note(source_id) {
            if let (Some(digest_time), Some(source_time)) =
                (digest_updated, note.frontmatter.updated)
            {
                if source_time > digest_time {
                    stale_sources.push(source_id.clone());
                }
            }
        }
    }

    let is_stale = !stale_sources.is_empty();
    let staleness_count = stale_sources.len();

    // 5. Conflicts/cycles
    let validation_errors = ctx.validate(&all_notes);
    let has_conflicts = !validation_errors.is_empty();

    // Output
    match cli.format {
        crate::lib::format::OutputFormat::Human => {
            println!("Compaction Report: {}", digest_id);
            println!("=================={}", "=".repeat(digest_id.len()));
            println!();
            println!("Compaction Metrics:");
            println!("  Direct count: {}", compacts_direct_count);
            println!("  Compaction: {:.1}%", compaction_pct);
            println!();
            println!("Edge Analysis:");
            println!("  Internal edges: {}", internal_edges);
            println!("  Boundary edges: {}", boundary_edges);
            println!("  Boundary ratio: {:.2}", boundary_edge_ratio);
            println!();
            println!("Staleness:");
            if is_stale {
                println!(
                    "  Status: STALE (digest older than {} sources)",
                    staleness_count
                );
                println!("  Stale sources:");
                for source_id in &stale_sources {
                    if let Ok(note) = store.get_note(source_id) {
                        println!("    - {} ({})", note.frontmatter.title, source_id);
                    }
                }
            } else {
                println!("  Status: CURRENT (digest up to date)");
            }
            println!();
            println!("Invariants:");
            if has_conflicts {
                println!("  Status: INVALID");
                println!("  Errors:");
                for err in &validation_errors {
                    println!("    - {}", err);
                }
            } else {
                println!("  Status: VALID (no conflicts or cycles)");
            }
        }
        crate::lib::format::OutputFormat::Json => {
            let output = serde_json::json!({
                "digest_id": digest_id,
                "compacts_direct_count": compacts_direct_count,
                "compaction_pct": format!("{:.1}", compaction_pct),
                "edges": {
                    "internal": internal_edges,
                    "boundary": boundary_edges,
                    "boundary_ratio": format!("{:.2}", boundary_edge_ratio),
                },
                "staleness": {
                    "is_stale": is_stale,
                    "stale_count": staleness_count,
                    "stale_sources": stale_sources,
                },
                "invariants": {
                    "valid": !has_conflicts,
                    "errors": validation_errors,
                },
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        crate::lib::format::OutputFormat::Records => {
            println!(
                "H qipu=1 records=1 mode=compact.report digest={} count={} compaction={:.1}% boundary_ratio={:.2} stale={} valid={}",
                digest_id,
                compacts_direct_count,
                compaction_pct,
                boundary_edge_ratio,
                is_stale,
                !has_conflicts
            );
            println!("D internal_edges {}", internal_edges);
            println!("D boundary_edges {}", boundary_edges);
            if is_stale {
                println!("D stale_count {}", staleness_count);
                for source_id in &stale_sources {
                    println!("D stale_source {}", source_id);
                }
            }
            if has_conflicts {
                for err in &validation_errors {
                    println!("D error {}", err);
                }
            }
        }
    }

    Ok(())
}
