//! `qipu prime` command - session-start primer for LLM agents
//!
//! Per spec (specs/llm-context.md):
//! - `qipu prime` outputs a short, bounded primer suitable for automatic injection
//!   at the start of an agent session.
//! - Requirements: deterministic ordering, stable formatting, bounded size (~1-2k tokens)
//! - Contents: qipu explanation, command reference, store location, key MOCs, recent notes

use crate::cli::{Cli, OutputFormat};
use crate::commands::context::path_relative_to_cwd;
use crate::lib::error::Result;
use crate::lib::note::NoteType;
use crate::lib::records::escape_quotes;
use crate::lib::store::Store;

// TARGET_MIN_CHARS and TARGET_MAX_CHARS define acceptable output range per spec (~4-8k chars).
// We target midpoint (6k) to balance context density with token efficiency.
// When notes are scarce, output will be shorter; when abundant, we include as many as possible up to target.
// This ensures primers are consistently useful regardless of store size.
const TARGET_MIN_CHARS: usize = 4000;
const TARGET_MAX_CHARS: usize = 8000;

/// Execute the prime command
pub fn execute(cli: &Cli, store: &Store) -> Result<()> {
    // Gather data for the primer
    let notes = store.list_notes()?;

    // Separate MOCs from regular notes
    let mut mocs: Vec<_> = notes.iter().filter(|n| n.note_type().is_moc()).collect();

    // Sort MOCs by updated (most recent first), then by id for stability
    mocs.sort_by(
        |a, b| match (&b.frontmatter.updated, &a.frontmatter.updated) {
            (Some(b_updated), Some(a_updated)) => b_updated.cmp(a_updated),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.id().cmp(b.id()),
        },
    );

    let top_mocs: Vec<_> = mocs.into_iter().collect();

    let mut recent_notes: Vec<_> = notes.iter().filter(|n| !n.note_type().is_moc()).collect();

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

    let recent_notes: Vec<_> = recent_notes.into_iter().collect();

    let store_path = path_relative_to_cwd(store.root());

    let selected_mocs =
        select_notes_within_budget(&top_mocs, &recent_notes, &store_path, cli.format);
    let selected_recent =
        select_recent_within_budget(&recent_notes, &selected_mocs, &store_path, cli.format);

    match cli.format {
        OutputFormat::Json => {
            let output = serde_json::json!({
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
                    "session_protocol": {
                        "why": "Knowledge not committed is knowledge lost. The graph only grows if you save your work.",
                        "steps": [
                            {"number": 1, "action": "Capture any new insights", "command": "qipu capture --title \"...\""},
                            {"number": 2, "action": "Link new notes to existing knowledge", "command": "qipu link add <new> <existing> --type <type>"},
                            {"number": 3, "action": "Commit changes", "command": "git add .qipu && git commit -m \"knowledge: ...\""}
                        ]
                    },
                },
                "mocs": selected_mocs.iter().map(|n: &&crate::lib::note::Note| {
                    serde_json::json!({
                        "id": n.id(),
                        "title": n.title(),
                        "tags": n.frontmatter.tags,

                    })
                }).collect::<Vec<_>>(),
                "recent_notes": selected_recent.iter().map(|n: &&crate::lib::note::Note| {
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
            output_human_primer(&store_path, &selected_mocs, &selected_recent);
        }
        OutputFormat::Records => {
            output_records_primer(&store_path, &selected_mocs, &selected_recent);
        }
    }

    Ok(())
}

fn estimate_char_count(notes: &[&crate::lib::note::Note], format: OutputFormat) -> usize {
    match format {
        OutputFormat::Json => notes
            .iter()
            .map(|n| {
                let id = n.id();
                let title = n.title();
                let tags = serde_json::to_string(&n.frontmatter.tags).unwrap_or_default();
                id.len() + title.len() + tags.len() + 50
            })
            .sum(),
        OutputFormat::Human => notes
            .iter()
            .map(|n| n.id().len() + n.title().len() + n.frontmatter.tags.join(", ").len() + 10)
            .sum(),
        OutputFormat::Records => notes
            .iter()
            .map(|n| n.id().len() + n.title().len() + n.frontmatter.tags.join(",").len() + 20)
            .sum(),
    }
}

fn select_notes_within_budget<'a>(
    mocs: &'a [&'a crate::lib::note::Note],
    _recent_notes: &[&crate::lib::note::Note],
    store_path: &str,
    format: OutputFormat,
) -> Vec<&'a crate::lib::note::Note> {
    let base_chars = estimate_base_char_count(store_path, format);
    let target = (TARGET_MIN_CHARS + TARGET_MAX_CHARS) / 2;
    let remaining = target.saturating_sub(base_chars);

    let mut selected = Vec::new();
    let mut current_count = 0;

    for moc in mocs {
        let moc_chars = estimate_single_note_char_count(moc, format, true);
        if current_count + moc_chars <= remaining {
            selected.push(*moc);
            current_count += moc_chars;
        } else {
            break;
        }
    }

    selected
}

fn select_recent_within_budget<'a>(
    recent_notes: &'a [&'a crate::lib::note::Note],
    selected_mocs: &[&crate::lib::note::Note],
    store_path: &str,
    format: OutputFormat,
) -> Vec<&'a crate::lib::note::Note> {
    let base_chars = estimate_base_char_count(store_path, format);
    let moc_chars = estimate_char_count(selected_mocs, format);
    let target = (TARGET_MIN_CHARS + TARGET_MAX_CHARS) / 2;
    let remaining = target.saturating_sub(base_chars + moc_chars);

    let mut selected = Vec::new();
    let mut current_count = 0;

    for note in recent_notes {
        let note_chars = estimate_single_note_char_count(note, format, false);
        if current_count + note_chars <= remaining {
            selected.push(*note);
            current_count += note_chars;
        } else {
            break;
        }
    }

    selected
}

fn estimate_base_char_count(store_path: &str, format: OutputFormat) -> usize {
    match format {
        OutputFormat::Json => {
            serde_json::json!({
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
                    "session_protocol": {
                        "why": "Knowledge not committed is knowledge lost. The graph only grows if you save your work.",
                        "steps": [
                            {"number": 1, "action": "Capture any new insights", "command": "qipu capture --title \"...\""},
                            {"number": 2, "action": "Link new notes to existing knowledge", "command": "qipu link add <new> <existing> --type <type>"},
                            {"number": 3, "action": "Commit changes", "command": "git add .qipu && git commit -m \"knowledge: ...\""}
                        ]
                    },
                },
                "mocs": [],
                "recent_notes": [],
            }).to_string().len()
        }
        OutputFormat::Human => {
            "# Qipu Knowledge Store Primer\n\nStore: \n\n## About Qipu\n\nQipu is a Zettelkasten-inspired knowledge management system for capturing research notes and navigating knowledge via links, tags, and Maps of Content.\n\nNote types: fleeting (quick capture), literature (from sources), permanent (distilled insights), moc (index/map notes).\n\n## Quick Reference\n\n  qipu list              List notes\n  qipu search <query>    Search notes by title and body\n  qipu show <id>         Display a note\n  qipu create <title>    Create a new note\n  qipu capture           Create note from stdin\n  qipu link tree <id>    Show traversal tree from a note\n  qipu link path A B     Find path between notes\n  qipu context           Build context bundle for LLM\n\n## Session Protocol\n\n**Before ending session:**\n1. Capture any new insights: `qipu capture --title \"...\"`\n2. Link new notes to existing knowledge: `qipu link add <new> <existing> --type <type>`\n3. Commit changes: `git add .qipu && git commit -m \"knowledge: ...\"`\n\n**Why this matters:** Knowledge not committed is knowledge lost. The graph only grows if you save your work.\n\n".len() + store_path.len()
        }
        OutputFormat::Records => {
            "H qipu=1 records=1 store= mode=prime mocs=0 recent=0 truncated=false\nD Qipu is a Zettelkasten-inspired knowledge management system for capturing research notes and navigating knowledge via links, tags, and Maps of Content.\nC list \"List notes\"\nC search \"Search notes by title and body\"\nC show \"Display a note\"\nC create \"Create a new note\"\nC capture \"Create note from stdin\"\nC link.tree \"Show traversal tree from a note\"\nC link.path \"Find path between notes\"\nC context \"Build context bundle for LLM\"\nS 1 \"Capture any new insights\" \"qipu capture --title \\\"...\\\"\"\nS 2 \"Link new notes to existing knowledge\" \"qipu link add <new> <existing> --type <type>\"\nS 3 \"Commit changes\" \"git add .qipu && git commit -m \\\"knowledge: ...\\\"\"\nW Knowledge not committed is knowledge lost. The graph only grows if you save your work.\n".len() + store_path.len()
        }
    }
}

fn estimate_single_note_char_count(
    note: &crate::lib::note::Note,
    format: OutputFormat,
    is_moc: bool,
) -> usize {
    match format {
        OutputFormat::Json => {
            if is_moc {
                let tags = serde_json::to_string(&note.frontmatter.tags).unwrap_or_default();
                note.id().len() + note.title().len() + tags.len() + 50
            } else {
                let tags = serde_json::to_string(&note.frontmatter.tags).unwrap_or_default();
                note.id().len()
                    + note.title().len()
                    + note.note_type().to_string().len()
                    + tags.len()
                    + 50
            }
        }
        OutputFormat::Human => {
            if is_moc {
                note.id().len() + note.title().len() + note.frontmatter.tags.join(", ").len() + 15
            } else {
                note.id().len() + note.title().len() + 5
            }
        }
        OutputFormat::Records => {
            if is_moc {
                note.id().len() + note.title().len() + note.frontmatter.tags.join(",").len() + 15
            } else {
                note.id().len()
                    + note.title().len()
                    + note.note_type().to_string().len()
                    + note.frontmatter.tags.join(",").len()
                    + 20
            }
        }
    }
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
            let type_char = match note.note_type().as_str() {
                NoteType::FLEETING => 'F',
                NoteType::LITERATURE => 'L',
                NoteType::PERMANENT => 'P',
                NoteType::MOC => 'M',
                _ => 'F',
            };
            println!("  {} [{}] {}", note.id(), type_char, note.title());
        }
        println!();
    }

    println!("## Session Protocol");
    println!();
    println!("**Before ending session:**");
    println!("1. Capture any new insights: `qipu capture --title \"...\"`");
    println!(
        "2. Link new notes to existing knowledge: `qipu link add <new> <existing> --type <type>`"
    );
    println!("3. Commit changes: `git add .qipu && git commit -m \"knowledge: ...\"`");
    println!();
    println!("**Why this matters:** Knowledge not committed is knowledge lost. The graph only grows if you save your work.");
    println!();

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
        "H qipu=1 records=1 store={} mode=prime mocs={} recent={} truncated=false",
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

    // Session protocol records
    println!("S 1 \"Capture any new insights\" \"qipu capture --title \\\"...\\\"\"");
    println!("S 2 \"Link new notes to existing knowledge\" \"qipu link add <new> <existing> --type <type>\"");
    println!("S 3 \"Commit changes\" \"git add .qipu && git commit -m \\\"knowledge: ...\\\"\"");
    println!(
        "W Knowledge not committed is knowledge lost. The graph only grows if you save your work."
    );

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
            escape_quotes(note.title()),
            if tags_csv.is_empty() { "-" } else { &tags_csv }
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn test_budget_constants() {
        assert!(TARGET_MIN_CHARS > 1000 && TARGET_MIN_CHARS <= 10000);
        assert!(TARGET_MAX_CHARS > TARGET_MIN_CHARS && TARGET_MAX_CHARS <= 20000);
    }
}
