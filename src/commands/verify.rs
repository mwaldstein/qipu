//! `qipu verify` command - toggle verification status of a note

use std::fs;
use std::path::Path;

use crate::cli::Cli;
use crate::commands::format::output_by_format_result;
use qipu_core::error::Result;
use qipu_core::note::Note;
use qipu_core::store::Store;

/// Execute the verify command
pub fn execute(cli: &Cli, store: &Store, id_or_path: &str, status: Option<bool>) -> Result<()> {
    // Try to interpret as path first
    let mut note = if Path::new(id_or_path).exists() {
        let content = fs::read_to_string(id_or_path)?;
        Note::parse(&content, Some(id_or_path.into()))?
    } else {
        // Treat as ID
        store.get_note(id_or_path)?
    };

    let old_status = note.frontmatter.verified.unwrap_or(false);
    let new_status = status.unwrap_or(!old_status);

    note.frontmatter.verified = Some(new_status);

    // Save the note
    store.save_note(&mut note)?;

    output_by_format_result!(cli.format,
        json => {
            println!(
                "{}",
                serde_json::json!({
                    "id": note.id(),
                    "verified": new_status,
                    "previous": old_status,
                })
            );
            Ok::<(), qipu_core::error::QipuError>(())
        },
        human => {
            if !cli.quiet {
                println!(
                    "Note {} verified: {} (was: {})",
                    note.id(),
                    new_status,
                    old_status
                );
            }
        },
        records => {
            println!(
                "H qipu=1 records=1 store={} mode=verify id={} verified={}",
                store.root().display(),
                note.id(),
                new_status
            );
        }
    )?;

    Ok(())
}
