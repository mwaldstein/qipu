//! `qipu create` command - create a new note
//!
//! Per spec (specs/cli-interface.md):
//! - `qipu create <title>` - create new note, print id/path
//! - `--type` flag (fleeting|literature|permanent|moc)
//! - `--tag` flag (repeatable)
//! - `--open` flag (launch $EDITOR)

use std::path::Path;
use std::process::Command;

use crate::cli::{Cli, OutputFormat};
use crate::lib::error::Result;
use crate::lib::note::NoteType;
use crate::lib::store::Store;

/// Execute the create command
pub fn execute(
    cli: &Cli,
    store: &Store,
    title: &str,
    note_type: Option<NoteType>,
    tags: &[String],
    open: bool,
) -> Result<()> {
    let note = store.create_note(title, note_type, tags)?;

    match cli.format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "id": note.id(),
                "title": note.title(),
                "type": note.note_type().to_string(),
                "path": note.path.as_ref().map(|p| p.display().to_string()),
                "tags": note.frontmatter.tags,
                "created": note.frontmatter.created,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Human => {
            println!("{}", note.id());
            if cli.verbose {
                if let Some(path) = &note.path {
                    println!("Created: {}", path.display());
                }
            }
        }
        OutputFormat::Records => {
            // Records format: N <id> <type> "<title>" tags=<csv>
            let tags_csv = note.frontmatter.tags.join(",");
            println!(
                "N {} {} \"{}\" tags={}",
                note.id(),
                note.note_type(),
                note.title(),
                tags_csv
            );
        }
    }

    // Open in editor if requested
    if open {
        if let Some(path) = &note.path {
            open_in_editor(path, store.config().editor.as_deref())?;
        }
    }

    Ok(())
}

/// Open a file in the user's editor
fn open_in_editor(path: &Path, editor_override: Option<&str>) -> Result<()> {
    let editor = editor_override
        .map(String::from)
        .or_else(|| std::env::var("EDITOR").ok())
        .or_else(|| std::env::var("VISUAL").ok())
        .unwrap_or_else(|| "vi".to_string());

    Command::new(&editor).arg(path).status()?;
    Ok(())
}
