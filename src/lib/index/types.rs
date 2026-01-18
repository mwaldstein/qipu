use crate::lib::note::{LinkType, NoteType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Link source - where the link was defined
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkSource {
    /// Link defined in frontmatter `links[]` array
    Typed,
    /// Link extracted from markdown body (wiki-style or markdown links)
    Inline,
    /// Virtual link generated at query time (e.g. semantic inverse)
    Virtual,
}

impl std::fmt::Display for LinkSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkSource::Typed => write!(f, "typed"),
            LinkSource::Inline => write!(f, "inline"),
            LinkSource::Virtual => write!(f, "virtual"),
        }
    }
}

/// An edge in the note graph
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edge {
    /// Source note ID
    pub from: String,
    /// Target note ID
    pub to: String,
    /// Link type (related, supports, etc.)
    #[serde(rename = "type")]
    pub link_type: LinkType,
    /// Where the link was defined
    pub source: LinkSource,
}

impl Edge {
    /// Invert this edge semantically
    pub fn invert(&self, config: &crate::lib::config::StoreConfig) -> Self {
        let inverted_type_str = config.get_inverse(self.link_type.as_str());

        Edge {
            from: self.to.clone(),
            to: self.from.clone(),
            link_type: LinkType::new(&inverted_type_str),
            source: LinkSource::Virtual,
        }
    }
}

/// Metadata for a single note (stored in index)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteMetadata {
    /// Note ID
    pub id: String,
    /// Note title
    pub title: String,
    /// Note type
    #[serde(rename = "type")]
    pub note_type: NoteType,
    /// Tags
    pub tags: Vec<String>,
    /// File path relative to store
    pub path: String,
    /// Creation timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<DateTime<Utc>>,
    /// Last update timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<DateTime<Utc>>,
}

/// File metadata for incremental indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FileEntry {
    /// File modification time
    pub(crate) mtime: u64,
    /// Note ID in this file
    pub(crate) note_id: String,
}

/// Search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Note ID (may be canonical digest if compaction is resolved)
    pub id: String,
    /// Note title
    pub title: String,
    /// Note type
    #[serde(rename = "type")]
    pub note_type: NoteType,
    /// Tags
    pub tags: Vec<String>,
    /// File path
    pub path: String,
    /// Match context (snippet showing where the match occurred)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_context: Option<String>,
    /// Relevance score (higher is better)
    pub relevance: f64,
    /// Via field - indicates which compacted note triggered this result
    /// Per spec (specs/compaction.md line 122): when a digest appears because
    /// a compacted note matched, annotate with via=<matching-note-id>
    #[serde(skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
}

/// The complete index structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Index {
    /// Index format version
    pub version: u32,
    /// Metadata index: id -> note metadata
    pub metadata: HashMap<String, NoteMetadata>,
    /// Tag index: tag -> [note ids]
    pub tags: HashMap<String, Vec<String>>,
    /// Graph: all edges
    pub edges: Vec<Edge>,
    /// Unresolved links (links to non-existent IDs)
    pub unresolved: HashSet<String>,
    /// File tracking for incremental updates
    #[serde(default)]
    pub(crate) files: HashMap<PathBuf, FileEntry>,
    /// Reverse mapping: note_id -> file_path (for fast lookup)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub(crate) id_to_path: HashMap<String, PathBuf>,

    /// Total number of documents (for BM25)
    #[serde(default)]
    pub total_docs: usize,
    /// Total number of terms across all documents (for BM25)
    #[serde(default)]
    pub total_len: usize,
    /// Document lengths: note_id -> word count (for BM25)
    #[serde(default)]
    pub doc_lengths: HashMap<String, usize>,
    /// Term document frequency: term -> number of documents containing it (for BM25)
    #[serde(default)]
    pub term_df: HashMap<String, usize>,
    /// Term frequencies in each note with field weighting applied (for TF-IDF similarity)
    /// Maps note_id -> (term -> weighted_frequency)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub(crate) note_terms: HashMap<String, HashMap<String, f64>>,
}

/// Current index format version
pub const INDEX_VERSION: u32 = 2;

impl Index {
    /// Create a new empty index
    pub fn new() -> Self {
        Index {
            version: INDEX_VERSION,
            metadata: HashMap::new(),
            tags: HashMap::new(),
            edges: Vec::new(),
            unresolved: HashSet::new(),
            files: HashMap::new(),
            id_to_path: HashMap::new(),
            total_docs: 0,
            total_len: 0,
            doc_lengths: HashMap::new(),
            term_df: HashMap::new(),
            note_terms: HashMap::new(),
        }
    }

    /// Get file path for a note ID (for fast lookup)
    pub fn get_note_path(&self, note_id: &str) -> Option<&PathBuf> {
        self.id_to_path.get(note_id)
    }

    /// Get metadata for a note by ID
    pub fn get_metadata(&self, id: &str) -> Option<&NoteMetadata> {
        self.metadata.get(id)
    }

    /// Get outbound edges from a note
    pub fn get_outbound_edges(&self, id: &str) -> Vec<&Edge> {
        self.edges.iter().filter(|e| e.from == id).collect()
    }

    /// Get inbound edges (backlinks) to a note
    pub fn get_inbound_edges(&self, id: &str) -> Vec<&Edge> {
        self.edges.iter().filter(|e| e.to == id).collect()
    }

    /// Check if a note ID exists in the index
    pub fn contains(&self, id: &str) -> bool {
        self.metadata.contains_key(id)
    }
}
