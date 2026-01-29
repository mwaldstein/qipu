use std::path::PathBuf;
use std::time::Instant;

use tracing::debug;

use crate::cli::Cli;
use qipu_core::compaction::CompactionContext;
use qipu_core::error::Result;
use qipu_core::store::Store;

use super::utils::estimate_size;

/// Execute `qipu compact show`
pub fn execute(cli: &Cli, digest_id: &str, depth: u32) -> Result<()> {
    let start = Instant::now();
    if cli.verbose {
        debug!(digest_id, depth, "show_params");
    }

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

    if cli.verbose {
        debug!(store = %store.root().display(), "discover_store");
    }

    let all_notes = store.list_notes()?;
    let ctx = CompactionContext::build(&all_notes)?;

    if cli.verbose {
        debug!(note_count = all_notes.len(), "build_compaction_context");
    }

    // Get direct compacted notes with truncation support
    // Use depth=1 for direct compaction only when getting the main list
    let (direct_compacts, truncated) = ctx
        .get_compacted_ids(digest_id, 1, cli.compaction_max_nodes)
        .unwrap_or_else(|| (Vec::new(), false));

    if direct_compacts.is_empty() {
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

    let depth_tree = if depth > 1 {
        let mut tree = build_compaction_tree(&store, &ctx, digest_id, 0, depth)?;
        // Apply max_nodes limit to tree if specified
        if let Some(max) = cli.compaction_max_nodes {
            if tree.len() > max {
                tree.truncate(max);
            }
        }
        tree
    } else {
        Vec::new()
    };

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

    // Output
    match cli.format {
        qipu_core::format::OutputFormat::Human => {
            println!("Digest: {}", digest_id);
            println!("Direct compaction count: {}", direct_compacts.len());
            if truncated {
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
        qipu_core::format::OutputFormat::Json => {
            let mut output = serde_json::json!({
                "digest_id": digest_id,
                "compacts": direct_compacts,
                "count": direct_compacts.len(),
                "compaction_pct": format!("{:.1}", compaction_pct),
                "depth": depth,
                "tree": depth_tree,
            });

            // Add truncated flag if applicable
            if truncated {
                if let Some(obj) = output.as_object_mut() {
                    obj.insert(
                        "compacted_ids_truncated".to_string(),
                        serde_json::json!(true),
                    );
                }
            }

            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        qipu_core::format::OutputFormat::Records => {
            println!(
                "H qipu=1 records=1 mode=compact.show digest={} count={} compaction={:.1}% depth={}",
                digest_id,
                direct_compacts.len(),
                compaction_pct,
                depth
            );
            for id in &direct_compacts {
                println!("D compacted {}", id);
            }

            // Add truncation marker if applicable
            if truncated {
                let total_count = ctx.get_compacts_count(digest_id);
                println!(
                    "D compacted_truncated max={} total={}",
                    cli.compaction_max_nodes.unwrap_or(direct_compacts.len()),
                    total_count
                );
            }

            if depth > 1 {
                for entry in depth_tree {
                    println!(
                        "D compacted_tree from={} to={} depth={}",
                        entry.from, entry.to, entry.depth
                    );
                }
            }
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
