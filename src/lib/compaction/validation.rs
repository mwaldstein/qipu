//! Compaction invariant validation

use std::collections::HashSet;

use crate::lib::note::Note;

use super::context::CompactionContext;

impl CompactionContext {
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
