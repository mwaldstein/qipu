//! `qipu edit` command - open a note in $EDITOR and update the index
//!
//! Per spec (specs/cli-interface.md):
//! - `qipu edit <id-or-path>` - open note in editor, update index on completion
//! - Atomic update: updates file and database index in one operation
//! - `--editor <cmd>`: override default editor
//! - Fails with usage error if no editor is configured/detected

use std::path::PathBuf;
use std::process::Command;

use tracing::debug;

use crate::cli::Cli;
use crate::commands::format::{dispatch_format, FormatDispatcher};
use crate::commands::helpers::resolve_editor;
use qipu_core::error::{QipuError, Result};
use qipu_core::store::Store;

struct EditFormatter<'a> {
    note: &'a qipu_core::note::Note,
    note_path: &'a PathBuf,
}

impl<'a> FormatDispatcher for EditFormatter<'a> {
    fn output_json(&self) -> Result<()> {
        let output = serde_json::json!({
            "id": self.note.id(),
            "title": self.note.title(),
            "type": self.note.note_type().to_string(),
            "path": self.note_path.to_string_lossy(),
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        Ok(())
    }

    fn output_human(&self) {
        println!("{}", self.note.id());
    }

    fn output_records(&self) {
        println!(
            "N id=\"{}\" path={}",
            self.note.id(),
            self.note_path.display()
        );
    }
}

/// Execute the edit command
pub fn execute(
    cli: &Cli,
    store: &Store,
    id_or_path: &str,
    editor_override: Option<&str>,
) -> Result<()> {
    // Load the note (either by ID or path)
    let mut note = store.load_note_by_id_or_path(id_or_path)?;

    // Ensure the note has a path
    let note_path = note
        .path
        .clone()
        .ok_or_else(|| QipuError::Other("note has no path".to_string()))?;

    // Get the editor to use
    let editor = resolve_editor(editor_override).ok_or_else(|| {
        QipuError::UsageError(
            "no editor configured. Set EDITOR or VISUAL environment variable, or use --editor"
                .to_string(),
        )
    })?;

    if cli.verbose {
        debug!(editor = %editor, path = %note_path.display(), "open_editor");
    }

    // Open the file in the editor
    let status = Command::new(&editor)
        .arg(&note_path)
        .status()
        .map_err(|e| QipuError::Other(format!("failed to open editor '{}': {}", editor, e)))?;

    if !status.success() {
        return Err(QipuError::Other(format!(
            "editor '{}' exited with non-zero status: {:?}",
            editor, status
        )));
    }

    // Reload the note from disk to get the edited content
    let updated_content = std::fs::read_to_string(&note_path)?;
    note = qipu_core::note::Note::parse(&updated_content, Some(note_path.clone()))?;

    // Save the note (this updates both file and database atomically)
    store.save_note(&mut note)?;

    if cli.verbose {
        debug!(note_id = note.id(), elapsed = ?std::time::Instant::now(), "save_note");
    }

    dispatch_format(
        cli,
        &EditFormatter {
            note: &note,
            note_path: &note_path,
        },
    )?;

    Ok(())
}
