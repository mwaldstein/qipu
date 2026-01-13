//! Note data structures for qipu
//!
//! Notes are atomic units of knowledge stored as markdown files with YAML frontmatter.
//! Per spec (specs/storage-format.md, specs/knowledge-model.md).

use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::lib::error::{QipuError, Result};

/// Note type (per specs/knowledge-model.md)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NoteType {
    /// Quick capture, low ceremony, meant to be refined later
    #[default]
    Fleeting,
    /// Notes derived from external sources (URLs, books, papers)
    Literature,
    /// Distilled insights in author's own words, meant to stand alone
    Permanent,
    /// Map of Content - curated index organizing a topic
    Moc,
}

impl NoteType {
    /// All valid note types
    pub const VALID_TYPES: &'static [&'static str] =
        &["fleeting", "literature", "permanent", "moc"];
}

impl FromStr for NoteType {
    type Err = QipuError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "fleeting" => Ok(NoteType::Fleeting),
            "literature" => Ok(NoteType::Literature),
            "permanent" => Ok(NoteType::Permanent),
            "moc" => Ok(NoteType::Moc),
            other => Err(QipuError::Other(format!(
                "unknown note type: {} (expected: {})",
                other,
                Self::VALID_TYPES.join(", ")
            ))),
        }
    }
}

impl fmt::Display for NoteType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NoteType::Fleeting => write!(f, "fleeting"),
            NoteType::Literature => write!(f, "literature"),
            NoteType::Permanent => write!(f, "permanent"),
            NoteType::Moc => write!(f, "moc"),
        }
    }
}

/// Typed link relationship (per specs/knowledge-model.md)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LinkType {
    /// Soft relationship
    #[default]
    Related,
    /// Note created because of another note/source
    DerivedFrom,
    /// Evidence supports a claim
    Supports,
    /// Evidence contradicts a claim
    Contradicts,
    /// Note is part of a larger outline/MOC
    PartOf,
}

impl LinkType {
    /// All valid link types
    pub const VALID_TYPES: &'static [&'static str] = &[
        "related",
        "derived-from",
        "supports",
        "contradicts",
        "part-of",
    ];
}

impl FromStr for LinkType {
    type Err = QipuError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "related" => Ok(LinkType::Related),
            "derived-from" => Ok(LinkType::DerivedFrom),
            "supports" => Ok(LinkType::Supports),
            "contradicts" => Ok(LinkType::Contradicts),
            "part-of" => Ok(LinkType::PartOf),
            other => Err(QipuError::Other(format!(
                "unknown link type: {} (expected: {})",
                other,
                Self::VALID_TYPES.join(", ")
            ))),
        }
    }
}

impl fmt::Display for LinkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LinkType::Related => write!(f, "related"),
            LinkType::DerivedFrom => write!(f, "derived-from"),
            LinkType::Supports => write!(f, "supports"),
            LinkType::Contradicts => write!(f, "contradicts"),
            LinkType::PartOf => write!(f, "part-of"),
        }
    }
}

/// A typed link in frontmatter
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypedLink {
    /// Link type
    #[serde(rename = "type")]
    pub link_type: LinkType,
    /// Target note ID
    pub id: String,
}

/// An external source reference
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    /// Source URL
    pub url: String,
    /// Source title (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Date accessed (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessed: Option<String>,
}

/// Note frontmatter (YAML header)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteFrontmatter {
    /// Unique note identifier (required)
    pub id: String,
    /// Note title (required)
    pub title: String,
    /// Note type (optional, defaults to fleeting)
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub note_type: Option<NoteType>,
    /// Creation timestamp (auto-populated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<DateTime<Utc>>,
    /// Last update timestamp (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<DateTime<Utc>>,
    /// Tags for categorization (optional)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// External sources (optional)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<Source>,
    /// Typed links to other notes (optional)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<TypedLink>,
    /// Optional summary field for records output (per specs/records-output.md)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Compaction: list of note IDs this digest compacts (per specs/compaction.md)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub compacts: Vec<String>,
}

impl NoteFrontmatter {
    /// Create new frontmatter with required fields
    pub fn new(id: String, title: String) -> Self {
        NoteFrontmatter {
            id,
            title,
            note_type: None,
            created: Some(Utc::now()),
            updated: None,
            tags: Vec::new(),
            sources: Vec::new(),
            links: Vec::new(),
            summary: None,
            compacts: Vec::new(),
        }
    }

    /// Create frontmatter with a specific type
    pub fn with_type(mut self, note_type: NoteType) -> Self {
        self.note_type = Some(note_type);
        self
    }

    /// Add a tag
    #[allow(dead_code)]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add tags
    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags.extend(tags.into_iter().map(Into::into));
        self
    }

    /// Get the note type, defaulting to Fleeting
    pub fn get_type(&self) -> NoteType {
        self.note_type.unwrap_or_default()
    }
}

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

    /// Get the note title
    pub fn title(&self) -> &str {
        &self.frontmatter.title
    }

    /// Get the note type
    pub fn note_type(&self) -> NoteType {
        self.frontmatter.get_type()
    }

    /// Parse a note from markdown content
    pub fn parse(content: &str, path: Option<PathBuf>) -> Result<Self> {
        let (frontmatter, body) = parse_frontmatter(content, path.as_ref())?;
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
    ///
    /// Order per spec (specs/records-output.md):
    /// 1. frontmatter `summary` field (if present)
    /// 2. `## Summary` section
    /// 3. First paragraph
    /// 4. Empty string
    pub fn summary(&self) -> String {
        // 1. Check frontmatter summary field first
        if let Some(summary) = &self.frontmatter.summary {
            if !summary.is_empty() {
                return summary.clone();
            }
        }

        // 2. Check for ## Summary section
        if let Some(summary) = extract_summary_section(&self.body) {
            return summary;
        }

        // 3. Fall back to first paragraph
        extract_first_paragraph(&self.body).unwrap_or_default()
    }
}

/// Parse YAML frontmatter from markdown content
fn parse_frontmatter(content: &str, path: Option<&PathBuf>) -> Result<(NoteFrontmatter, String)> {
    let content = content.trim_start();

    if !content.starts_with("---") {
        return Err(QipuError::InvalidFrontmatter {
            path: path.cloned().unwrap_or_default(),
            reason: "missing frontmatter delimiter (---)".to_string(),
        });
    }

    let after_first = &content[3..];
    let end_pos = after_first
        .find("\n---")
        .ok_or_else(|| QipuError::InvalidFrontmatter {
            path: path.cloned().unwrap_or_default(),
            reason: "missing closing frontmatter delimiter (---)".to_string(),
        })?;

    let yaml_content = &after_first[..end_pos];
    let body_start = 3 + end_pos + 4; // Skip first ---, yaml, \n---
    let body = if body_start < content.len() {
        content[body_start..].trim_start_matches('\n').to_string()
    } else {
        String::new()
    };

    let frontmatter: NoteFrontmatter =
        serde_yaml::from_str(yaml_content).map_err(|e| QipuError::InvalidFrontmatter {
            path: path.cloned().unwrap_or_default(),
            reason: e.to_string(),
        })?;

    // Validate required fields
    if frontmatter.id.is_empty() {
        return Err(QipuError::InvalidFrontmatter {
            path: path.cloned().unwrap_or_default(),
            reason: "missing required field: id".to_string(),
        });
    }
    if frontmatter.title.is_empty() {
        return Err(QipuError::InvalidFrontmatter {
            path: path.cloned().unwrap_or_default(),
            reason: "missing required field: title".to_string(),
        });
    }

    Ok((frontmatter, body))
}

/// Extract content from a `## Summary` section
///
/// Per spec (specs/records-output.md): extracts "first paragraph under it"
fn extract_summary_section(body: &str) -> Option<String> {
    let lines: Vec<&str> = body.lines().collect();
    let mut in_summary = false;
    let mut in_first_paragraph = false;
    let mut summary_lines = Vec::new();

    for line in lines {
        if line.starts_with("## Summary") {
            in_summary = true;
            continue;
        }
        if in_summary {
            // Stop at next heading
            if line.starts_with("## ") || line.starts_with("# ") {
                break;
            }

            // Skip leading empty lines
            if !in_first_paragraph && line.trim().is_empty() {
                continue;
            }

            // Start collecting the first paragraph
            in_first_paragraph = true;

            // Stop at the end of the first paragraph (empty line)
            if line.trim().is_empty() {
                break;
            }

            summary_lines.push(line);
        }
    }

    if summary_lines.is_empty() {
        return None;
    }

    // Join lines and trim
    let summary = summary_lines.join("\n").trim_end().to_string();

    if summary.is_empty() {
        None
    } else {
        Some(summary)
    }
}

/// Extract the first paragraph from markdown
fn extract_first_paragraph(body: &str) -> Option<String> {
    let body = body.trim();
    if body.is_empty() {
        return None;
    }

    // Skip any leading heading
    let mut lines = body.lines().peekable();
    while let Some(line) = lines.peek() {
        if line.starts_with('#') || line.trim().is_empty() {
            lines.next();
        } else {
            break;
        }
    }

    // Collect lines until empty line
    let mut para_lines = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            break;
        }
        para_lines.push(line);
    }

    if para_lines.is_empty() {
        None
    } else {
        Some(para_lines.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_type_parsing() {
        assert_eq!("fleeting".parse::<NoteType>().unwrap(), NoteType::Fleeting);
        assert_eq!(
            "literature".parse::<NoteType>().unwrap(),
            NoteType::Literature
        );
        assert_eq!(
            "permanent".parse::<NoteType>().unwrap(),
            NoteType::Permanent
        );
        assert_eq!("moc".parse::<NoteType>().unwrap(), NoteType::Moc);
        assert_eq!("MOC".parse::<NoteType>().unwrap(), NoteType::Moc);
    }

    #[test]
    fn test_link_type_parsing() {
        assert_eq!("related".parse::<LinkType>().unwrap(), LinkType::Related);
        assert_eq!(
            "derived-from".parse::<LinkType>().unwrap(),
            LinkType::DerivedFrom
        );
        assert_eq!("supports".parse::<LinkType>().unwrap(), LinkType::Supports);
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
        assert_eq!(note.note_type(), NoteType::Fleeting);
        assert_eq!(note.frontmatter.tags, vec!["test"]);
        assert_eq!(note.body.trim(), "This is the body.");
    }

    #[test]
    fn test_note_to_markdown() {
        let frontmatter = NoteFrontmatter::new("qp-test".to_string(), "Test".to_string())
            .with_type(NoteType::Fleeting);
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
        let summary = extract_summary_section(body).unwrap();
        assert_eq!(summary, "This is the summary.");
    }

    #[test]
    fn test_extract_first_paragraph() {
        let body = r#"# Heading

First paragraph line one.
First paragraph line two.

Second paragraph.
"#;
        let para = extract_first_paragraph(body).unwrap();
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
