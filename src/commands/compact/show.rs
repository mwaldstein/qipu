//! Show command for displaying compaction relationships
//!
//! Displays which notes are compacted by a digest note,
//! supporting multiple output formats and depth levels.

use std::path::Path;
use std::time::Instant;

use tracing::debug;

use crate::cli::Cli;
use qipu_core::compaction::CompactionContext;
use qipu_core::error::Result;
use qipu_core::store::Store;

use super::utils::{discover_compact_store, estimate_size};

/// Output format when a digest note compacts no other notes
fn handle_empty_compaction(cli: &Cli, digest_id: &str) -> Result<()> {
    match cli.format {
        qipu_core::format::OutputFormat::Human => {
            println!("Note {} does not compact any notes", digest_id);
        }
        qipu_core::format::OutputFormat::Json => {
            let output = serde_json::json!({
                "digest_id": digest_id,
                "compacts": [],
                "count": 0,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        qipu_core::format::OutputFormat::Records => {
            println!(
                "H qipu=1 records=1 mode=compact.show digest={} count=0",
                digest_id
            );
        }
    }
    Ok(())
}

/// Calculate compaction metrics (size reduction percentage)
/// When depth > 1, calculates depth-aware metrics including all nested sources
fn compute_compaction_metrics(
    store: &Store,
    ctx: &CompactionContext,
    digest_id: &str,
    direct_compacts: &[String],
    depth: u32,
) -> Result<f64> {
    let digest_note = store.get_note(digest_id)?;

    if depth <= 1 {
        // Original behavior: only direct sources
        let digest_size = estimate_size(&digest_note);
        let mut expanded_size = 0;
        for source_id in direct_compacts {
            if let Ok(note) = store.get_note(source_id) {
                expanded_size += estimate_size(&note);
            }
        }
        Ok(if expanded_size > 0 {
            100.0 * (1.0 - (digest_size as f64 / expanded_size as f64))
        } else {
            0.0
        })
    } else {
        // Depth-aware metrics: include all sources up to specified depth
        let all_notes = store.list_notes()?;
        let note_map = CompactionContext::build_note_map(&all_notes);
        Ok(ctx
            .get_compaction_pct_at_depth(&digest_note, &note_map, depth)
            .unwrap_or(0.0) as f64)
    }
}

/// Build depth tree if depth > 1, with optional node limit
fn build_depth_tree_if_needed(
    store: &Store,
    ctx: &CompactionContext,
    digest_id: &str,
    depth: u32,
    max_nodes: Option<usize>,
) -> Result<Vec<CompactionTreeEntry>> {
    let mut tree = if depth > 1 {
        build_compaction_tree(store, ctx, digest_id, 0, depth)?
    } else {
        Vec::new()
    };
    if let Some(max) = max_nodes {
        if tree.len() > max {
            tree.truncate(max);
        }
    }
    Ok(tree)
}

/// Context for output formatting to reduce argument count
struct OutputContext<'a> {
    store: &'a Store,
    ctx: &'a CompactionContext,
    cli: &'a Cli,
    digest_id: &'a str,
    direct_compacts: &'a [String],
    compaction_pct: f64,
    truncated: bool,
    depth: u32,
}

/// Output compaction info in human-readable format
fn output_human_format(oc: &OutputContext<'_>) -> Result<()> {
    let OutputContext {
        store,
        ctx,
        cli,
        digest_id,
        direct_compacts,
        compaction_pct,
        truncated,
        depth,
    } = oc;
    println!("Digest: {}", digest_id);
    println!("Direct compaction count: {}", direct_compacts.len());
    if *truncated {
        let total_count = ctx.get_compacts_count(digest_id);
        println!(
            "  (truncated: showing {} of {} notes)",
            cli.compaction_max_nodes.unwrap_or(direct_compacts.len()),
            total_count
        );
    }
    println!("Compaction: {:.1}%", compaction_pct);
    println!();
    println!("Compacted notes:");
    for id in *direct_compacts {
        if let Ok(note) = store.get_note(id) {
            println!("  - {} ({})", note.frontmatter.title, id);
        } else {
            println!("  - {} (not found)", id);
        }
    }

    if *depth > 1 {
        println!();
        println!("Nested compaction (depth {}):", depth);
        show_nested_compaction(store, ctx, digest_id, 1, *depth)?;
    }
    Ok(())
}

/// Output compaction info in JSON format
fn output_json_format(
    direct_compacts: &[String],
    depth_tree: &[CompactionTreeEntry],
    digest_id: &str,
    compaction_pct: f64,
    depth: u32,
    truncated: bool,
) -> Result<()> {
    let mut output = serde_json::json!({
        "digest_id": digest_id,
        "compacts": direct_compacts,
        "count": direct_compacts.len(),
        "compaction_pct": format!("{:.1}", compaction_pct),
        "depth": depth,
        "tree": depth_tree,
    });

    if truncated {
        if let Some(obj) = output.as_object_mut() {
            obj.insert(
                "compacted_ids_truncated".to_string(),
                serde_json::json!(true),
            );
        }
    }

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Context for records format with tree data
struct RecordsOutputContext<'a> {
    ctx: &'a CompactionContext,
    cli: &'a Cli,
    digest_id: &'a str,
    direct_compacts: &'a [String],
    compaction_pct: f64,
    depth: u32,
    depth_tree: &'a [CompactionTreeEntry],
    truncated: bool,
}

/// Output compaction info in records format
fn output_records_format(oc: &RecordsOutputContext<'_>) -> Result<()> {
    let RecordsOutputContext {
        ctx,
        cli,
        digest_id,
        direct_compacts,
        compaction_pct,
        depth,
        depth_tree,
        truncated,
    } = oc;
    println!(
        "H qipu=1 records=1 mode=compact.show digest={} count={} compaction={:.1}% depth={}",
        digest_id,
        direct_compacts.len(),
        compaction_pct,
        depth
    );
    for id in *direct_compacts {
        println!("D compacted {}", id);
    }

    if *truncated {
        let total_count = ctx.get_compacts_count(digest_id);
        println!(
            "D compacted_truncated max={} total={}",
            cli.compaction_max_nodes.unwrap_or(direct_compacts.len()),
            total_count
        );
    }

    if *depth > 1 {
        for entry in *depth_tree {
            println!(
                "D compacted_tree from={} to={} depth={}",
                entry.from, entry.to, entry.depth
            );
        }
    }
    Ok(())
}

/// Execute `qipu compact show`
pub fn execute(cli: &Cli, root: &Path, digest_id: &str, depth: u32) -> Result<()> {
    let start = Instant::now();
    if cli.verbose {
        debug!(digest_id, depth, "show_params");
    }

    let store = discover_compact_store(cli, root)?;

    let all_notes = store.list_notes()?;
    let ctx = CompactionContext::build(&all_notes)?;

    if cli.verbose {
        debug!(note_count = all_notes.len(), "build_compaction_context");
    }

    let (direct_compacts, truncated) = ctx
        .get_compacted_ids(digest_id, 1, cli.compaction_max_nodes)
        .unwrap_or_else(|| (Vec::new(), false));

    if direct_compacts.is_empty() {
        return handle_empty_compaction(cli, digest_id);
    }

    let compaction_pct =
        compute_compaction_metrics(&store, &ctx, digest_id, &direct_compacts, depth)?;
    let depth_tree =
        build_depth_tree_if_needed(&store, &ctx, digest_id, depth, cli.compaction_max_nodes)?;

    if cli.verbose {
        debug!(
            digest_id,
            direct_count = direct_compacts.len(),
            compaction_pct = format!("{:.1}", compaction_pct),
            depth,
            tree_entries = depth_tree.len(),
            elapsed = ?start.elapsed(),
            "show_compaction"
        );
    }

    match cli.format {
        qipu_core::format::OutputFormat::Human => {
            output_human_format(&OutputContext {
                store: &store,
                ctx: &ctx,
                cli,
                digest_id,
                direct_compacts: &direct_compacts,
                compaction_pct,
                truncated,
                depth,
            })?;
        }
        qipu_core::format::OutputFormat::Json => {
            output_json_format(
                &direct_compacts,
                &depth_tree,
                digest_id,
                compaction_pct,
                depth,
                truncated,
            )?;
        }
        qipu_core::format::OutputFormat::Records => {
            output_records_format(&RecordsOutputContext {
                ctx: &ctx,
                cli,
                digest_id,
                direct_compacts: &direct_compacts,
                compaction_pct,
                depth,
                depth_tree: &depth_tree,
                truncated,
            })?;
        }
    }

    Ok(())
}

/// Show nested compaction recursively (helper for show command)
#[derive(Debug, Clone, serde::Serialize)]
struct CompactionTreeEntry {
    from: String,
    to: String,
    depth: u32,
}

/// Recursively build the compaction tree structure
fn build_compaction_tree(
    store: &Store,
    ctx: &CompactionContext,
    root_id: &str,
    current_depth: u32,
    max_depth: u32,
) -> Result<Vec<CompactionTreeEntry>> {
    if current_depth >= max_depth {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    if let Some(compacts) = ctx.get_compacted_notes(root_id) {
        let mut sorted = compacts.clone();
        sorted.sort();
        for source_id in sorted {
            if store.get_note(&source_id).is_ok() {
                entries.push(CompactionTreeEntry {
                    from: root_id.to_string(),
                    to: source_id.clone(),
                    depth: current_depth + 1,
                });
                let mut nested =
                    build_compaction_tree(store, ctx, &source_id, current_depth + 1, max_depth)?;
                entries.append(&mut nested);
            }
        }
    }

    Ok(entries)
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
        let mut sorted = compacts.clone();
        sorted.sort();
        for source_id in sorted {
            let indent = "  ".repeat(current_depth as usize);
            if let Ok(note) = store.get_note(&source_id) {
                println!("{}  - {} ({})", indent, note.frontmatter.title, source_id);
                show_nested_compaction(store, ctx, &source_id, current_depth + 1, max_depth)?;
            }
        }
    }

    Ok(())
}
