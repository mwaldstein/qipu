use std::path::Path;
use std::time::Instant;

use tracing::debug;

use crate::cli::Cli;
use qipu_core::compaction::CompactionCandidate;
use qipu_core::error::Result;

use super::utils::discover_compact_store;

fn output_human(candidates: &[CompactionCandidate]) {
    if candidates.is_empty() {
        println!("No compaction candidates found.");
        println!();
        println!("Try creating more interconnected notes or adjusting clustering parameters.");
        return;
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

fn output_json(candidates: &[CompactionCandidate]) -> Result<()> {
    let output: Vec<_> = candidates
        .iter()
        .map(|c| {
            let note_flags = c
                .ids
                .iter()
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
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn output_records(candidates: &[CompactionCandidate]) {
    println!(
        "H qipu=1 records=1 mode=compact.suggest candidates={}",
        candidates.len()
    );
    for candidate in candidates {
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

/// Execute `qipu compact suggest`
pub fn execute(cli: &Cli, root: &Path, include_mocs: bool) -> Result<()> {
    let start = Instant::now();
    if cli.verbose {
        debug!(suggest_params = ?include_mocs);
    }

    let store = discover_compact_store(cli, root)?;

    let index = qipu_core::index::IndexBuilder::new(&store).build()?;

    if cli.verbose {
        debug!("build_index");
    }

    let all_notes = store.list_notes()?;
    let ctx = qipu_core::compaction::CompactionContext::build(&all_notes)?;

    if cli.verbose {
        debug!(note_count = all_notes.len(), "build_compaction_context");
    }

    let candidates = ctx.suggest(&store, &index, include_mocs)?;

    if cli.verbose {
        debug!(
            candidate_count = candidates.len(),
            elapsed = ?start.elapsed(),
            "suggest_compaction"
        );
    }

    match cli.format {
        qipu_core::format::OutputFormat::Human => output_human(&candidates),
        qipu_core::format::OutputFormat::Json => output_json(&candidates)?,
        qipu_core::format::OutputFormat::Records => output_records(&candidates),
    }

    Ok(())
}
