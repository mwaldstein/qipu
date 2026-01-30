//! `qipu verify` command - toggle verification status of a note

use crate::cli::Cli;
use crate::commands::format::output_by_format_result;
use qipu_core::error::Result;
use qipu_core::store::Store;

/// Execute the verify command
pub fn execute(cli: &Cli, store: &Store, id_or_path: &str, status: Option<bool>) -> Result<()> {
    // Load note by ID or path
    let mut note = store.load_note_by_id_or_path(id_or_path)?;

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
