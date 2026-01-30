//! `qipu store` commands - manage qipu store

use crate::cli::Cli;
use crate::commands::context::path_relative_to_cwd;
use crate::commands::format::output_by_format_result;
use qipu_core::error::Result;
use qipu_core::store::Store;

/// Execute the stats command
pub fn execute_stats(cli: &Cli, store: &Store) -> Result<()> {
    let db = store.db();

    let note_count = db.get_note_count().unwrap_or(0);
    let tag_count = db.get_tag_count().unwrap_or(0);
    let edge_count = db.get_edge_count().unwrap_or(0);
    let unresolved_count = db.get_unresolved_count().unwrap_or(0);
    let schema_version = db.get_schema_version().unwrap_or(6);

    let db_path = store.root().join("qipu.db");
    let db_size = std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);

    let store_path = path_relative_to_cwd(store.root());

    output_by_format_result!(cli.format,
        json => {
            let output = serde_json::json!({
                "store": store_path,
                "database": {
                    "path": format!("{}/qipu.db", store_path),
                    "size_bytes": db_size,
                    "schema_version": schema_version,
                },
                "notes": note_count,
                "tags": tag_count,
                "links": edge_count,
                "unresolved_links": unresolved_count,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
            Ok::<(), qipu_core::error::QipuError>(())
        },
        human => {
            println!("Store: {}", store_path);
            println!();
            println!("Database:");
            println!("  Size: {} bytes", db_size);
            println!("  Schema version: {}", schema_version);
            println!();
            println!("Contents:");
            println!("  Notes: {}", note_count);
            println!("  Tags: {}", tag_count);
            println!("  Links: {}", edge_count);
            println!("  Unresolved links: {}", unresolved_count);
        },
        records => {
            println!("H qipu=1 records=1 store={} mode=stats", store_path);
            println!(
                "D database.path=\"{}/qipu.db\" database.size={} database.schema_version={}",
                store_path, db_size, schema_version
            );
            println!(
                "C notes={} tags={} links={} unresolved={}",
                note_count, tag_count, edge_count, unresolved_count
            );
        }
    )?;

    Ok(())
}
