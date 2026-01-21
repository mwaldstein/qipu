//! `qipu index` command - build/refresh derived indexes
//!
//! Per spec (specs/cli-interface.md, specs/indexing-search.md):
//! - `qipu index` - build/refresh indexes
//! - `qipu index --rebuild` - drop and regenerate
//!
#![allow(clippy::if_same_then_else)]

use crate::cli::{Cli, OutputFormat};
use crate::lib::error::Result;
use crate::lib::index::links;
use crate::lib::store::Store;

/// Execute the index command
pub fn execute(cli: &Cli, store: &Store, rebuild: bool, rewrite_wiki_links: bool) -> Result<()> {
    let mut notes = store.list_notes()?;

    if rewrite_wiki_links {
        let mut rewritten_count = 0;
        for note in &mut notes {
            if links::rewrite_wiki_links(note)? {
                store.save_note(note)?;
                rewritten_count += 1;
            }
        }
        if !cli.quiet && rewritten_count > 0 {
            eprintln!("Rewrote wiki-links in {} notes", rewritten_count);
        }
    }

    if rebuild {
        store.db().rebuild(store.root())?;
    } else {
        store.db().incremental_repair(store.root())?;
    }

    let notes_count = notes.len();

    match cli.format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "status": "ok",
                "notes_indexed": notes_count,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Records => {
            let store_path = store.root().display();

            println!(
                "H qipu=1 records=1 store={} mode=index notes={}",
                store_path, notes_count
            );
        }
        OutputFormat::Human => {
            if !cli.quiet {
                println!("Indexed {} notes", notes_count);
            }
        }
    }

    Ok(())
}
