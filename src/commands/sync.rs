//! `qipu sync` command - synchronize indexes and optionally validate
//!
//! Per spec (specs/cli-interface.md):
//! - Ensure derived indexes are up to date
//! - Optionally run validations
//! - Optional convenience command for multi-agent workflows

use crate::cli::{Cli, OutputFormat};
use crate::commands::doctor;
use crate::lib::error::Result;
use crate::lib::index::IndexBuilder;
use crate::lib::store::Store;

/// Execute the sync command
pub fn execute(cli: &Cli, store: &Store, validate: bool, fix: bool) -> Result<()> {
    // Step 1: Update indexes silently
    let builder = IndexBuilder::new(store);
    let builder = builder.load_existing()?;
    let index = builder.build()?;

    // Save index to cache
    let cache_dir = store.root().join(".cache");
    index.save(&cache_dir)?;

    let notes_indexed = index.metadata.len();
    let tags_indexed = index.tags.len();
    let edges_indexed = index.edges.len();

    // Step 2: Optionally validate
    let (_errors, _warnings, _fixed) = if validate {
        // Run doctor quietly - it will output its own results
        // We would need to refactor doctor to return structured data to capture results
        doctor::execute(cli, store, fix)?;

        // Placeholder values since we can't capture doctor's actual results yet
        (0, 0, 0)
    } else {
        (0, 0, 0)
    };

    // Output based on format - but only if doctor wasn't run or we're in human mode
    // In JSON/Records mode, doctor will output its own structured result
    if !validate || cli.format == OutputFormat::Human {
        match cli.format {
            OutputFormat::Human => {
                if !cli.quiet {
                    println!("Indexed {} notes", notes_indexed);
                    if validate {
                        println!("Store validated");
                    }
                }
            }
            OutputFormat::Json => {
                let output = serde_json::json!({
                    "status": "ok",
                    "notes_indexed": notes_indexed,
                    "tags_indexed": tags_indexed,
                    "edges_indexed": edges_indexed,
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            }
            OutputFormat::Records => {
                // Header line per spec (specs/records-output.md)
                println!(
                    "H qipu=1 records=1 store={} mode=sync notes={} tags={} edges={}",
                    store.root().display(),
                    notes_indexed,
                    tags_indexed,
                    edges_indexed
                );
            }
        }
    }

    Ok(())
}
