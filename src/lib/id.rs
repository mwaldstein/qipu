//! Note ID generation for qipu
//!
//! ID format per spec (specs/knowledge-model.md):
//! - Format: `qp-<hash>` with adaptive length
//! - Examples: `qp-a1b2`, `qp-f14c3`, `qp-3e7a5b`
//! - Collision-resistant for multi-agent, multi-branch creation
//!
//! Alternate schemes supported:
//! - `ulid`: Time-based ULID identifiers
//! - `timestamp`: Simple timestamp-based IDs

use std::collections::HashSet;
use std::str::FromStr;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::lib::error::{QipuError, Result};

/// ID generation scheme
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IdScheme {
    /// Hash-based IDs (default): `qp-<hex>`
    #[default]
    Hash,
    /// ULID-based IDs: `qp-<ulid>`
    Ulid,
    /// Timestamp-based IDs: `qp-<timestamp>`
    Timestamp,
}

impl FromStr for IdScheme {
    type Err = QipuError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "hash" => Ok(IdScheme::Hash),
            "ulid" => Ok(IdScheme::Ulid),
            "timestamp" => Ok(IdScheme::Timestamp),
            other => Err(QipuError::Other(format!("unknown ID scheme: {}", other))),
        }
    }
}

/// Note ID with the `qp-` prefix
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NoteId(String);

#[allow(dead_code)]
impl NoteId {
    /// The standard ID prefix
    pub const PREFIX: &'static str = "qp-";

    /// Minimum hash length (4 hex chars)
    pub const MIN_HASH_LEN: usize = 4;

    /// Maximum hash length (64 hex chars for SHA256)
    pub const MAX_HASH_LEN: usize = 64;

    /// Create a new NoteId from a raw string (with validation)
    pub fn new(id: impl Into<String>) -> Result<Self> {
        let id = id.into();
        Self::validate(&id)?;
        Ok(NoteId(id))
    }

    /// Create a NoteId without validation (internal use only)
    fn new_unchecked(id: String) -> Self {
        NoteId(id)
    }

    /// Validate an ID string
    fn validate(id: &str) -> Result<()> {
        if !id.starts_with(Self::PREFIX) {
            return Err(QipuError::InvalidNoteId { id: id.to_string() });
        }

        let suffix = &id[Self::PREFIX.len()..];
        if suffix.is_empty() {
            return Err(QipuError::InvalidNoteId { id: id.to_string() });
        }

        // Must be valid hex, ULID, or timestamp format
        // For now, just check it's non-empty alphanumeric
        if !suffix.chars().all(|c| c.is_alphanumeric()) {
            return Err(QipuError::InvalidNoteId { id: id.to_string() });
        }

        Ok(())
    }

    /// Get the ID string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the suffix (part after `qp-`)
    pub fn suffix(&self) -> &str {
        &self.0[Self::PREFIX.len()..]
    }

    /// Generate a new hash-based ID
    ///
    /// Uses adaptive length based on existing IDs to minimize collisions
    /// while keeping IDs short.
    pub fn generate_hash(title: &str, existing_ids: &HashSet<String>) -> Self {
        let timestamp = Utc::now().timestamp_nanos_opt().unwrap_or(0);
        let input = format!("{}:{}:{}", title, timestamp, rand_suffix());

        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let hash = hasher.finalize();
        let full_hex = hex::encode(hash);

        // Find minimum length that doesn't collide
        let mut len = Self::MIN_HASH_LEN;
        loop {
            let candidate = format!("{}{}", Self::PREFIX, &full_hex[..len]);
            if !existing_ids.contains(&candidate) || len >= Self::MAX_HASH_LEN {
                return NoteId::new_unchecked(candidate);
            }
            len += 1;
        }
    }

    /// Generate a new ULID-based ID
    pub fn generate_ulid() -> Self {
        let ulid = ulid::Ulid::new();
        NoteId::new_unchecked(format!(
            "{}{}",
            Self::PREFIX,
            ulid.to_string().to_lowercase()
        ))
    }

    /// Generate a new timestamp-based ID
    pub fn generate_timestamp() -> Self {
        let now = Utc::now();
        let ts = now.format("%Y%m%d%H%M%S").to_string();
        NoteId::new_unchecked(format!("{}{}", Self::PREFIX, ts))
    }

    /// Generate a new ID using the specified scheme
    pub fn generate(scheme: IdScheme, title: &str, existing_ids: &HashSet<String>) -> Self {
        match scheme {
            IdScheme::Hash => Self::generate_hash(title, existing_ids),
            IdScheme::Ulid => Self::generate_ulid(),
            IdScheme::Timestamp => Self::generate_timestamp(),
        }
    }
}

impl std::fmt::Display for NoteId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for NoteId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Generate a random suffix for hash uniqueness
fn rand_suffix() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    // Mix in nanoseconds for randomness
    duration.as_nanos() as u64 ^ (duration.as_secs() * 1_000_000_007)
}

/// Generate a slug from a title
///
/// Converts title to lowercase, replaces non-alphanumeric with hyphens,
/// and removes leading/trailing hyphens.
pub fn slugify(title: &str) -> String {
    slug::slugify(title)
}

/// Generate a filename from ID and title
///
/// Format: `<id>-<slug(title)>.md`
/// Example: `qp-a1b2-zettelkasten-note-types.md`
pub fn filename(id: &NoteId, title: &str) -> String {
    let slug = slugify(title);
    if slug.is_empty() {
        format!("{}.md", id)
    } else {
        format!("{}-{}.md", id, slug)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_id_validation() {
        assert!(NoteId::new("qp-a1b2").is_ok());
        assert!(NoteId::new("qp-f14c3").is_ok());
        assert!(NoteId::new("qp-01j5kzqm1234").is_ok());

        assert!(NoteId::new("invalid").is_err());
        assert!(NoteId::new("qp-").is_err());
        assert!(NoteId::new("").is_err());
    }

    #[test]
    fn test_generate_hash_id() {
        let existing = HashSet::new();
        let id = NoteId::generate_hash("Test Title", &existing);
        assert!(id.as_str().starts_with("qp-"));
        assert!(id.suffix().len() >= NoteId::MIN_HASH_LEN);
    }

    #[test]
    fn test_generate_ulid() {
        let id = NoteId::generate_ulid();
        assert!(id.as_str().starts_with("qp-"));
        assert_eq!(id.suffix().len(), 26); // ULID is 26 chars
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(
            slugify("Zettelkasten Note Types"),
            "zettelkasten-note-types"
        );
        assert_eq!(slugify("Test!@#$%"), "test");
    }

    #[test]
    fn test_filename() {
        let id = NoteId::new("qp-a1b2").unwrap();
        assert_eq!(filename(&id, "Hello World"), "qp-a1b2-hello-world.md");
        assert_eq!(filename(&id, ""), "qp-a1b2.md");
    }
}
