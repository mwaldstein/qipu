use std::path::PathBuf;

use crate::cli::Cli;
use crate::lib::compaction::CompactionContext;
use crate::lib::error::Result;
use crate::lib::store::Store;

use super::utils::estimate_size;

/// Execute `qipu compact show`
pub fn execute(cli: &Cli, digest_id: &str, depth: u32) -> Result<()> {
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
