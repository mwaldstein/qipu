//! `qipu edit` command - open a note in $EDITOR and update the index
//!
//! Per spec (specs/cli-interface.md):
//! - `qipu edit <id-or-path>` - open note in editor, update index on completion
//! - Atomic update: updates file and database index in one operation
//! - `--editor <cmd>`: override default editor
//! - Fails with usage error if no editor is configured/detected

use std::path::Path;
use std::process::Command;

use tracing::debug;

use crate::cli::{Cli, OutputFormat};
use qipu_core::error::{QipuError, Result};
use qipu_core::store::Store;

/// Execute the edit command
pub fn execute(
    cli: &Cli,
    store: &Store,
    id_or_path: &str,
    editor_override: Option<&str>,
) -> Result<()> {
    // Load the note (either by ID or path)
    let mut note = if Path::new(id_or_path).exists() {
        let content = std::fs::read_to_string(id_or_path)?;
        qipu_core::note::Note::parse(&content, Some(id_or_path.into()))?
    } else {
        store.get_note(id_or_path)?
    };

    // Ensure the note has a path
    let note_path = note
        .path
        .clone()
        .ok_or_else(|| QipuError::Other("note has no path".to_string()))?;

    // Get the editor to use
    let editor = editor_override
        .map(String::from)
        .or_else(|| std::env::var("EDITOR").ok())
        .or_else(|| std::env::var("VISUAL").ok())
        .ok_or_else(|| {
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

    // Output the note ID/path
    match cli.format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "id": note.id(),
                "title": note.title(),
                "type": note.note_type().to_string(),
                "path": note_path.to_string_lossy(),
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Human => {
            println!("{}", note.id());
        }
        OutputFormat::Records => {
            println!("N id=\"{}\" path={}", note.id(), note_path.display());
        }
    }

    Ok(())
}
