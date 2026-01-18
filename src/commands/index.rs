//! `qipu index` command - build/refresh derived indexes
//!
//! Per spec (specs/cli-interface.md, specs/indexing-search.md):
//! - `qipu index` - build/refresh indexes
//! - `qipu index --rebuild` - drop and regenerate
//!
use crate::cli::{Cli, OutputFormat};
use crate::lib::error::Result;
use crate::lib::store::Store;

/// Execute the index command
pub fn execute(cli: &Cli, store: &Store, rebuild: bool) -> Result<()> {
    if rebuild {
        store.db().rebuild(store.root())?;
    } else {
        store.db().rebuild(store.root())?;
    }

    let notes_count = store.list_notes()?.len();

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
