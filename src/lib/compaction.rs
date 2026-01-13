//! Compaction utilities for qipu
//!
//! Implements digest-first navigation and lossless knowledge decay.
//! Per spec: specs/compaction.md

use std::collections::{HashMap, HashSet};

use crate::lib::error::{QipuError, Result};
use crate::lib::note::Note;

/// Compaction context - tracks which notes compact which
#[derive(Debug, Clone)]
pub struct CompactionContext {
    /// Map from note ID to its compactor (digest that compacts it)
    /// Invariant: at most one compactor per note
    pub compactors: HashMap<String, String>,

    /// Map from digest ID to the set of notes it compacts
    pub compacted_by: HashMap<String, Vec<String>>,
}

impl CompactionContext {
    /// Build compaction context from a set of notes
    pub fn build(notes: &[Note]) -> Result<Self> {
        let mut compactors = HashMap::new();
        let mut compacted_by: HashMap<String, Vec<String>> = HashMap::new();

        // Build the mapping from notes to their compactors
        for note in notes {
            let digest_id = &note.frontmatter.id;
            let compacts = &note.frontmatter.compacts;

            if compacts.is_empty() {
                continue;
            }

            // Store what this digest compacts
            compacted_by.insert(digest_id.clone(), compacts.clone());

            // For each compacted note, record its compactor
            for source_id in compacts {
                // Invariant check: at most one compactor per note
                if let Some(existing_compactor) = compactors.get(source_id) {
                    return Err(QipuError::Other(format!(
                        "note {} has multiple compactors: {} and {}",
                        source_id, existing_compactor, digest_id
                    )));
                }
                compactors.insert(source_id.clone(), digest_id.clone());
            }
        }

        Ok(CompactionContext {
            compactors,
            compacted_by,
        })
    }

    /// Get the canonical ID for a note (follow compaction chain to topmost digest)
    /// Returns the original ID if not compacted.
    /// Detects cycles and returns an error.
    pub fn canon(&self, id: &str) -> Result<String> {
        let mut current = id.to_string();
        let mut visited = HashSet::new();

        loop {
            // Check for cycles
            if !visited.insert(current.clone()) {
                return Err(QipuError::Other(format!(
                    "compaction cycle detected involving note {}",
                    current
                )));
            }

            // If no compactor, this is the canonical ID
            match self.compactors.get(&current) {
                None => return Ok(current),
                Some(compactor) => current = compactor.clone(),
            }
        }
    }

    /// Check if a note is compacted by any digest
    pub fn is_compacted(&self, id: &str) -> bool {
        self.compactors.contains_key(id)
    }

    /// Get the direct compactor for a note (if any)
    pub fn get_compactor(&self, id: &str) -> Option<&String> {
        self.compactors.get(id)
    }

    /// Get the notes compacted by a digest
    pub fn get_compacted_notes(&self, digest_id: &str) -> Option<&Vec<String>> {
        self.compacted_by.get(digest_id)
    }

    /// Validate compaction invariants
    /// Returns a list of error messages (empty if valid)
    pub fn validate(&self, notes: &[Note]) -> Vec<String> {
        let mut errors = Vec::new();
        let note_ids: HashSet<String> = notes.iter().map(|n| n.frontmatter.id.clone()).collect();

        // Check for unresolved IDs
        for (source_id, digest_id) in &self.compactors {
            if !note_ids.contains(source_id) {
                errors.push(format!(
                    "compaction references unknown source note: {}",
                    source_id
                ));
            }
            if !note_ids.contains(digest_id) {
                errors.push(format!(
                    "compaction references unknown digest note: {}",
                    digest_id
                ));
            }
        }

        // Check for self-compaction
        for note in notes {
            if note.frontmatter.compacts.contains(&note.frontmatter.id) {
                errors.push(format!(
                    "note {} compacts itself (self-compaction not allowed)",
                    note.frontmatter.id
                ));
            }
        }

        // Check for cycles by trying to canonicalize each note
        for note in notes {
            if let Err(e) = self.canon(&note.frontmatter.id) {
                errors.push(e.to_string());
            }
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::note::NoteFrontmatter;

    #[test]
    fn test_canon_no_compaction() {
        let notes = vec![Note {
            frontmatter: NoteFrontmatter::new("qp-1".to_string(), "Note 1".to_string()),
            body: String::new(),
            path: None,
        }];

        let ctx = CompactionContext::build(&notes).unwrap();
        assert_eq!(ctx.canon("qp-1").unwrap(), "qp-1");
    }

    #[test]
    fn test_canon_single_level() {
        let mut digest = NoteFrontmatter::new("qp-digest".to_string(), "Digest".to_string());
        digest.compacts = vec!["qp-1".to_string(), "qp-2".to_string()];

        let notes = vec![
            Note {
                frontmatter: NoteFrontmatter::new("qp-1".to_string(), "Note 1".to_string()),
                body: String::new(),
                path: None,
            },
            Note {
                frontmatter: NoteFrontmatter::new("qp-2".to_string(), "Note 2".to_string()),
                body: String::new(),
                path: None,
            },
            Note {
                frontmatter: digest,
                body: String::new(),
                path: None,
            },
        ];

        let ctx = CompactionContext::build(&notes).unwrap();
        assert_eq!(ctx.canon("qp-1").unwrap(), "qp-digest");
        assert_eq!(ctx.canon("qp-2").unwrap(), "qp-digest");
        assert_eq!(ctx.canon("qp-digest").unwrap(), "qp-digest");
    }

    #[test]
    fn test_canon_multi_level() {
        let mut digest1 = NoteFrontmatter::new("qp-digest1".to_string(), "Digest 1".to_string());
        digest1.compacts = vec!["qp-1".to_string(), "qp-2".to_string()];

        let mut digest2 = NoteFrontmatter::new("qp-digest2".to_string(), "Digest 2".to_string());
        digest2.compacts = vec!["qp-digest1".to_string(), "qp-3".to_string()];

        let notes = vec![
            Note {
                frontmatter: NoteFrontmatter::new("qp-1".to_string(), "Note 1".to_string()),
                body: String::new(),
                path: None,
            },
            Note {
                frontmatter: NoteFrontmatter::new("qp-2".to_string(), "Note 2".to_string()),
                body: String::new(),
                path: None,
            },
            Note {
                frontmatter: NoteFrontmatter::new("qp-3".to_string(), "Note 3".to_string()),
                body: String::new(),
                path: None,
            },
            Note {
                frontmatter: digest1,
                body: String::new(),
                path: None,
            },
            Note {
                frontmatter: digest2,
                body: String::new(),
                path: None,
            },
        ];

        let ctx = CompactionContext::build(&notes).unwrap();
        assert_eq!(ctx.canon("qp-1").unwrap(), "qp-digest2");
        assert_eq!(ctx.canon("qp-2").unwrap(), "qp-digest2");
        assert_eq!(ctx.canon("qp-3").unwrap(), "qp-digest2");
        assert_eq!(ctx.canon("qp-digest1").unwrap(), "qp-digest2");
        assert_eq!(ctx.canon("qp-digest2").unwrap(), "qp-digest2");
    }

    #[test]
    fn test_cycle_detection() {
        let mut digest1 = NoteFrontmatter::new("qp-digest1".to_string(), "Digest 1".to_string());
        digest1.compacts = vec!["qp-digest2".to_string()];

        let mut digest2 = NoteFrontmatter::new("qp-digest2".to_string(), "Digest 2".to_string());
        digest2.compacts = vec!["qp-digest1".to_string()];

        let notes = vec![
            Note {
                frontmatter: digest1,
                body: String::new(),
                path: None,
            },
            Note {
                frontmatter: digest2,
                body: String::new(),
                path: None,
            },
        ];

        let ctx = CompactionContext::build(&notes).unwrap();
        assert!(ctx.canon("qp-digest1").is_err());
        assert!(ctx.canon("qp-digest2").is_err());
    }

    #[test]
    fn test_multiple_compactors_error() {
        let mut digest1 = NoteFrontmatter::new("qp-digest1".to_string(), "Digest 1".to_string());
        digest1.compacts = vec!["qp-1".to_string()];

        let mut digest2 = NoteFrontmatter::new("qp-digest2".to_string(), "Digest 2".to_string());
        digest2.compacts = vec!["qp-1".to_string()];

        let notes = vec![
            Note {
                frontmatter: NoteFrontmatter::new("qp-1".to_string(), "Note 1".to_string()),
                body: String::new(),
                path: None,
            },
            Note {
                frontmatter: digest1,
                body: String::new(),
                path: None,
            },
            Note {
                frontmatter: digest2,
                body: String::new(),
                path: None,
            },
        ];

        let result = CompactionContext::build(&notes);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_self_compaction() {
        let mut note = NoteFrontmatter::new("qp-1".to_string(), "Note 1".to_string());
        note.compacts = vec!["qp-1".to_string()];

        let notes = vec![Note {
            frontmatter: note,
            body: String::new(),
            path: None,
        }];

        let ctx = CompactionContext::build(&notes).unwrap();
        let errors = ctx.validate(&notes);
        assert!(!errors.is_empty());
        assert!(errors[0].contains("compacts itself"));
    }
}
