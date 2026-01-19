use super::links::extract_links;
use super::types::{Index, NoteMetadata};
use crate::lib::error::Result;
use crate::lib::store::Store;
use crate::lib::text::tokenize_with_stemming;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Index builder - handles construction and updates
pub struct IndexBuilder<'a> {
    store: &'a Store,
    index: Index,
}

impl<'a> IndexBuilder<'a> {
    /// Create a new index builder
    pub fn new(store: &'a Store) -> Self {
        IndexBuilder {
            store,
            index: Index::new(),
        }
    }

    /// Build the index
    pub fn build(mut self) -> Result<Index> {
        let notes = self.store.list_notes()?;
        let all_ids: HashSet<_> = notes.iter().map(|n| n.id().to_string()).collect();

        // Build path-to-ID mapping for link resolution
        let mut path_to_id: HashMap<PathBuf, String> = HashMap::new();
        for note in &notes {
            if let Some(path) = &note.path {
                path_to_id.insert(path.clone(), note.id().to_string());
            }
        }

        for note in &notes {
            let path = match &note.path {
                Some(p) => p.clone(),
                None => continue,
            };

            // TF-IDF statistics with field weighting
            // Per spec: title weight=2.0, tags weight=1.5, body weight=1.0
            // Use stemming for better similarity matching (e.g., "graph" matches "graphs")
            let mut term_freqs: HashMap<String, f64> = HashMap::new();

            // Tokenize and weight title (weight: 2.0)
            for term in tokenize_with_stemming(note.title(), true) {
                *term_freqs.entry(term).or_insert(0.0) += 2.0;
            }

            // Tokenize and weight tags (weight: 1.5)
            for tag in &note.frontmatter.tags {
                for term in tokenize_with_stemming(tag, true) {
                    *term_freqs.entry(term).or_insert(0.0) += 1.5;
                }
            }

            // Tokenize and weight body (weight: 1.0)
            for term in tokenize_with_stemming(&note.body, true) {
                *term_freqs.entry(term).or_insert(0.0) += 1.0;
            }

            let word_count = term_freqs.values().map(|&f| f as usize).sum();
            let unique_terms: HashSet<String> = term_freqs.keys().cloned().collect();

            self.index.total_docs += 1;
            self.index.total_len += word_count;
            self.index
                .doc_lengths
                .insert(note.id().to_string(), word_count);

            for term in &unique_terms {
                *self.index.term_df.entry(term.clone()).or_insert(0) += 1;
            }
            self.index
                .note_terms
                .insert(note.id().to_string(), term_freqs);

            // Update metadata
            let meta = NoteMetadata {
                id: note.id().to_string(),
                title: note.title().to_string(),
                note_type: note.note_type(),
                tags: note.frontmatter.tags.clone(),
                path: self.relative_path(&path),
                created: note.frontmatter.created,
                updated: note.frontmatter.updated,
            };

            // Update tag index
            for tag in &meta.tags {
                self.index
                    .tags
                    .entry(tag.clone())
                    .or_default()
                    .push(meta.id.clone());
            }

            self.index.metadata.insert(meta.id.clone(), meta);
        }

        // Extract links with complete path mapping available
        for note in &notes {
            let path = note.path.as_ref();
            let edges = extract_links(
                note,
                &all_ids,
                &mut self.index.unresolved,
                path.map(|p| p.as_path()),
                &path_to_id,
            );
            self.index.edges.extend(edges);
        }

        // Deduplicate tag lists
        for ids in self.index.tags.values_mut() {
            ids.sort();
            ids.dedup();
        }

        // Sort edges for determinism
        self.index.edges.sort_by(|a, b| {
            a.from
                .cmp(&b.from)
                .then_with(|| a.link_type.cmp(&b.link_type))
                .then_with(|| a.to.cmp(&b.to))
        });

        Ok(self.index)
    }

    #[allow(clippy::ptr_arg)]
    fn relative_path(&self, path: &PathBuf) -> String {
        path.strip_prefix(self.store.root())
            .unwrap_or(path)
            .to_string_lossy()
            .to_string()
    }
}
