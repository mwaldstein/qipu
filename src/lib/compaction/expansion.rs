//! Note expansion and compaction metrics

use std::collections::{HashMap, HashSet};

use crate::lib::note::Note;

use super::context::CompactionContext;
use super::estimate_note_size;

impl CompactionContext {
    /// Get all compacted IDs for a digest with depth traversal
    /// Based on spec lines 131-141 in specs/compaction.md
    /// Returns None if not a digest
    /// If max_nodes is Some(), will truncate and return a tuple with truncated flag
    pub fn get_compacted_ids(
        &self,
        digest_id: &str,
        depth: u32,
        max_nodes: Option<usize>,
    ) -> Option<(Vec<String>, bool)> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut queue: Vec<(String, u32)> = vec![(digest_id.to_string(), 0)];

        while let Some((current_id, current_depth)) = queue.pop() {
            if current_depth >= depth {
                continue;
            }

            if visited.contains(&current_id) {
                continue;
            }
            visited.insert(current_id.clone());

            if let Some(compacted_ids) = self.get_compacted_notes(&current_id) {
                for id in compacted_ids {
                    if !visited.contains(id) {
                        result.push(id.clone());
                        queue.push((id.clone(), current_depth + 1));
                    }
                }
            }
        }

        // Sort for deterministic output (per spec line 141)
        result.sort();

        // Apply max_nodes limit
        let truncated = if let Some(max) = max_nodes {
            if result.len() > max {
                result.truncate(max);
                true
            } else {
                false
            }
        } else {
            false
        };

        if result.is_empty() {
            None
        } else {
            Some((result, truncated))
        }
    }

    /// Get all compacted notes for a digest with depth traversal
    /// Returns None if not a digest
    /// If max_nodes is Some(), will truncate and return a tuple with truncated flag
    pub fn get_compacted_notes_expanded<'a>(
        &self,
        digest_id: &str,
        depth: u32,
        max_nodes: Option<usize>,
        all_notes: &'a [Note],
    ) -> Option<(Vec<&'a Note>, bool)> {
        let ids = self.get_compacted_ids(digest_id, depth, max_nodes)?;
        let note_map: HashMap<&str, &Note> = all_notes
            .iter()
            .map(|n| (n.frontmatter.id.as_str(), n))
            .collect();

        let mut notes = Vec::new();
        for id in &ids.0 {
            if let Some(note) = note_map.get(id.as_str()) {
                notes.push(*note);
            }
        }

        Some((notes, ids.1))
    }

    /// Build note map from notes for efficient lookups
    pub fn build_note_map<'a>(all_notes: &'a [Note]) -> HashMap<&'a str, &'a Note> {
        all_notes
            .iter()
            .map(|note| (note.frontmatter.id.as_str(), note))
            .collect()
    }

    /// Calculate compaction percentage for a digest
    /// Based on spec lines 156-166 in specs/compaction.md
    /// Returns None if not a digest or expanded_size is 0
    /// Note: For efficiency, build the note_map once with build_note_map() and reuse it
    pub fn get_compaction_pct(
        &self,
        digest: &Note,
        note_map: &HashMap<&str, &Note>,
    ) -> Option<f32> {
        // Check if this is a digest (has compacted notes)
        let compacted_ids = self.get_compacted_notes(&digest.frontmatter.id)?;
        if compacted_ids.is_empty() {
            return None;
        }

        // Calculate digest size using summary
        let digest_size = estimate_note_size(digest);

        // Calculate expanded size (sum of direct sources)
        let mut expanded_size = 0usize;
        for source_id in compacted_ids {
            if let Some(note) = note_map.get(source_id.as_str()) {
                expanded_size += estimate_note_size(note);
            }
        }

        // If expanded_size is 0, treat as 0% per spec
        if expanded_size == 0 {
            return Some(0.0);
        }

        // compaction_pct = 100 * (1 - digest_size / expanded_size)
        let ratio = digest_size as f32 / expanded_size as f32;
        Some(100.0 * (1.0 - ratio))
    }
}
