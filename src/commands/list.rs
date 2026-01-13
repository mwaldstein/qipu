//! `qipu list` command - list notes
//!
//! Per spec (specs/cli-interface.md):
//! - `--tag` filter
//! - `--type` filter
//! - `--since` filter
//! - Deterministic ordering (by created, then id)

use chrono::{DateTime, Utc};

use crate::cli::{Cli, OutputFormat};
use crate::lib::error::Result;
use crate::lib::note::NoteType;
use crate::lib::store::Store;

/// Execute the list command
pub fn execute(
    cli: &Cli,
    store: &Store,
    tag: Option<&str>,
    note_type: Option<NoteType>,
    since: Option<DateTime<Utc>>,
) -> Result<()> {
    let mut notes = store.list_notes()?;

    // Apply filters
    if let Some(tag) = tag {
        notes.retain(|n| n.frontmatter.tags.iter().any(|t| t == tag));
    }

    if let Some(nt) = note_type {
        notes.retain(|n| n.note_type() == nt);
    }

    if let Some(since) = since {
        notes.retain(|n| {
            n.frontmatter
                .created
                .map_or(false, |created| created >= since)
        });
    }

    match cli.format {
        OutputFormat::Json => {
            let output: Vec<_> = notes
                .iter()
                .map(|n| {
                    serde_json::json!({
                        "id": n.id(),
                        "title": n.title(),
                        "type": n.note_type().to_string(),
                        "tags": n.frontmatter.tags,
                        "path": n.path.as_ref().map(|p| p.display().to_string()),
                        "created": n.frontmatter.created,
                        "updated": n.frontmatter.updated,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Human => {
            if notes.is_empty() {
                if !cli.quiet {
                    println!("No notes found");
                }
            } else {
                for note in &notes {
                    let type_indicator = match note.note_type() {
                        NoteType::Fleeting => "F",
                        NoteType::Literature => "L",
                        NoteType::Permanent => "P",
                        NoteType::Moc => "M",
                    };
                    println!("{} [{}] {}", note.id(), type_indicator, note.title());
                }
            }
        }
        OutputFormat::Records => {
            for note in &notes {
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
    }

    Ok(())
}
