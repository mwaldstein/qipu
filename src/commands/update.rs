//! `qipu update` command - update a note's metadata or content non-interactively
//!
//! Per spec (specs/cli-interface.md):
//! - `qipu update <id-or-path>` - non-interactive update for scripting/LLM mode
//! - Atomic update: updates file and database index in one operation
//! - Only provided flags are applied; omitted fields remain unchanged
//! - Reading from stdin replaces note body (preserving frontmatter)

use std::io::Read;

use tracing::debug;

use crate::cli::Cli;
use crate::commands::format::output_by_format_result;
use qipu_core::error::{QipuError, Result};
use qipu_core::store::Store;

/// Execute the update command
#[allow(clippy::too_many_arguments)]
pub fn execute(
    cli: &Cli,
    store: &Store,
    id_or_path: &str,
    title: Option<&str>,
    note_type: Option<qipu_core::note::NoteType>,
    tags: &[String],
    remove_tags: &[String],
    value: Option<u8>,
    source: Option<&str>,
    author: Option<&str>,
    generated_by: Option<&str>,
    prompt_hash: Option<&str>,
    verified: Option<bool>,
) -> Result<()> {
    // Load the note (either by ID or path)
    let mut note = store.load_note_by_id_or_path(id_or_path)?;

    let note_id = note.id().to_string();
    let note_path = note
        .path
        .as_ref()
        .ok_or_else(|| QipuError::Other("note has no path".to_string()))?
        .clone();

    // Check if stdin has data (try to peek, but only if not reading from terminal)
    use std::io::IsTerminal;
    let read_body_from_stdin = !std::io::stdin().is_terminal();

    let mut modified = false;

    // Update title if provided
    if let Some(new_title) = title {
        note.frontmatter.title = new_title.to_string();
        modified = true;

        // Rename the file if title changed
        if new_title != note.title() {
            let note_id_ref = qipu_core::id::NoteId::new_unchecked(note_id.clone());
            let new_file_name = qipu_core::id::filename(&note_id_ref, new_title);
            let new_file_path = note_path
                .parent()
                .ok_or_else(|| QipuError::Other("cannot determine parent directory".to_string()))?
                .join(&new_file_name);

            if new_file_path != note_path {
                std::fs::rename(&note_path, &new_file_path)?;
                note.path = Some(new_file_path.clone());
            }
        }
    }

    // Update type if provided
    if let Some(new_type) = note_type {
        note.frontmatter.note_type = Some(new_type.clone());
        modified = true;

        // Move file to appropriate directory if type changed
        if let Some(path) = &note.path {
            let is_moc = new_type.is_moc();
            let was_moc: bool = path
                .parent()
                .and_then(|p: &std::path::Path| p.file_name())
                .and_then(|n| n.to_str())
                .map(|n| n == "mocs")
                .unwrap_or(false);

            if is_moc != was_moc {
                let target_dir = if is_moc {
                    store.root().join(qipu_core::store::paths::MOCS_DIR)
                } else {
                    store.root().join(qipu_core::store::paths::NOTES_DIR)
                };

                let new_file_path = target_dir
                    .join(path.file_name().ok_or_else(|| {
                        QipuError::Other("cannot determine filename".to_string())
                    })?);

                std::fs::rename(path, &new_file_path)?;
                note.path = Some(new_file_path);
            }
        }
    }

    // Add tags if provided
    if !tags.is_empty() {
        for tag in tags {
            if !note.frontmatter.tags.contains(tag) {
                note.frontmatter.tags.push(tag.clone());
            }
        }
        modified = true;
    }

    // Remove tags if provided
    if !remove_tags.is_empty() {
        note.frontmatter.tags.retain(|t| !remove_tags.contains(t));
        modified = true;
    }

    // Update value if provided
    if let Some(new_value) = value {
        note.frontmatter.value = Some(new_value);
        modified = true;
    }

    // Update source if provided
    if let Some(new_source) = source {
        note.frontmatter.source = Some(new_source.to_string());
        modified = true;
    }

    // Update author if provided
    if let Some(new_author) = author {
        note.frontmatter.author = Some(new_author.to_string());
        modified = true;
    }

    // Update generated_by if provided
    if let Some(new_generated_by) = generated_by {
        note.frontmatter.generated_by = Some(new_generated_by.to_string());
        modified = true;
    }

    // Update prompt_hash if provided
    if let Some(new_prompt_hash) = prompt_hash {
        note.frontmatter.prompt_hash = Some(new_prompt_hash.to_string());
        modified = true;
    }

    // Update verified if provided
    if let Some(new_verified) = verified {
        note.frontmatter.verified = Some(new_verified);
        modified = true;
    }

    // Read body from stdin if data is available
    if read_body_from_stdin {
        let mut body = String::new();
        std::io::stdin()
            .read_to_string(&mut body)
            .map_err(|e| QipuError::Other(format!("failed to read from stdin: {}", e)))?;
        note.body = body;
        modified = true;
    }

    if !modified {
        // No changes to apply
        output_by_format_result!(cli.format,
            json => {
                let output = serde_json::json!({
                    "id": note_id,
                    "message": "no changes to apply"
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
                Ok::<(), QipuError>(())
            },
            human => {
                if !cli.quiet {
                    println!("No changes to apply");
                }
            },
            records => {
                println!("N id=\"{}\" status=unchanged", note_id);
            }
        )?;
        return Ok(());
    }

    // Save the note (this updates both file and database atomically)
    store.save_note(&mut note)?;

    if cli.verbose {
        debug!(note_id = note.id(), "save_note");
    }

    // Output the updated note info
    output_by_format_result!(cli.format,
        json => {
            let output = serde_json::json!({
                "id": note_id,
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
            Ok::<(), QipuError>(())
        },
        human => {
            println!("{}", note_id);
        },
        records => {
            use qipu_core::records::escape_quotes;

            println!(
                "H qipu=1 records=1 store={} mode=update",
                store.root().display()
            );

            let tags_csv = note.frontmatter.format_tags();
            println!(
                "N {} {} \"{}\" tags={}",
                note.id(),
                note.note_type(),
                escape_quotes(note.title()),
                tags_csv
            );
        }
    )
}
