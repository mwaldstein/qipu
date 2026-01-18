use super::cache::get_mtime;
use super::links::extract_links;
use super::types::{FileEntry, Index, NoteMetadata};
use crate::lib::error::Result;
use crate::lib::store::Store;
use crate::lib::text::tokenize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Index builder - handles construction and updates
pub struct IndexBuilder<'a> {
    store: &'a Store,
    index: Index,
    rebuild: bool,
}

impl<'a> IndexBuilder<'a> {
    /// Create a new index builder
    pub fn new(store: &'a Store) -> Self {
        IndexBuilder {
            store,
            index: Index::new(),
            rebuild: false,
        }
    }

    /// Load existing index (for incremental updates)
    pub fn load_existing(mut self) -> Result<Self> {
        if !self.rebuild {
            self.index = Index::load(&self.store.root().join(".cache"))?;
        }
        Ok(self)
    }

    /// Force full rebuild (ignore existing index)
    pub fn rebuild(mut self) -> Self {
        self.rebuild = true;
        self.index = Index::new();
        self
    }

    /// Build or update the index
    pub fn build(mut self) -> Result<Index> {
        let notes = self.store.list_notes()?;
        let all_ids: HashSet<_> = notes.iter().map(|n| n.id().to_string()).collect();

        // Track which files we've seen this pass
        let mut seen_files: HashSet<PathBuf> = HashSet::new();

        // Clear edges for rebuild (we'll rebuild them all)
        self.index.edges.clear();
        self.index.unresolved.clear();

        // First pass: Update metadata, BM25 stats, and build path mappings
        for note in &notes {
            let path = match &note.path {
                Some(p) => p.clone(),
                None => continue,
            };

            seen_files.insert(path.clone());

            // Check if we need to re-index this file
            let needs_reindex = self.rebuild || self.file_changed(&path);

            if needs_reindex {
                self.prune_note_indexes(&path, note.id());

                // BM25 statistics
                let terms = tokenize(&note.body);
                let word_count = terms.len();
                let unique_terms: HashSet<String> = terms.into_iter().collect();

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
                    .insert(note.id().to_string(), unique_terms);

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

                // Track file for incremental updates
                if let Ok(mtime) = get_mtime(&path) {
                    self.index.files.insert(
                        path.clone(),
                        FileEntry {
                            mtime,
                            note_id: note.id().to_string(),
                        },
                    );

                    // Add reverse mapping for fast lookup
                    self.index.id_to_path.insert(note.id().to_string(), path);
                }
            } else {
                let relative_path = self.relative_path(&path);
                if let Some(meta) = self.index.metadata.get_mut(note.id()) {
                    if meta.path != relative_path {
                        meta.path = relative_path;
                    }
                }
            }
        }

        // Build path-to-ID mapping for link resolution
        use std::collections::HashMap;
        let path_to_id: HashMap<PathBuf, String> = self
            .index
            .id_to_path
            .iter()
            .map(|(id, path)| (path.clone(), id.clone()))
            .collect();

        // Second pass: Extract links with complete path mapping available
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

        // Remove entries for deleted files
        let deleted: Vec<_> = self
            .index
            .files
            .keys()
            .filter(|p| !seen_files.contains(*p))
            .cloned()
            .collect();

        for path in deleted {
            if let Some(entry) = self.index.files.remove(&path) {
                let should_remove = self
                    .index
                    .id_to_path
                    .get(&entry.note_id)
                    .map(|current| current == &path)
                    .unwrap_or(true);

                if should_remove {
                    self.remove_note_from_tags(&entry.note_id);
                    self.remove_note_from_bm25_stats(&entry.note_id);
                    self.index.metadata.remove(&entry.note_id);
                    self.index.id_to_path.remove(&entry.note_id);
                }
            }
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

    fn relative_path(&self, path: &Path) -> String {
        path.strip_prefix(self.store.root())
            .unwrap_or(path)
            .to_string_lossy()
            .to_string()
    }

    fn prune_note_indexes(&mut self, path: &Path, note_id: &str) {
        if let Some(entry) = self.index.files.get(path) {
            let existing_id = entry.note_id.clone();
            self.remove_note_from_tags(&existing_id);
            self.remove_note_from_bm25_stats(&existing_id);

            if existing_id != note_id {
                self.index.metadata.remove(&existing_id);
                self.index.id_to_path.remove(&existing_id);
            }
        } else if self.index.metadata.contains_key(note_id) {
            self.remove_note_from_tags(note_id);
            self.remove_note_from_bm25_stats(note_id);
        }
    }

    fn remove_note_from_bm25_stats(&mut self, note_id: &str) {
        if let Some(length) = self.index.doc_lengths.remove(note_id) {
            self.index.total_docs = self.index.total_docs.saturating_sub(1);
            self.index.total_len = self.index.total_len.saturating_sub(length);
        }

        if let Some(terms) = self.index.note_terms.remove(note_id) {
            for term in terms {
                let mut remove_term = false;
                if let Some(df) = self.index.term_df.get_mut(&term) {
                    *df = df.saturating_sub(1);
                    if *df == 0 {
                        remove_term = true;
                    }
                }
                if remove_term {
                    self.index.term_df.remove(&term);
                }
            }
        }
    }

    fn remove_note_from_tags(&mut self, note_id: &str) {
        let mut empty_tags = Vec::new();
        for (tag, ids) in self.index.tags.iter_mut() {
            ids.retain(|id| id != note_id);
            if ids.is_empty() {
                empty_tags.push(tag.clone());
            }
        }

        for tag in empty_tags {
            self.index.tags.remove(&tag);
        }
    }

    /// Check if a file has changed since last index
    fn file_changed(&self, path: &Path) -> bool {
        let current_mtime = match get_mtime(path) {
            Ok(m) => m,
            Err(_) => return true,
        };

        match self.index.files.get(path) {
            Some(entry) => entry.mtime != current_mtime,
            None => true,
        }
    }
}
