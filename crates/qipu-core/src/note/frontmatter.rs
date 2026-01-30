use super::types::{NoteType, Source, TypedLink};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    /// The original source of the information (per specs/provenance.md)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Name of the human or agent who created the note
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Name of the LLM model used to generate the content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_by: Option<String>,
    /// Hash or ID of the prompt used to generate the content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_hash: Option<String>,
    /// Flag indicating if a human has manually reviewed the content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,
    /// Note importance/quality score (0-100, default 50)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<u8>,
    /// Custom metadata for downstream applications
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom: HashMap<String, serde_yaml::Value>,
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
            source: None,
            author: None,
            generated_by: None,
            prompt_hash: None,
            verified: None,
            value: None,
            custom: HashMap::new(),
        }
    }

    /// Create frontmatter with a specific type
    pub fn with_type(mut self, note_type: NoteType) -> Self {
        self.note_type = Some(note_type);
        self
    }

    /// Add tags
    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags.extend(tags.into_iter().map(Into::into));
        self
    }

    /// Get the note type, defaulting to Fleeting
    pub fn get_type(&self) -> NoteType {
        self.note_type.clone().unwrap_or_default()
    }

    /// Format tags as comma-separated values, using "-" for empty tags
    pub fn format_tags(&self) -> String {
        if self.tags.is_empty() {
            "-".to_string()
        } else {
            self.tags.join(",")
        }
    }
}
