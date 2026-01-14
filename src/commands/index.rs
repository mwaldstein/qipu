//! `qipu index` command - build/refresh derived indexes
//!
//! Per spec (specs/cli-interface.md, specs/indexing-search.md):
//! - `qipu index` - build/refresh indexes
//! - `qipu index --rebuild` - drop and regenerate

use crate::cli::{Cli, OutputFormat};
use crate::lib::error::Result;
use crate::lib::index::IndexBuilder;
use crate::lib::store::Store;

/// Execute the index command
pub fn execute(cli: &Cli, store: &Store, rebuild: bool) -> Result<()> {
    let builder = IndexBuilder::new(store);

    let builder = if rebuild {
        builder.rebuild()
    } else {
        builder.load_existing()?
    };

    let index = builder.build()?;

    // Save index to cache
    let cache_dir = store.root().join(".cache");
    index.save(&cache_dir)?;

    match cli.format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "status": "ok",
                "notes_indexed": index.metadata.len(),
                "tags_indexed": index.tags.len(),
                "edges_indexed": index.edges.len(),
                "unresolved_links": index.unresolved.len(),
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Records => {
            // Header line with index statistics
            let store_path = cli
                .store
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| ".qipu".to_string());

            println!(
                "H qipu=1 records=1 store={} mode=index notes={} tags={} edges={} unresolved={}",
                store_path,
                index.metadata.len(),
                index.tags.len(),
                index.edges.len(),
                index.unresolved.len()
            );

            // Output unresolved links as diagnostic lines if any exist
            if !index.unresolved.is_empty() {
                let mut unresolved = index.unresolved.iter().cloned().collect::<Vec<_>>();
                unresolved.sort();
                for unresolved_id in unresolved {
                    println!("D warning unresolved-link \"{}\"", unresolved_id);
                }
            }
        }
        OutputFormat::Human => {
            if !cli.quiet {
                println!("Indexed {} notes", index.metadata.len());
                if cli.verbose {
                    println!("  {} tags", index.tags.len());
                    println!("  {} edges", index.edges.len());
                    if !index.unresolved.is_empty() {
                        let mut unresolved = index.unresolved.iter().cloned().collect::<Vec<_>>();
                        unresolved.sort();
                        println!("  {} unresolved links: {:?}", unresolved.len(), unresolved);
                    }
                }
            }
        }
    }

    Ok(())
}
