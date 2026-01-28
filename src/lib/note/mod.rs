//! Note data structures for qipu
//!
//! Notes are atomic units of knowledge stored as markdown files with YAML frontmatter.

pub mod frontmatter;
pub mod parse;
pub mod types;

use crate::lib::error::Result;
use std::path::PathBuf;

pub use frontmatter::NoteFrontmatter;
pub use types::{LinkType, NoteType, Source, TypedLink};

/// A complete note (frontmatter + body)
#[derive(Debug, Clone)]
pub struct Note {
    /// Note frontmatter
    pub frontmatter: NoteFrontmatter,
    /// Note body (markdown content after frontmatter)
    pub body: String,
    /// Path to the note file (if loaded from disk)
    pub path: Option<PathBuf>,
}

impl Note {
    /// Create a new note
    pub fn new(frontmatter: NoteFrontmatter, body: impl Into<String>) -> Self {
        Note {
            frontmatter,
            body: body.into(),
            path: None,
        }
    }

    /// Get the note ID
    pub fn id(&self) -> &str {
        &self.frontmatter.id
    }

    /// Get the note ID as a String
    pub fn id_string(&self) -> String {
        self.id().to_string()
    }

    /// Get the note title
    pub fn title(&self) -> &str {
        &self.frontmatter.title
    }

    /// Get the path as a display string (if available)
    pub fn path_display(&self) -> Option<String> {
        self.path.as_ref().map(|p| p.display().to_string())
    }

    /// Get the note type
    pub fn note_type(&self) -> NoteType {
        self.frontmatter.get_type()
    }

    /// Parse a note from markdown content
    #[tracing::instrument(skip(content), fields(path = ?path))]
    pub fn parse(content: &str, path: Option<PathBuf>) -> Result<Self> {
        let (frontmatter, body) = parse::parse_frontmatter(content, path.as_ref())?;
        Ok(Note {
            frontmatter,
            body,
            path,
        })
    }

    /// Serialize the note to markdown
    pub fn to_markdown(&self) -> Result<String> {
        let yaml = serde_yaml::to_string(&self.frontmatter)?;
        Ok(format!("---\n{}---\n\n{}", yaml, self.body))
    }

    /// Extract the summary from the note
    pub fn summary(&self) -> String {
        // 1. Check frontmatter summary field first
        if let Some(summary) = &self.frontmatter.summary {
            if !summary.is_empty() {
                return summary.clone();
            }
        }

        // 2. Check for ## Summary section
        if let Some(summary) = parse::extract_summary_section(&self.body) {
            return summary;
        }

        // 3. Fall back to first paragraph
        parse::extract_first_paragraph(&self.body).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_type_parsing() {
        assert_eq!(
            "fleeting".parse::<NoteType>().unwrap(),
            NoteType::from(NoteType::FLEETING)
        );
        assert_eq!(
            "literature".parse::<NoteType>().unwrap(),
            NoteType::from(NoteType::LITERATURE)
        );
        assert_eq!(
            "permanent".parse::<NoteType>().unwrap(),
            NoteType::from(NoteType::PERMANENT)
        );
        assert_eq!(
            "moc".parse::<NoteType>().unwrap(),
            NoteType::from(NoteType::MOC)
        );
        assert_eq!(
            "MOC".parse::<NoteType>().unwrap(),
            NoteType::from(NoteType::MOC)
        );
    }

    #[test]
    fn test_link_type_parsing() {
        assert_eq!(
            "related".parse::<LinkType>().unwrap(),
            LinkType::from(LinkType::RELATED)
        );
        assert_eq!(
            "derived-from".parse::<LinkType>().unwrap(),
            LinkType::from(LinkType::DERIVED_FROM)
        );
        assert_eq!(
            "supports".parse::<LinkType>().unwrap(),
            LinkType::from(LinkType::SUPPORTS)
        );
    }

    #[test]
    fn test_parse_note() {
        let content = r#"---
id: qp-a1b2
title: Test Note
type: fleeting
tags:
  - test
---

This is the body.
"#;

        let note = Note::parse(content, None).unwrap();
        assert_eq!(note.id(), "qp-a1b2");
        assert_eq!(note.title(), "Test Note");
        assert_eq!(note.note_type(), NoteType::from(NoteType::FLEETING));
        assert_eq!(note.frontmatter.tags, vec!["test"]);
        assert_eq!(note.body.trim(), "This is the body.");
    }

    #[test]
    fn test_note_to_markdown() {
        let frontmatter = NoteFrontmatter::new("qp-test".to_string(), "Test".to_string())
            .with_type(NoteType::from(NoteType::FLEETING));
        let note = Note::new(frontmatter, "Body content.");

        let md = note.to_markdown().unwrap();
        assert!(md.contains("id: qp-test"));
        assert!(md.contains("title: Test"));
        assert!(md.contains("Body content."));
    }

    #[test]
    fn test_extract_summary_section() {
        let body = r#"## Summary
This is the summary.

## Notes
More content here.
"#;
        let summary = parse::extract_summary_section(body).unwrap();
        assert_eq!(summary, "This is the summary.");
    }

    #[test]
    fn test_extract_first_paragraph() {
        let body = r#"# Heading

First paragraph line one.
First paragraph line two.

Second paragraph.
"#;
        let para = parse::extract_first_paragraph(body).unwrap();
        assert_eq!(para, "First paragraph line one. First paragraph line two.");
    }

    #[test]
    fn test_summary_from_frontmatter() {
        let content = r#"---
id: qp-test
title: Test Note
summary: This is the frontmatter summary.
---

## Summary
This is the body summary section.

Body content.
"#;

        let note = Note::parse(content, None).unwrap();
        // Frontmatter summary should take precedence
        assert_eq!(note.summary(), "This is the frontmatter summary.");
    }

    #[test]
    fn test_summary_fallback_to_section() {
        let content = r#"---
id: qp-test
title: Test Note
---

## Summary
This is the body summary section.

Body content.
"#;

        let note = Note::parse(content, None).unwrap();
        // Should fall back to ## Summary section
        assert_eq!(note.summary(), "This is the body summary section.");
    }
}
