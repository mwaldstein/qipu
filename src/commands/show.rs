//! `qipu show` command - display a note
//!
//! Per spec (specs/cli-interface.md):
//! - `qipu show <id-or-path>` - print note to stdout

use std::fs;
use std::path::Path;

use crate::cli::{Cli, OutputFormat};
use crate::lib::error::Result;
use crate::lib::note::Note;
use crate::lib::store::Store;

/// Execute the show command
pub fn execute(cli: &Cli, store: &Store, id_or_path: &str) -> Result<()> {
    // Try to interpret as path first
    let note = if Path::new(id_or_path).exists() {
        let content = fs::read_to_string(id_or_path)?;
        Note::parse(&content, Some(id_or_path.into()))?
    } else {
        // Treat as ID
        store.get_note(id_or_path)?
    };

    match cli.format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "id": note.id(),
                "title": note.title(),
                "type": note.note_type().to_string(),
                "tags": note.frontmatter.tags,
                "path": note.path.as_ref().map(|p| p.display().to_string()),
                "created": note.frontmatter.created,
                "updated": note.frontmatter.updated,
                "sources": note.frontmatter.sources,
                "links": note.frontmatter.links,
                "body": note.body,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Human => {
            // Print the raw markdown content
            let content = note.to_markdown()?;
            print!("{}", content);
        }
        OutputFormat::Records => {
            // Note metadata line
            let tags_csv = note.frontmatter.tags.join(",");
            println!(
                "N {} {} \"{}\" tags={}",
                note.id(),
                note.note_type(),
                note.title(),
                tags_csv
            );

            // Summary line
            let summary = note.summary();
            if !summary.is_empty() {
                println!("S {} {}", note.id(), summary);
            }

            // Body lines
            println!("B {}", note.id());
            for line in note.body.lines() {
                println!("{}", line);
            }
        }
    }

    Ok(())
}
