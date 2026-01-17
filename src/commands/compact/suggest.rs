use std::path::PathBuf;

use crate::cli::Cli;
use crate::lib::error::Result;
use crate::lib::store::Store;

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
    let all_notes = store.list_notes()?;
    let ctx = crate::lib::compaction::CompactionContext::build(&all_notes)?;
    let candidates = ctx.suggest(&store, &index)?;

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
