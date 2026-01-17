use crate::lib::error::{QipuError, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

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

/// Typed link relationship (per specs/knowledge-model.md and specs/semantic-graph.md)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LinkType(String);

impl PartialEq<&str> for LinkType {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<String> for LinkType {
    fn eq(&self, other: &String) -> bool {
        &self.0 == other
    }
}

impl From<String> for LinkType {
    fn from(s: String) -> Self {
        LinkType(s.to_lowercase())
    }
}

impl From<&str> for LinkType {
    fn from(s: &str) -> Self {
        LinkType(s.to_lowercase())
    }
}

impl Default for LinkType {
    fn default() -> Self {
        LinkType("related".to_string())
    }
}

impl LinkType {
    // Standard Ontology Constants
    pub const RELATED: &'static str = "related";
    pub const DERIVED_FROM: &'static str = "derived-from";
    pub const SUPPORTS: &'static str = "supports";
    pub const CONTRADICTS: &'static str = "contradicts";
    pub const PART_OF: &'static str = "part-of";
    pub const ANSWERS: &'static str = "answers";
    pub const REFINES: &'static str = "refines";
    pub const SAME_AS: &'static str = "same-as";
    pub const ALIAS_OF: &'static str = "alias-of";
    pub const FOLLOWS: &'static str = "follows";

    // Inverses (Standard)
    pub const DERIVED_TO: &'static str = "derived-to";
    pub const SUPPORTED_BY: &'static str = "supported-by";
    pub const CONTRADICTED_BY: &'static str = "contradicted-by";
    pub const HAS_PART: &'static str = "has-part";
    pub const ANSWERED_BY: &'static str = "answered-by";
    pub const REFINED_BY: &'static str = "refined-by";
    pub const HAS_ALIAS: &'static str = "has-alias";
    pub const PRECEDES: &'static str = "precedes";

    /// Create a new LinkType from a string
    pub fn new(s: &str) -> Self {
        LinkType(s.to_lowercase())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// All valid link types (stored on disk by default)
    pub const VALID_TYPES: &'static [&'static str] = &[
        Self::RELATED,
        Self::DERIVED_FROM,
        Self::SUPPORTS,
        Self::CONTRADICTS,
        Self::PART_OF,
        Self::ANSWERS,
        Self::REFINES,
        Self::SAME_AS,
        Self::ALIAS_OF,
        Self::FOLLOWS,
    ];

    /// Returns the inverse of a link type using standard ontology
    pub fn inverse(&self) -> Self {
        let inv = match self.0.as_str() {
            Self::RELATED => Self::RELATED,
            Self::DERIVED_FROM => Self::DERIVED_TO,
            Self::DERIVED_TO => Self::DERIVED_FROM,
            Self::SUPPORTS => Self::SUPPORTED_BY,
            Self::SUPPORTED_BY => Self::SUPPORTS,
            Self::CONTRADICTS => Self::CONTRADICTED_BY,
            Self::CONTRADICTED_BY => Self::CONTRADICTS,
            Self::PART_OF => Self::HAS_PART,
            Self::HAS_PART => Self::PART_OF,
            Self::ANSWERS => Self::ANSWERED_BY,
            Self::ANSWERED_BY => Self::ANSWERS,
            Self::REFINES => Self::REFINED_BY,
            Self::REFINED_BY => Self::REFINES,
            Self::SAME_AS => Self::SAME_AS,
            Self::ALIAS_OF => Self::HAS_ALIAS,
            Self::HAS_ALIAS => Self::ALIAS_OF,
            Self::FOLLOWS => Self::PRECEDES,
            Self::PRECEDES => Self::FOLLOWS,
            other => {
                if let Some(stripped) = other.strip_prefix("inverse-") {
                    stripped
                } else {
                    return LinkType(format!("inverse-{}", other));
                }
            }
        };
        LinkType(inv.to_string())
    }
}

impl FromStr for LinkType {
    type Err = QipuError;

    fn from_str(s: &str) -> Result<Self> {
        Ok(LinkType(s.to_lowercase()))
    }
}

impl fmt::Display for LinkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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
