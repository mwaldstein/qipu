//! `qipu search` command - search notes
//!
//! Per spec (specs/cli-interface.md, specs/indexing-search.md):
//! - `qipu search <query>` - search titles + bodies
//! - `--type` filter
//! - `--tag` filter
//! - Result ranking: title > exact tag > body, recency boost

use crate::cli::{Cli, OutputFormat};
use crate::lib::error::Result;
use crate::lib::index::{search, Index, IndexBuilder};
use crate::lib::note::NoteType;
use crate::lib::store::Store;

/// Execute the search command
pub fn execute(
    cli: &Cli,
    store: &Store,
    query: &str,
    type_filter: Option<NoteType>,
    tag_filter: Option<&str>,
    exclude_mocs: bool,
) -> Result<()> {
    // Load or build index
    let cache_dir = store.root().join(".cache");
    let index = match Index::load(&cache_dir) {
        Ok(idx) if !idx.metadata.is_empty() => idx,
        _ => {
            // Index doesn't exist or is empty - build it
            if cli.verbose {
                eprintln!("Building index...");
            }
            let index = IndexBuilder::new(store).build()?;
            index.save(&cache_dir)?;
            index
        }
    };

    let mut results = search(store, &index, query, type_filter, tag_filter)?;

    // Apply exclude_mocs filter if requested
    if exclude_mocs {
        results.retain(|r| r.note_type != NoteType::Moc);
    }

    match cli.format {
        OutputFormat::Json => {
            let output: Vec<_> = results
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "id": r.id,
                        "title": r.title,
                        "type": r.note_type.to_string(),
                        "tags": r.tags,
                        "path": r.path,
                        "match_context": r.match_context,
                        "relevance": r.relevance,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Human => {
            if results.is_empty() {
                if !cli.quiet {
                    println!("No results found for '{}'", query);
                }
            } else {
                for result in &results {
                    let type_indicator = match result.note_type {
                        NoteType::Fleeting => "F",
                        NoteType::Literature => "L",
                        NoteType::Permanent => "P",
                        NoteType::Moc => "M",
                    };
                    println!("{} [{}] {}", result.id, type_indicator, result.title);
                    if cli.verbose {
                        if let Some(ctx) = &result.match_context {
                            println!("    {}", ctx);
                        }
                    }
                }
            }
        }
        OutputFormat::Records => {
            // Header line per spec (specs/records-output.md)
            println!(
                "H qipu=1 records=1 store={} mode=search query=\"{}\" results={}",
                store.root().display(),
                query.replace('"', "\\\""),
                results.len()
            );
            for result in &results {
                let tags_csv = if result.tags.is_empty() {
                    "-".to_string()
                } else {
                    result.tags.join(",")
                };
                println!(
                    "N {} {} \"{}\" tags={}",
                    result.id, result.note_type, result.title, tags_csv
                );
                if let Some(ctx) = &result.match_context {
                    println!("S {} {}", result.id, ctx);
                }
            }
        }
    }

    Ok(())
}
