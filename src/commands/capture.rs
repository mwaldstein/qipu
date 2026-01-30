//! `qipu capture` command - create a new note from stdin
//!
//! Per spec (specs/cli-interface.md):
//! - Creates a new note with content read from stdin
//! - `--title` flag (auto-generate from content if not provided)
//! - `--type` flag (default: fleeting per spec open question)
//! - `--tag` flag (repeatable)
//!
//! Example usage:
//! - `pbpaste | qipu capture --type fleeting --tag docs`
//! - `qipu capture --title "Thoughts on indexing" < notes.txt`

use std::io::{self, Read};
use std::time::Instant;

use tracing::debug;

use crate::cli::{Cli, OutputFormat};
use qipu_core::error::Result;
use qipu_core::note::NoteType;
use qipu_core::records::escape_quotes;
use qipu_core::store::Store;

/// Execute the capture command
#[allow(clippy::too_many_arguments)]
pub fn execute(
    cli: &Cli,
    store: &Store,
    title: Option<&str>,
    note_type: Option<NoteType>,
    tags: &[String],
    source: Option<String>,
    author: Option<String>,
    generated_by: Option<String>,
    prompt_hash: Option<String>,
    verified: Option<bool>,
    id: Option<&str>,
) -> Result<()> {
    let start = Instant::now();

    // Read content from stdin
    let mut content = String::new();
    io::stdin().read_to_string(&mut content)?;

    // Trim trailing whitespace but preserve internal formatting
    let content = content.trim_end();

    if cli.verbose {
        debug!(content_len = content.len(), "read_stdin");
    }

    // Generate title from content if not provided
    let title = match title {
        Some(t) => t.to_string(),
        None => generate_title_from_content(content),
    };

    if cli.verbose {
        debug!(title, ?note_type, tags_count = tags.len(), "capture_params");
    }

    // Default type is fleeting for captures (per spec open question)
    let note_type = note_type.or(Some(NoteType::from(NoteType::FLEETING)));

    // Validate note type against active ontology
    if let Some(ref nt) = note_type {
        store.config().validate_note_type(nt.as_str())?;
    }

    // Create note with the captured content
    let mut note = store.create_note_with_content(&title, note_type, tags, content, id)?;

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
        note.frontmatter.source = source.clone();

        // Per spec (specs/provenance.md): Web capture defaults
        // When capturing a webpage (source is provided):
        // - Set author to user's name (if manual) or "Qipu Clipper" (if automated)
        // Default to "Qipu Clipper" when source is provided but author is not
        note.frontmatter.author = if author.is_some() {
            author
        } else if source.is_some() {
            Some("Qipu Clipper".to_string())
        } else {
            None
        };

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
                "H qipu=1 records=1 store={} mode=capture",
                store.root().display()
            );

            // Records format: N <id> <type> "<title>" tags=<csv>
            let tags_csv = note.frontmatter.format_tags();
            println!(
                "N {} {} \"{}\" tags={}",
                note.id(),
                note.note_type(),
                escape_quotes(note.title()),
                tags_csv
            );
        }
    }

    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "execute_command");
    }

    Ok(())
}

/// Generate a title from the content
///
/// Uses the first non-empty line, truncated to reasonable length.
/// Falls back to "Untitled capture" if content is empty.
fn generate_title_from_content(content: &str) -> String {
    const MAX_TITLE_LENGTH: usize = 60;

    // Find first non-empty, non-heading line
    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines
        if line.is_empty() {
            continue;
        }

        // Skip markdown headings (we want content, not structure)
        if line.starts_with('#') {
            // But extract heading text if it's the only content
            let heading_text = line.trim_start_matches('#').trim();
            if !heading_text.is_empty() {
                return truncate_title(heading_text, MAX_TITLE_LENGTH);
            }
            continue;
        }

        // Skip HTML comments
        if line.starts_with("<!--") {
            continue;
        }

        // Use this line as the title
        return truncate_title(line, MAX_TITLE_LENGTH);
    }

    "Untitled capture".to_string()
}

/// Truncate a title to max length, breaking at word boundary if possible
fn truncate_title(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }

    // Try to break at a word boundary
    let truncated = &s[..max_len];
    if let Some(last_space) = truncated.rfind(' ') {
        if last_space > max_len / 2 {
            return format!("{}...", &s[..last_space]);
        }
    }

    format!("{}...", truncated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_title_simple() {
        let content = "This is a simple note\nWith multiple lines";
        assert_eq!(
            generate_title_from_content(content),
            "This is a simple note"
        );
    }

    #[test]
    fn test_generate_title_from_heading() {
        let content = "# My Heading\n\nSome content";
        assert_eq!(generate_title_from_content(content), "My Heading");
    }

    #[test]
    fn test_generate_title_skip_empty() {
        let content = "\n\n\nActual content";
        assert_eq!(generate_title_from_content(content), "Actual content");
    }

    #[test]
    fn test_generate_title_empty() {
        assert_eq!(generate_title_from_content(""), "Untitled capture");
        assert_eq!(generate_title_from_content("   \n\n  "), "Untitled capture");
    }

    #[test]
    fn test_generate_title_truncate() {
        let content = "This is a very long title that should be truncated because it exceeds the maximum allowed length for a note title";
        let title = generate_title_from_content(content);
        assert!(title.len() <= 63); // 60 + "..."
        assert!(title.ends_with("..."));
    }

    #[test]
    fn test_truncate_title_word_boundary() {
        let title = truncate_title("Hello world this is a test", 15);
        // Should break at "Hello world" (11 chars) + "..."
        assert_eq!(title, "Hello world...");
    }
}
