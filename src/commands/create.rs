//! `qipu create` command - create a new note
//!
//! Per spec (specs/cli-interface.md):
//! - `qipu create <title>` - create new note, print id/path
//! - `--type` flag (fleeting|literature|permanent|moc)
//! - `--tag` flag (repeatable)
//! - `--open` flag (launch $EDITOR)

use std::path::Path;
use std::process::Command;
use std::time::Instant;

use tracing::debug;

use crate::cli::{Cli, OutputFormat};
use crate::lib::error::Result;
use crate::lib::note::NoteType;
use crate::lib::records::escape_quotes;
use crate::lib::store::Store;

/// Execute the create command
#[allow(clippy::too_many_arguments)]
pub fn execute(
    cli: &Cli,
    store: &Store,
    title: &str,
    note_type: Option<NoteType>,
    tags: &[String],
    open: bool,
    id: Option<String>,
    source: Option<String>,
    author: Option<String>,
    generated_by: Option<String>,
    prompt_hash: Option<String>,
    verified: Option<bool>,
) -> Result<()> {
    let start = Instant::now();

    if cli.verbose {
        debug!(title, ?note_type, tags_count = tags.len(), "create_params");
    }

    let mut note = store.create_note(title, note_type, tags, id.as_deref())?;

    if cli.verbose {
        debug!(note_id = note.id(), elapsed = ?start.elapsed(), "create_note");
    }

    // Add provenance fields if provided
    if source.is_some()
        || author.is_some()
        || generated_by.is_some()
        || prompt_hash.is_some()
        || verified.is_some()
    {
        note.frontmatter.source = source;
        note.frontmatter.author = author;
        note.frontmatter.generated_by = generated_by.clone();
        note.frontmatter.prompt_hash = prompt_hash;

        // Per spec (specs/provenance.md): When an agent generates a note, set verified: false by default
        note.frontmatter.verified = if verified.is_some() {
            verified
        } else if generated_by.is_some() {
            Some(false)
        } else {
            None
        };

        // Save the updated note
        store.save_note(&mut note)?;

        if cli.verbose {
            debug!(note_id = note.id(), elapsed = ?start.elapsed(), "update_provenance");
        }
    }

    match cli.format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "id": note.id(),
                "title": note.title(),
                "type": note.note_type().to_string(),

                "tags": note.frontmatter.tags,
                "created": note.frontmatter.created,
                "updated": note.frontmatter.updated,
                "source": note.frontmatter.source,
                "author": note.frontmatter.author,
                "generated_by": note.frontmatter.generated_by,
                "prompt_hash": note.frontmatter.prompt_hash,
                "verified": note.frontmatter.verified,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Human => {
            println!("{}", note.id());
        }
        OutputFormat::Records => {
            // Header line per spec (specs/records-output.md)
            println!(
                "H qipu=1 records=1 store={} mode=create",
                store.root().display()
            );

            // Records format: N <id> <type> "<title>" tags=<csv>
            let tags_csv = if note.frontmatter.tags.is_empty() {
                "-".to_string()
            } else {
                note.frontmatter.tags.join(",")
            };
            println!(
                "N {} {} \"{}\" tags={}",
                note.id(),
                note.note_type(),
                escape_quotes(note.title()),
                tags_csv
            );
        }
    }

    // Open in editor if requested
    if open {
        if let Some(path) = &note.path {
            if cli.verbose {
                debug!(path = %path.display(), "open_editor");
            }
            open_in_editor(path, store.config().editor.as_deref())?;
        }
    }

    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "execute_command");
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
