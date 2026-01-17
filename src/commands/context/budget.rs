use super::types::SelectedNote;
use crate::lib::note::Note;
use tiktoken_rs::get_bpe_from_model;

/// Apply character and token budget to notes
/// Returns (truncated, notes_to_output)
pub fn apply_budget<'a>(
    notes: &'a [SelectedNote<'a>],
    max_chars: Option<usize>,
    max_tokens: Option<usize>,
    model: &str,
    with_body: bool,
) -> (bool, Vec<&'a SelectedNote<'a>>) {
    if max_chars.is_none() && max_tokens.is_none() {
        return (false, notes.iter().collect());
    }

    let bpe = if max_tokens.is_some() {
        get_bpe_from_model(model).ok()
    } else {
        None
    };

    let mut result = Vec::new();
    let mut used_chars = 0;
    let mut used_tokens = 0;
    let mut truncated = false;

    // Conservative header estimate with buffer
    // Different formats have different header sizes, so we use a conservative estimate
    let header_estimate_chars = 250;
    let header_estimate_tokens = if let Some(ref bpe) = bpe {
        bpe.encode_with_special_tokens("# Qipu Context Bundle\nStore: .qipu/\n\n")
            .len()
            + 20
    } else {
        0
    };

    used_chars += header_estimate_chars;
    used_tokens += header_estimate_tokens;

    for note in notes {
        let note_size_chars = estimate_note_size(note.note, with_body);
        let note_size_tokens = if let Some(ref bpe) = bpe {
            estimate_note_tokens(note.note, with_body, bpe)
        } else {
            0
        };

        // Add safety buffer to ensure actual output doesn't exceed budget
        let note_size_chars_with_buffer = note_size_chars + (note_size_chars / 10);
        let note_size_tokens_with_buffer = note_size_tokens + (note_size_tokens / 10);

        let char_ok = max_chars
            .map(|limit| used_chars + note_size_chars_with_buffer <= limit)
            .unwrap_or(true);
        let token_ok = max_tokens
            .map(|limit| used_tokens + note_size_tokens_with_buffer <= limit)
            .unwrap_or(true);

        if char_ok && token_ok {
            result.push(note);
            used_chars += note_size_chars_with_buffer;
            used_tokens += note_size_tokens_with_buffer;
        } else {
            truncated = true;
            break;
        }
    }

    (truncated, result)
}

/// Estimate the output size of a note in tokens
pub fn estimate_note_tokens(note: &Note, with_body: bool, bpe: &tiktoken_rs::CoreBPE) -> usize {
    let mut text = String::new();

    // Rough approximation of the markdown output format
    text.push_str(&format!("## Note: {} ({})\n", note.title(), note.id()));
    if let Some(path) = &note.path {
        text.push_str(&format!("Path: {}\n", path.display()));
    }
    text.push_str(&format!("Type: {}\n", note.note_type()));
    if !note.frontmatter.tags.is_empty() {
        text.push_str(&format!("Tags: {}\n", note.frontmatter.tags.join(", ")));
    }

    if !note.frontmatter.sources.is_empty() {
        text.push_str("Sources:\n");
        for source in &note.frontmatter.sources {
            text.push_str(&format!("- {}\n", source.url));
        }
    }
    text.push_str("\n---\n");

    if with_body {
        text.push_str(&note.body);
    } else {
        text.push_str(&note.summary());
    }
    text.push_str("\n---\n");

    bpe.encode_with_special_tokens(&text).len()
}

/// Estimate the output size of a note
pub fn estimate_note_size(note: &Note, with_body: bool) -> usize {
    let mut size = 0;

    // Metadata size with realistic format overhead
    size += note.id().len() + 15; // "N qp-xxx type "
    size += note.title().len() + 20; // Title with quotes and labels
    size += note.note_type().to_string().len() + 15;

    // Tags
    if !note.frontmatter.tags.is_empty() {
        size += note.frontmatter.tags.join(",").len() + 20; // "tags=..." overhead
    } else {
        size += 10; // "tags=-"
    }

    // Path
    if let Some(path) = &note.path {
        size += path.display().to_string().len() + 20; // "Path: " or "path=" overhead
    } else {
        size += 10; // "path=-" or no path
    }

    // Sources - account for markdown/JSON/records formatting
    for source in &note.frontmatter.sources {
        size += source.url.len() + 50;
        if let Some(title) = &source.title {
            size += title.len() + 15;
        } else {
            size += source.url.len() + 15;
        }
        if let Some(accessed) = &source.accessed {
            size += accessed.len() + 20;
        } else {
            size += 10;
        }
        size += note.id().len() + 10;
    }

    // Body or summary
    if with_body {
        size += note.body.len();
        size += 30; // "B qp-xxx\n" + "B-END\n"
    } else {
        let summary = note.summary();
        size += summary.len();
        if !summary.is_empty() {
            size += note.id().len() + 5; // "S qp-xxx "
        }
    }

    size += 100;
    size
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::note::NoteFrontmatter;

    #[test]
    fn test_estimate_note_size() {
        let fm = NoteFrontmatter::new("qp-test".to_string(), "Test Note".to_string());
        let note = Note::new(fm, "This is the body content.");

        let size_with_body = estimate_note_size(&note, true);
        let size_without_body = estimate_note_size(&note, false);

        assert!(size_with_body > 0);
        assert!(size_without_body > 0);
        assert!(size_with_body >= size_without_body);
    }
}
