use super::links::extract_links;
use super::types::{Index, NoteMetadata};
use super::weights::{BODY_WEIGHT, TAGS_WEIGHT, TITLE_WEIGHT};
use crate::error::Result;
use crate::note::Note;
use crate::store::Store;
use crate::text::tokenize_with_stemming;
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
    #[tracing::instrument(skip(self), fields(store_root = %self.store.root().display()))]
    pub fn build(mut self) -> Result<Index> {
        let notes = self.store.list_notes()?;
        let all_ids: HashSet<_> = notes.iter().map(|n| n.id().to_string()).collect();

        let path_to_id = self.build_path_to_id_map(&notes);
        let use_stemming = self.store.config().stemming;

        self.build_note_index(&notes, use_stemming);
        self.build_link_graph(&notes, &all_ids, &path_to_id);
        self.finalize_index();

        Ok(self.index)
    }

    fn build_path_to_id_map(&self, notes: &[Note]) -> HashMap<PathBuf, String> {
        let mut map = HashMap::new();
        for note in notes {
            if let Some(path) = &note.path {
                map.insert(path.clone(), note.id().to_string());
            }
        }
        map
    }

    fn build_note_index(&mut self, notes: &[Note], use_stemming: bool) {
        for note in notes {
            let path = match &note.path {
                Some(p) => p.clone(),
                None => continue,
            };

            let term_freqs = self.compute_term_frequencies(note, use_stemming);
            self.update_term_statistics(note.id(), &term_freqs);
            let meta = self.build_note_metadata(note, &path, &term_freqs);
            self.update_tag_index(&meta);
            self.index.metadata.insert(meta.id.clone(), meta);
        }
    }

    fn compute_term_frequencies(&self, note: &Note, use_stemming: bool) -> HashMap<String, f64> {
        let mut term_freqs: HashMap<String, f64> = HashMap::new();

        for term in tokenize_with_stemming(note.title(), use_stemming) {
            *term_freqs.entry(term).or_insert(0.0) += TITLE_WEIGHT;
        }

        for tag in &note.frontmatter.tags {
            for term in tokenize_with_stemming(tag, use_stemming) {
                *term_freqs.entry(term).or_insert(0.0) += TAGS_WEIGHT;
            }
        }

        for term in tokenize_with_stemming(&note.body, use_stemming) {
            *term_freqs.entry(term).or_insert(0.0) += BODY_WEIGHT;
        }

        term_freqs
    }

    fn update_term_statistics(&mut self, note_id: &str, term_freqs: &HashMap<String, f64>) {
        let word_count = term_freqs.values().map(|&f| f as usize).sum();
        let unique_terms: HashSet<String> = term_freqs.keys().cloned().collect();

        self.index.total_docs += 1;
        self.index.total_len += word_count;
        self.index
            .doc_lengths
            .insert(note_id.to_string(), word_count);

        for term in &unique_terms {
            *self.index.term_df.entry(term.clone()).or_insert(0) += 1;
        }
        self.index
            .note_terms
            .insert(note_id.to_string(), term_freqs.clone());
    }

    fn build_note_metadata(
        &self,
        note: &Note,
        path: &PathBuf,
        _term_freqs: &HashMap<String, f64>,
    ) -> NoteMetadata {
        NoteMetadata {
            id: note.id().to_string(),
            title: note.title().to_string(),
            note_type: note.note_type(),
            tags: note.frontmatter.tags.clone(),
            path: self.relative_path(path),
            created: note.frontmatter.created,
            updated: note.frontmatter.updated,
            value: note.frontmatter.value,
        }
    }

    fn update_tag_index(&mut self, meta: &NoteMetadata) {
        for tag in &meta.tags {
            self.index
                .tags
                .entry(tag.clone())
                .or_default()
                .push(meta.id.clone());
        }
    }

    fn build_link_graph(
        &mut self,
        notes: &[Note],
        all_ids: &HashSet<String>,
        path_to_id: &HashMap<PathBuf, String>,
    ) {
        for note in notes {
            let path = note.path.as_ref();
            let edges = extract_links(
                note,
                all_ids,
                &mut self.index.unresolved,
                path.map(|p| p.as_path()),
                path_to_id,
            );
            self.index.edges.extend(edges);
        }
    }

    fn finalize_index(&mut self) {
        for ids in self.index.tags.values_mut() {
            ids.sort();
            ids.dedup();
        }

        self.index.edges.sort_by(|a, b| {
            a.from
                .cmp(&b.from)
                .then_with(|| a.link_type.cmp(&b.link_type))
                .then_with(|| a.to.cmp(&b.to))
        });
    }

    #[allow(clippy::ptr_arg)]
    fn relative_path(&self, path: &PathBuf) -> String {
        path.strip_prefix(self.store.root())
            .unwrap_or(path)
            .to_string_lossy()
            .to_string()
    }
}
