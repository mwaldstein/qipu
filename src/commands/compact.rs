//! Compaction command implementations
//!
//! Per spec: specs/compaction.md

use std::fs;
use std::io::{self, BufRead};
use std::path::PathBuf;

use crate::cli::{Cli, CompactCommands};
use crate::lib::compaction::CompactionContext;
use crate::lib::error::Result;
use crate::lib::note::Note;
use crate::lib::store::Store;

/// Execute compact subcommand
pub fn execute(cli: &Cli, command: &CompactCommands) -> Result<()> {
    match command {
        CompactCommands::Apply {
            digest_id,
            note,
            from_stdin,
            notes_file,
        } => execute_apply(cli, digest_id, note, *from_stdin, notes_file.as_ref()),
        CompactCommands::Show {
            digest_id,
            compaction_depth,
        } => execute_show(cli, digest_id, *compaction_depth),
        CompactCommands::Status { id } => execute_status(cli, id),
        CompactCommands::Report { digest_id } => execute_report(cli, digest_id),
        CompactCommands::Suggest => execute_suggest(cli),
        CompactCommands::Guide => execute_guide(cli),
    }
}

/// Execute `qipu compact apply`
fn execute_apply(
    cli: &Cli,
    digest_id: &str,
    note_ids: &[String],
    from_stdin: bool,
    notes_file: Option<&PathBuf>,
) -> Result<()> {
    // Discover store
    let root = cli
        .root
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap());
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

    // Collect source note IDs from various sources
    let mut source_ids = note_ids.to_vec();

    // Read from stdin if requested
    if from_stdin {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line?;
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                source_ids.push(trimmed.to_string());
            }
        }
    }

    // Read from file if requested
    if let Some(file_path) = notes_file {
        let content = fs::read_to_string(file_path)?;
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                source_ids.push(trimmed.to_string());
            }
        }
    }

    if source_ids.is_empty() {
        return Err(crate::lib::error::QipuError::UsageError(
            "no source note IDs provided (use --note, --from-stdin, or --notes-file)".to_string(),
        ));
    }

    // Load the digest note
    let mut digest_note = store.get_note(digest_id)?;

    // Sort source IDs for deterministic ordering
    source_ids.sort();
    source_ids.dedup();

    // Update compacts field (idempotent - merge with existing)
    let mut existing_compacts: std::collections::HashSet<String> =
        digest_note.frontmatter.compacts.iter().cloned().collect();
    for id in &source_ids {
        existing_compacts.insert(id.clone());
    }
    let mut new_compacts: Vec<String> = existing_compacts.into_iter().collect();
    new_compacts.sort();
    digest_note.frontmatter.compacts = new_compacts;

    // Validate invariants before saving
    let all_notes = store.list_notes()?;
    let ctx = CompactionContext::build(&all_notes)?;
    let errors = ctx.validate(&all_notes);
    if !errors.is_empty() {
        eprintln!("Compaction validation errors:");
        for err in &errors {
            eprintln!("  - {}", err);
        }
        return Err(crate::lib::error::QipuError::Other(
            "compaction invariants violated".to_string(),
        ));
    }

    // Save the digest note
    store.save_note(&mut digest_note)?;

    // Output
    match cli.format {
        crate::lib::format::OutputFormat::Human => {
            println!("Applied compaction:");
            println!("  Digest: {}", digest_id);
            println!("  Compacts {} notes:", source_ids.len());
            for id in &source_ids {
                println!("    - {}", id);
            }
        }
        crate::lib::format::OutputFormat::Json => {
            let output = serde_json::json!({
                "digest_id": digest_id,
                "compacts": source_ids,
                "count": source_ids.len(),
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        crate::lib::format::OutputFormat::Records => {
            println!(
                "H qipu=1 records=1 mode=compact.apply digest={} count={}",
                digest_id,
                source_ids.len()
            );
            for id in &source_ids {
                println!("D compacted {}", id);
            }
        }
    }

    Ok(())
}

/// Execute `qipu compact show`
fn execute_show(cli: &Cli, digest_id: &str, depth: u32) -> Result<()> {
    let root = cli
        .root
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap());
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

    // Get direct compacted notes
    let direct_compacts = ctx
        .get_compacted_notes(digest_id)
        .cloned()
        .unwrap_or_default();

    if direct_compacts.is_empty() {
        match cli.format {
            crate::lib::format::OutputFormat::Human => {
                println!("Note {} does not compact any notes", digest_id);
            }
            crate::lib::format::OutputFormat::Json => {
                let output = serde_json::json!({
                    "digest_id": digest_id,
                    "compacts": [],
                    "count": 0,
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            }
            crate::lib::format::OutputFormat::Records => {
                println!(
                    "H qipu=1 records=1 mode=compact.show digest={} count=0",
                    digest_id
                );
            }
        }
        return Ok(());
    }

    // Compute compaction metrics
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

    // Output
    match cli.format {
        crate::lib::format::OutputFormat::Human => {
            println!("Digest: {}", digest_id);
            println!("Direct compaction count: {}", direct_compacts.len());
            println!("Compaction: {:.1}%", compaction_pct);
            println!();
            println!("Compacted notes:");
            for id in &direct_compacts {
                if let Ok(note) = store.get_note(id) {
                    println!("  - {} ({})", note.frontmatter.title, id);
                } else {
                    println!("  - {} (not found)", id);
                }
            }

            // Show nested compaction if depth > 1
            if depth > 1 {
                println!();
                println!("Nested compaction (depth {}):", depth);
                show_nested_compaction(&store, &ctx, digest_id, 1, depth)?;
            }
        }
        crate::lib::format::OutputFormat::Json => {
            let output = serde_json::json!({
                "digest_id": digest_id,
                "compacts": direct_compacts,
                "count": direct_compacts.len(),
                "compaction_pct": format!("{:.1}", compaction_pct),
                "depth": depth,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        crate::lib::format::OutputFormat::Records => {
            println!(
                "H qipu=1 records=1 mode=compact.show digest={} count={} compaction={:.1}%",
                digest_id,
                direct_compacts.len(),
                compaction_pct
            );
            for id in &direct_compacts {
                println!("D compacted {}", id);
            }
        }
    }

    Ok(())
}

/// Show nested compaction recursively (helper for show command)
fn show_nested_compaction(
    store: &Store,
    ctx: &CompactionContext,
    current_id: &str,
    current_depth: u32,
    max_depth: u32,
) -> Result<()> {
    if current_depth >= max_depth {
        return Ok(());
    }

    if let Some(compacts) = ctx.get_compacted_notes(current_id) {
        for source_id in compacts {
            let indent = "  ".repeat(current_depth as usize);
            if let Ok(note) = store.get_note(source_id) {
                println!("{}  - {} ({})", indent, note.frontmatter.title, source_id);
                show_nested_compaction(store, ctx, source_id, current_depth + 1, max_depth)?;
            }
        }
    }

    Ok(())
}

/// Execute `qipu compact status`
fn execute_status(cli: &Cli, note_id: &str) -> Result<()> {
    let root = cli
        .root
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap());
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

    // Get compaction status
    let canonical = ctx.canon(note_id)?;
    let direct_compactor = ctx.get_compactor(note_id);
    let compacted_notes = ctx.get_compacted_notes(note_id);

    // Output
    match cli.format {
        crate::lib::format::OutputFormat::Human => {
            let note = store.get_note(note_id)?;
            println!("Note: {} ({})", note.frontmatter.title, note_id);
            println!();

            if let Some(compactor) = direct_compactor {
                let compactor_note = store.get_note(compactor)?;
                println!(
                    "  Compacted by: {} ({})",
                    compactor_note.frontmatter.title, compactor
                );
            } else {
                println!("  Compacted by: (none)");
            }

            if canonical != note_id {
                let canonical_note = store.get_note(&canonical)?;
                println!(
                    "  Canonical: {} ({})",
                    canonical_note.frontmatter.title, canonical
                );
            } else {
                println!("  Canonical: (self)");
            }

            if let Some(compacts) = compacted_notes {
                println!("  Compacts {} notes:", compacts.len());
                for id in compacts {
                    if let Ok(n) = store.get_note(id) {
                        println!("    - {} ({})", n.frontmatter.title, id);
                    }
                }
            } else {
                println!("  Compacts: (none)");
            }
        }
        crate::lib::format::OutputFormat::Json => {
            let output = serde_json::json!({
                "note_id": note_id,
                "compactor": direct_compactor,
                "canonical": canonical,
                "compacts": compacted_notes,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        crate::lib::format::OutputFormat::Records => {
            println!("H qipu=1 records=1 mode=compact.status note={}", note_id);
            if let Some(compactor) = direct_compactor {
                println!("D compactor {}", compactor);
            }
            if canonical != note_id {
                println!("D canonical {}", canonical);
            }
            if let Some(compacts) = compacted_notes {
                for id in compacts {
                    println!("D compacts {}", id);
                }
            }
        }
    }

    Ok(())
}

/// Execute `qipu compact report`
fn execute_report(cli: &Cli, digest_id: &str) -> Result<()> {
    let root = cli
        .root
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap());
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
    let index = crate::lib::index::IndexBuilder::new(&store)
        .load_existing()?
        .build()?;

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

/// Execute `qipu compact suggest` (placeholder for now)
fn execute_suggest(_cli: &Cli) -> Result<()> {
    // TODO: Implement suggest with graph clustering
    // - Find dense, self-contained clumps
    // - Rank by size, node count, cohesion, boundary edges
    // - Output suggested compaction candidates
    Err(crate::lib::error::QipuError::Other(
        "compact suggest not yet implemented".to_string(),
    ))
}

/// Execute `qipu compact guide`
fn execute_guide(_cli: &Cli) -> Result<()> {
    println!(
        r#"# Qipu Compaction Guide

Compaction allows you to create digest notes that summarize and replace sets of notes
in day-to-day retrieval, while keeping the original notes intact.

## Workflow

1. **Find candidates**: Use `qipu compact suggest` to find groups of notes that might
   benefit from compaction (dense, self-contained clusters).

2. **Review summaries**: Use `qipu context --format records` to review candidate notes
   in summary form before authoring a digest.

3. **Author a digest**: Create a new note that summarizes the candidate notes.
   
   Guidelines for digests:
   - Include a one-paragraph Summary
   - List key claims or insights
   - Add a section explaining when to expand into source notes
   - Keep it concise (shorter than the expanded sources)
   - Include source note IDs for traceability

   Example prompt for LLM:
   "Create a digest note that replaces these notes in day-to-day retrieval.
   Include a one-paragraph Summary, key claims, and a small section explaining
   when to expand into sources. Keep it short; include IDs for traceability."

4. **Register compaction**: Use `qipu compact apply <digest-id> --note <id>...`
   to register the compaction relationship.

5. **Validate**: Use `qipu compact report <digest-id>` to check compaction quality.
   Also run `qipu doctor` to validate invariants.

## Commands

- `qipu compact apply <digest> --note <id>...` - Register compaction
- `qipu compact show <digest>` - Show what a digest compacts
- `qipu compact status <id>` - Show compaction relationships for a note
- `qipu compact report <digest>` - Quality metrics (coming soon)
- `qipu compact suggest` - Suggest compaction candidates (coming soon)
- `qipu compact guide` - Print this guide

## Invariants

Compaction must satisfy these invariants:

- At most one compactor per note
- No cycles in compaction chains
- No self-compaction
- All referenced IDs must exist

Use `qipu doctor` to validate compaction invariants.
"#
    );

    Ok(())
}

/// Estimate note size for compaction metrics
/// Uses summary-sized content (same as records output)
fn estimate_size(note: &Note) -> usize {
    // Use summary if present
    if let Some(summary) = &note.frontmatter.summary {
        return summary.len();
    }

    // Otherwise use first paragraph or truncated body
    let summary = note.summary();
    summary.len()
}
