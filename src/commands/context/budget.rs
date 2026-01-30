use super::types::SelectedNote;
use qipu_core::note::Note;
use std::time::Instant;

/// Apply character budget to notes
/// Returns (truncated, notes_to_output, excluded_notes)
/// Note: This function now includes all notes to support per-note content truncation
/// instead of excluding entire notes. The excluded_notes vector will be empty.
pub fn apply_budget<'a>(
    notes: &'a [SelectedNote<'a>],
    max_chars: Option<usize>,
    with_body: bool,
) -> (bool, Vec<&'a SelectedNote<'a>>, Vec<&'a SelectedNote<'a>>) {
    let start = Instant::now();

    tracing::debug!(
        input_notes = notes.len(),
        max_chars,
        with_body,
        "apply_budget"
    );

    if max_chars.is_none() {
        return (false, notes.iter().collect(), Vec::new());
    }

    let mut truncated = false;

    // Conservative header estimate with buffer
    // Different formats have different header sizes, so we use a conservative estimate
    let header_estimate_chars = 250;

    let mut used_chars = header_estimate_chars;

    // Include all notes - per-note truncation will be handled by output formatters
    let result: Vec<&'a SelectedNote<'a>> = notes.iter().collect();

    // Calculate total size to determine if truncation is needed
    for note in notes {
        let note_size_chars = estimate_note_size(note.note, with_body);

        // Add safety buffer to ensure actual output doesn't exceed budget
        let note_size_chars_with_buffer = note_size_chars + (note_size_chars / 10);

        let char_ok = max_chars
            .map(|limit| used_chars + note_size_chars_with_buffer <= limit)
            .unwrap_or(true);

        if !char_ok {
            truncated = true;
        }

        used_chars += note_size_chars_with_buffer;
    }

    tracing::debug!(
        output_notes = result.len(),
        excluded_notes = 0,
        truncated,
        elapsed = ?start.elapsed(),
        "apply_budget_complete"
    );

    (truncated, result, Vec::new())
}

/// Estimate the output size of a note
pub fn estimate_note_size(note: &Note, with_body: bool) -> usize {
    let mut size = 0;

    // Metadata size with realistic format overhead
    size += note.id().len() + 15; // "N qp-xxx type "
    size += note.title().len() + 20; // Title with quotes and labels
    size += note.note_type().to_string().len() + 15;

    // Tags
    size += note.frontmatter.format_tags().len() + 20; // "tags=..." overhead

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
    use qipu_core::note::NoteFrontmatter;

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
