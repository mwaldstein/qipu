//! Budgeting logic for prime command output

use crate::cli::OutputFormat;

// TARGET_MIN_CHARS and TARGET_MAX_CHARS define acceptable output range per spec (~4-8k chars).
// We target midpoint (6k) to balance context density with token efficiency.
// When notes are scarce, output will be shorter; when abundant, we include as many as possible up to target.
// This ensures primers are consistently useful regardless of store size.
pub const TARGET_MIN_CHARS: usize = 4000;
pub const TARGET_MAX_CHARS: usize = 8000;

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

pub fn select_notes_within_budget<'a>(
    mocs: &'a [&'a crate::lib::note::Note],
    _recent_notes: &[&crate::lib::note::Note],
    store_path: &str,
    format: OutputFormat,
    is_empty: bool,
) -> Vec<&'a crate::lib::note::Note> {
    let base_chars = estimate_base_char_count(store_path, format, is_empty);
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

pub fn select_recent_within_budget_compact<'a>(
    recent_notes: &'a [&'a crate::lib::note::Note],
    store_path: &str,
    format: OutputFormat,
    is_empty: bool,
) -> Vec<&'a crate::lib::note::Note> {
    let base_chars = estimate_base_char_count(store_path, format, is_empty);
    let target = (TARGET_MIN_CHARS + TARGET_MAX_CHARS) / 2;
    let remaining = target.saturating_sub(base_chars);

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

pub fn select_recent_within_budget<'a>(
    recent_notes: &'a [&'a crate::lib::note::Note],
    selected_mocs: &[&crate::lib::note::Note],
    store_path: &str,
    format: OutputFormat,
    is_empty: bool,
) -> Vec<&'a crate::lib::note::Note> {
    let base_chars = estimate_base_char_count(store_path, format, is_empty);
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

fn estimate_base_char_count(store_path: &str, format: OutputFormat, is_empty: bool) -> usize {
    match format {
        OutputFormat::Json => {
            let primer_description = if is_empty {
                "Welcome to qipu! Your knowledge store is empty. Start by capturing your first insights with `qipu capture` or `qipu create`. Qipu is a Zettelkasten-inspired knowledge management system for capturing research notes and navigating knowledge via links, tags, and Maps of Content (MOCs)."
            } else {
                "Qipu is a Zettelkasten-inspired knowledge management system for capturing research notes and navigating knowledge via links, tags, and Maps of Content (MOCs)."
            };
            serde_json::json!({
                "store": store_path,
                "primer": {
                    "description": primer_description,
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
            let getting_started = if is_empty {
                "## Getting Started\n\nYour knowledge store is empty. Start by capturing your first insights:\n\n  qipu capture --title \"...\"     Quick capture from stdin\n  qipu create <title>             Create a new note\n  qipu create <title> --type moc   Create a Map of Content\n\n"
            } else {
                ""
            };
            format!("# Qipu Knowledge Store Primer\n\nStore: {}\n\n## About Qipu\n\nQipu is a Zettelkasten-inspired knowledge management system for capturing research notes and navigating knowledge via links, tags, and Maps of Content.\n\n{}## Quick Reference\n\n  qipu list              List notes\n  qipu search <query>    Search notes by title and body\n  qipu show <id>         Display a note\n  qipu create <title>    Create a new note\n  qipu capture           Create note from stdin\n  qipu link tree <id>    Show traversal tree from a note\n  qipu link path A B     Find path between notes\n  qipu context           Build context bundle for LLM\n\n## Session Protocol\n\n**Before ending session:**\n1. Capture any new insights: `qipu capture --title \"...\"`\n2. Link new notes to existing knowledge: `qipu link add <new> <existing> --type <type>`\n3. Commit changes: `git add .qipu && git commit -m \"knowledge: ...\"`\n\n**Why this matters:** Knowledge not committed is knowledge lost. The graph only grows if you save your work.\n\n", store_path, getting_started).len()
        }
        OutputFormat::Records => {
            let store_status = if is_empty { "empty" } else { "populated" };
            format!("H qipu=1 records=1 store={} mode=prime mocs=0 recent=0 status={} truncated=false\nD Qipu is a Zettelkasten-inspired knowledge management system for capturing research notes and navigating knowledge via links, tags, and Maps of Content.\nC list \"List notes\"\nC search \"Search notes by title and body\"\nC show \"Display a note\"\nC create \"Create a new note\"\nC capture \"Create note from stdin\"\nC link.tree \"Show traversal tree from a note\"\nC link.path \"Find path between notes\"\nC context \"Build context bundle for LLM\"\nS 1 \"Capture any new insights\" \"qipu capture --title \\\"...\\\"\"\nS 2 \"Link new notes to existing knowledge\" \"qipu link add <new> <existing> --type <type>\"\nS 3 \"Commit changes\" \"git add .qipu && git commit -m \\\"knowledge: ...\\\"\"\nW Knowledge not committed is knowledge lost. The graph only grows if you save your work.\n", store_path, store_status).len()
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
