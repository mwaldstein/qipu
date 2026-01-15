use super::types::SelectedNote;
use crate::lib::note::Note;

/// Apply character budget to notes
/// Returns (truncated, notes_to_output)
pub fn apply_budget<'a>(
    notes: &'a [SelectedNote<'a>],
    max_chars: Option<usize>,
    with_body: bool,
) -> (bool, Vec<&'a SelectedNote<'a>>) {
    let Some(budget) = max_chars else {
        return (false, notes.iter().collect());
    };

    let mut result = Vec::new();
    let mut used_chars = 0;
    let mut truncated = false;

    // Conservative header estimate with buffer
    // Different formats have different header sizes, so we use a conservative estimate
    let header_estimate = 250; // Conservative header size estimate
    used_chars += header_estimate;

    for note in notes {
        let note_size = estimate_note_size(note.note, with_body);

        // Add 10% safety buffer to ensure actual output doesn't exceed budget
        let note_size_with_buffer = note_size + (note_size / 10);

        if used_chars + note_size_with_buffer <= budget {
            result.push(note);
            used_chars += note_size_with_buffer;
        } else {
            truncated = true;
            break;
        }
    }

    (truncated, result)
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
