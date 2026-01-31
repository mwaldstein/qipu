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

use crate::cli::Cli;
use crate::commands::format::{dispatch_format, FormatDispatcher};
use crate::commands::helpers::resolve_editor;
use crate::commands::provenance::{update_provenance_if_provided, ProvenanceUpdate};
use qipu_core::error::Result;
use qipu_core::note::NoteType;
use qipu_core::records::escape_quotes;
use qipu_core::store::Store;

struct CreateFormatter<'a> {
    note: &'a qipu_core::note::Note,
    store: &'a Store,
}

impl<'a> FormatDispatcher for CreateFormatter<'a> {
    fn output_json(&self) -> Result<()> {
        let output = serde_json::json!({
            "id": self.note.id(),
            "title": self.note.title(),
            "type": self.note.note_type().to_string(),

            "tags": self.note.frontmatter.tags,
            "created": self.note.frontmatter.created,
            "updated": self.note.frontmatter.updated,
            "source": self.note.frontmatter.source,
            "author": self.note.frontmatter.author,
            "generated_by": self.note.frontmatter.generated_by,
            "prompt_hash": self.note.frontmatter.prompt_hash,
            "verified": self.note.frontmatter.verified,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        Ok(())
    }

    fn output_human(&self) {
        println!("{}", self.note.id());
    }

    fn output_records(&self) {
        println!(
            "H qipu=1 records=1 store={} mode=create",
            self.store.root().display()
        );

        let tags_csv = self.note.frontmatter.format_tags();
        println!(
            "N {} {} \"{}\" tags={}",
            self.note.id(),
            self.note.note_type(),
            escape_quotes(self.note.title()),
            tags_csv
        );
    }
}

/// Execute the create command
#[allow(clippy::too_many_arguments)]
pub fn execute(
    cli: &Cli,
    store: &Store,
    title: &str,
    note_type: Option<&NoteType>,
    tags: &[String],
    open: bool,
    id: Option<&str>,
    source: Option<&str>,
    author: Option<&str>,
    generated_by: Option<&str>,
    prompt_hash: Option<&str>,
    verified: Option<bool>,
) -> Result<()> {
    let start = Instant::now();

    if cli.verbose {
        debug!(title, ?note_type, tags_count = tags.len(), "create_params");
    }

    // Validate note type against active ontology
    if let Some(nt) = note_type {
        store.config().validate_note_type(nt.as_str())?;
    }

    let mut note = store.create_note(title, note_type.cloned(), tags, id)?;

    if cli.verbose {
        debug!(note_id = note.id(), elapsed = ?start.elapsed(), "create_note");
    }

    let _ = update_provenance_if_provided(
        store,
        &mut note,
        ProvenanceUpdate {
            source,
            author,
            generated_by,
            prompt_hash,
            verified,
        },
        false,
    )?;

    if cli.verbose {
        debug!(note_id = note.id(), elapsed = ?start.elapsed(), "update_provenance");
    }

    dispatch_format(cli, &CreateFormatter { note: &note, store })?;

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
    let editor = resolve_editor(editor_override).unwrap_or_else(|| "vi".to_string());

    Command::new(&editor).arg(path).status()?;
    Ok(())
}
