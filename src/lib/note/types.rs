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
    /// Solution to a question
    Answers,
    /// Improved iteration of a note
    Refines,
    /// Strong identity/synonym
    SameAs,
    /// Alternative name for canonical note
    AliasOf,
    /// Sequence: note comes after another
    Follows,

    // Inverses (Virtual)
    /// Inverse of derived-from
    DerivedTo,
    /// Inverse of supports
    SupportedBy,
    /// Inverse of contradicts
    ContradictedBy,
    /// Inverse of part-of
    HasPart,
    /// Inverse of answers
    AnsweredBy,
    /// Inverse of refines
    RefinedBy,
    /// Inverse of alias-of
    HasAlias,
    /// Inverse of follows
    Precedes,
}

impl LinkType {
    /// All valid link types (stored on disk)
    pub const VALID_TYPES: &'static [&'static str] = &[
        "related",
        "derived-from",
        "supports",
        "contradicts",
        "part-of",
        "answers",
        "refines",
        "same-as",
        "alias-of",
        "follows",
    ];

    /// Returns the inverse of a link type
    pub fn inverse(&self) -> Self {
        match self {
            LinkType::Related => LinkType::Related,
            LinkType::DerivedFrom => LinkType::DerivedTo,
            LinkType::DerivedTo => LinkType::DerivedFrom,
            LinkType::Supports => LinkType::SupportedBy,
            LinkType::SupportedBy => LinkType::Supports,
            LinkType::Contradicts => LinkType::ContradictedBy,
            LinkType::ContradictedBy => LinkType::Contradicts,
            LinkType::PartOf => LinkType::HasPart,
            LinkType::HasPart => LinkType::PartOf,
            LinkType::Answers => LinkType::AnsweredBy,
            LinkType::AnsweredBy => LinkType::Answers,
            LinkType::Refines => LinkType::RefinedBy,
            LinkType::RefinedBy => LinkType::Refines,
            LinkType::SameAs => LinkType::SameAs,
            LinkType::AliasOf => LinkType::HasAlias,
            LinkType::HasAlias => LinkType::AliasOf,
            LinkType::Follows => LinkType::Precedes,
            LinkType::Precedes => LinkType::Follows,
        }
    }
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
            "answers" => Ok(LinkType::Answers),
            "refines" => Ok(LinkType::Refines),
            "same-as" => Ok(LinkType::SameAs),
            "alias-of" => Ok(LinkType::AliasOf),
            "follows" => Ok(LinkType::Follows),

            // Virtual/Inverse types
            "derived-to" => Ok(LinkType::DerivedTo),
            "supported-by" => Ok(LinkType::SupportedBy),
            "contradicted-by" => Ok(LinkType::ContradictedBy),
            "has-part" => Ok(LinkType::HasPart),
            "answered-by" => Ok(LinkType::AnsweredBy),
            "refined-by" => Ok(LinkType::RefinedBy),
            "has-alias" => Ok(LinkType::HasAlias),
            "precedes" => Ok(LinkType::Precedes),

            other => Err(QipuError::Other(format!(
                "unknown link type: {} (expected one of the standard ontology types)",
                other
            ))),
        }
    }
}

impl fmt::Display for LinkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            LinkType::Related => "related",
            LinkType::DerivedFrom => "derived-from",
            LinkType::Supports => "supports",
            LinkType::Contradicts => "contradicts",
            LinkType::PartOf => "part-of",
            LinkType::Answers => "answers",
            LinkType::Refines => "refines",
            LinkType::SameAs => "same-as",
            LinkType::AliasOf => "alias-of",
            LinkType::Follows => "follows",

            LinkType::DerivedTo => "derived-to",
            LinkType::SupportedBy => "supported-by",
            LinkType::ContradictedBy => "contradicted-by",
            LinkType::HasPart => "has-part",
            LinkType::AnsweredBy => "answered-by",
            LinkType::RefinedBy => "refined-by",
            LinkType::HasAlias => "has-alias",
            LinkType::Precedes => "precedes",
        };
        write!(f, "{}", s)
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
