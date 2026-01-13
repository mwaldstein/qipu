//! `qipu prime` command - session-start primer for LLM agents
//!
//! Per spec (specs/llm-context.md):
//! - `qipu prime` outputs a short, bounded primer suitable for automatic injection
//!   at the start of an agent session.
//! - Requirements: deterministic ordering, stable formatting, bounded size (~1-2k tokens)
//! - Contents: qipu explanation, command reference, store location, key MOCs, recent notes

use chrono::Utc;

use crate::cli::{Cli, OutputFormat};
use crate::lib::error::Result;
use crate::lib::note::NoteType;
use crate::lib::store::Store;

/// Maximum number of MOCs to include in the primer
const MAX_MOCS: usize = 5;

/// Maximum number of recent notes to include in the primer
const MAX_RECENT_NOTES: usize = 5;

/// Execute the prime command
pub fn execute(cli: &Cli, store: &Store) -> Result<()> {
    // Gather data for the primer
    let notes = store.list_notes()?;

    // Separate MOCs from regular notes
    let mut mocs: Vec<_> = notes
        .iter()
        .filter(|n| n.note_type() == NoteType::Moc)
        .collect();

    // Sort MOCs by updated (most recent first), then by id for stability
    mocs.sort_by(
        |a, b| match (&b.frontmatter.updated, &a.frontmatter.updated) {
            (Some(b_updated), Some(a_updated)) => b_updated.cmp(a_updated),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.id().cmp(b.id()),
        },
    );

    // Get top MOCs
    let top_mocs: Vec<_> = mocs.into_iter().take(MAX_MOCS).collect();

    // Get recent non-MOC notes sorted by updated/created
    let mut recent_notes: Vec<_> = notes
        .iter()
        .filter(|n| n.note_type() != NoteType::Moc)
        .collect();

    recent_notes.sort_by(|a, b| {
        let a_time = a
            .frontmatter
            .updated
            .as_ref()
            .or(a.frontmatter.created.as_ref());
        let b_time = b
            .frontmatter
            .updated
            .as_ref()
            .or(b.frontmatter.created.as_ref());
        match (b_time, a_time) {
            (Some(b_t), Some(a_t)) => b_t.cmp(a_t),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.id().cmp(b.id()),
        }
    });

    let recent_notes: Vec<_> = recent_notes.into_iter().take(MAX_RECENT_NOTES).collect();

    // Get store path for display
    let store_path = store.root().display().to_string();

    match cli.format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "generated_at": Utc::now().to_rfc3339(),
                "store": store_path,
                "primer": {
                    "description": "Qipu is a Zettelkasten-inspired knowledge management system for capturing research notes and navigating knowledge via links, tags, and Maps of Content (MOCs).",
                    "commands": [
                        {"name": "qipu list", "description": "List notes"},
                        {"name": "qipu search <query>", "description": "Search notes by title and body"},
                        {"name": "qipu show <id>", "description": "Display a note"},
                        {"name": "qipu create <title>", "description": "Create a new note"},
                        {"name": "qipu capture", "description": "Create note from stdin"},
                        {"name": "qipu link tree <id>", "description": "Show traversal tree from a note"},
                        {"name": "qipu link path <from> <to>", "description": "Find path between notes"},
                        {"name": "qipu context", "description": "Build context bundle for LLM"},
                    ],
                },
                "mocs": top_mocs.iter().map(|n| {
                    serde_json::json!({
                        "id": n.id(),
                        "title": n.title(),
                        "tags": n.frontmatter.tags,
                        "path": n.path.as_ref().map(|p| p.display().to_string()),
                    })
                }).collect::<Vec<_>>(),
                "recent_notes": recent_notes.iter().map(|n| {
                    serde_json::json!({
                        "id": n.id(),
                        "title": n.title(),
                        "type": n.note_type().to_string(),
                        "tags": n.frontmatter.tags,
                    })
                }).collect::<Vec<_>>(),
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Human => {
            output_human_primer(&store_path, &top_mocs, &recent_notes);
        }
        OutputFormat::Records => {
            output_records_primer(&store_path, &top_mocs, &recent_notes);
        }
    }

    Ok(())
}

/// Output primer in human-readable format
fn output_human_primer(
    store_path: &str,
    mocs: &[&crate::lib::note::Note],
    recent_notes: &[&crate::lib::note::Note],
) {
    println!("# Qipu Knowledge Store Primer");
    println!();
    println!("Store: {}", store_path);
    println!();
    println!("## About Qipu");
    println!();
    println!("Qipu is a Zettelkasten-inspired knowledge management system for capturing");
    println!("research notes and navigating knowledge via links, tags, and Maps of Content.");
    println!();
    println!("Note types: fleeting (quick capture), literature (from sources),");
    println!("permanent (distilled insights), moc (index/map notes).");
    println!();
    println!("## Quick Reference");
    println!();
    println!("  qipu list              List notes");
    println!("  qipu search <query>    Search notes by title and body");
    println!("  qipu show <id>         Display a note");
    println!("  qipu create <title>    Create a new note");
    println!("  qipu capture           Create note from stdin");
    println!("  qipu link tree <id>    Show traversal tree from a note");
    println!("  qipu link path A B     Find path between notes");
    println!("  qipu context           Build context bundle for LLM");
    println!();

    if !mocs.is_empty() {
        println!("## Key Maps of Content");
        println!();
        for moc in mocs {
            let tags = if moc.frontmatter.tags.is_empty() {
                String::new()
            } else {
                format!(" [{}]", moc.frontmatter.tags.join(", "))
            };
            println!("  {} - {}{}", moc.id(), moc.title(), tags);
        }
        println!();
    }

    if !recent_notes.is_empty() {
        println!("## Recently Updated Notes");
        println!();
        for note in recent_notes {
            let type_char = match note.note_type() {
                NoteType::Fleeting => 'F',
                NoteType::Literature => 'L',
                NoteType::Permanent => 'P',
                NoteType::Moc => 'M',
            };
            println!("  {} [{}] {}", note.id(), type_char, note.title());
        }
        println!();
    }

    println!("Use `qipu context --note <id>` to fetch full note content.");
}

/// Output primer in records format
fn output_records_primer(
    store_path: &str,
    mocs: &[&crate::lib::note::Note],
    recent_notes: &[&crate::lib::note::Note],
) {
    // Header line
    let mocs_count = mocs.len();
    let notes_count = recent_notes.len();
    println!(
        "H qipu=1 records=1 mode=prime store={} mocs={} recent={}",
        store_path, mocs_count, notes_count
    );

    // System description as a special record
    println!("D Qipu is a Zettelkasten-inspired knowledge management system for capturing research notes and navigating knowledge via links, tags, and Maps of Content.");

    // Commands reference
    println!("C list \"List notes\"");
    println!("C search \"Search notes by title and body\"");
    println!("C show \"Display a note\"");
    println!("C create \"Create a new note\"");
    println!("C capture \"Create note from stdin\"");
    println!("C link.tree \"Show traversal tree from a note\"");
    println!("C link.path \"Find path between notes\"");
    println!("C context \"Build context bundle for LLM\"");

    // MOC records
    for moc in mocs {
        let tags_csv = moc.frontmatter.tags.join(",");
        println!(
            "M {} \"{}\" tags={}",
            moc.id(),
            moc.title(),
            if tags_csv.is_empty() { "-" } else { &tags_csv }
        );
    }

    // Recent note records
    for note in recent_notes {
        let tags_csv = note.frontmatter.tags.join(",");
        println!(
            "N {} {} \"{}\" tags={}",
            note.id(),
            note.note_type(),
            note.title(),
            if tags_csv.is_empty() { "-" } else { &tags_csv }
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_constants() {
        // Ensure we have reasonable defaults
        assert!(MAX_MOCS > 0 && MAX_MOCS <= 10);
        assert!(MAX_RECENT_NOTES > 0 && MAX_RECENT_NOTES <= 10);
    }
}
