use crate::lib::note::NoteType;

/// Strip frontmatter from template content
pub(crate) fn strip_frontmatter(content: &str) -> String {
    let content = content.trim_start();
    if let Some(stripped) = content.strip_prefix("---") {
        if let Some(end) = stripped.find("\n---") {
            let after_fm = &stripped[end + 4..];
            return after_fm.trim_start_matches('\n').to_string();
        }
    }
    content.to_string()
}

/// Get default body for a note type
pub(crate) fn default_body(note_type: NoteType) -> String {
    match note_type {
        NoteType::Fleeting => "## Summary\n\n\n\n## Notes\n\n".to_string(),
        NoteType::Literature => "## Summary\n\n\n\n## Notes\n\n\n\n## Quotes\n\n".to_string(),
        NoteType::Permanent => "## Summary\n\n\n\n## Notes\n\n\n\n## See Also\n\n".to_string(),
        NoteType::Moc => {
            "## Summary\n\n\n\n## Overview\n\n\n\n## Reading Path\n\n\n\n## Topics\n\n".to_string()
        }
    }
}

pub(crate) fn default_template(note_type: NoteType) -> &'static str {
    match note_type {
        NoteType::Fleeting => {
            r#"## Summary

<!-- One-sentence summary of this thought -->

## Notes

<!-- Quick capture - refine later -->
"#
        }
        NoteType::Literature => {
            r#"## Summary

<!-- Key takeaway from this source -->

## Notes

<!-- Your notes on this external source -->

## Quotes

<!-- Notable quotes from the source -->
"#
        }
        NoteType::Permanent => {
            r#"## Summary

<!-- One idea, in your own words, that can stand alone -->

## Notes

<!-- Explanation and context -->

## See Also

<!-- Related notes: explain *why* each is related, not just bare links -->
"#
        }
        NoteType::Moc => {
            r#"## Summary

<!-- What this map covers and why it exists -->

## Overview

<!-- Brief introduction to the topic -->

## Reading Path

<!-- Suggested order for exploring this topic -->

## Topics

<!-- Organized links to notes, grouped by subtopic -->
<!-- Explain what belongs here and why -->
"#
        }
    }
}
