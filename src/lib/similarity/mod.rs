//! Similarity engine for finding related notes and duplicates

use crate::lib::index::types::Index;
use crate::lib::text::tokenize;
use std::collections::HashMap;

/// Similarity score between two notes
#[derive(Debug, Clone, PartialEq)]
pub struct SimilarityResult {
    /// Note ID
    pub id: String,
    /// Similarity score (0.0 to 1.0)
    pub score: f64,
}

/// Similarity Engine
pub struct SimilarityEngine<'a> {
    index: &'a Index,
}

impl<'a> SimilarityEngine<'a> {
    /// Create a new Similarity Engine
    pub fn new(index: &'a Index) -> Self {
        SimilarityEngine { index }
    }

    /// Calculate cosine similarity between two sets of terms using BM25 weights
    pub fn calculate_similarity(&self, note_id_a: &str, note_id_b: &str) -> f64 {
        let terms_a = match self.index.note_terms.get(note_id_a) {
            Some(t) => t,
            None => return 0.0,
        };
        let terms_b = match self.index.note_terms.get(note_id_b) {
            Some(t) => t,
            None => return 0.0,
        };

        if terms_a.is_empty() || terms_b.is_empty() {
            return 0.0;
        }

        // Get BM25 weighted vectors for both notes
        let vec_a = self.get_bm25_vector(note_id_a, terms_a);
        let vec_b = self.get_bm25_vector(note_id_b, terms_b);

        self.cosine_similarity(&vec_a, &vec_b)
    }

    /// Get top N similar notes for a given note
    pub fn find_similar(
        &self,
        note_id: &str,
        limit: usize,
        threshold: f64,
    ) -> Vec<SimilarityResult> {
        let mut results = Vec::new();

        for other_id in self.index.metadata.keys() {
            if other_id == note_id {
                continue;
            }

            let score = self.calculate_similarity(note_id, other_id);
            if score >= threshold {
                results.push(SimilarityResult {
                    id: other_id.clone(),
                    score,
                });
            }
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(limit);
        results
    }

    /// Find all near-duplicates in the store
    pub fn find_all_duplicates(&self, threshold: f64) -> Vec<(String, String, f64)> {
        let mut duplicates = Vec::new();
        let ids: Vec<_> = self.index.metadata.keys().cloned().collect();

        for i in 0..ids.len() {
            for j in i + 1..ids.len() {
                let score = self.calculate_similarity(&ids[i], &ids[j]);
                if score >= threshold {
                    duplicates.push((ids[i].clone(), ids[j].clone(), score));
                }
            }
        }

        duplicates.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        duplicates
    }

    /// Calculate cosine similarity between two weighted vectors
    fn cosine_similarity(&self, vec_a: &HashMap<String, f64>, vec_b: &HashMap<String, f64>) -> f64 {
        let mut dot_product = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;

        for (term, weight) in vec_a {
            norm_a += weight * weight;
            if let Some(weight_b) = vec_b.get(term) {
                dot_product += weight * weight_b;
            }
        }

        for weight in vec_b.values() {
            norm_b += weight * weight;
        }

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a.sqrt() * norm_b.sqrt())
    }

    /// Get BM25 weighted vector for a note
    fn get_bm25_vector(
        &self,
        note_id: &str,
        terms: &std::collections::HashSet<String>,
    ) -> HashMap<String, f64> {
        let mut vector = HashMap::new();
        let total_docs = self.index.total_docs as f64;
        let avgdl = (self.index.total_len as f64 / total_docs).max(1.0);
        let doc_len = *self.index.doc_lengths.get(note_id).unwrap_or(&0) as f64;

        let k1 = 1.2;
        let b = 0.75;

        // In a real BM25, we'd need term frequencies within the document.
        // Since we only store the set of terms in the index for now, we assume tf=1
        // for each term in the note for similarity purposes.
        // TODO: Store term frequencies in index for more accurate similarity.
        let tf = 1.0;

        for term in terms {
            let df = *self.index.term_df.get(term).unwrap_or(&1) as f64;
            let idf = ((total_docs - df + 0.5) / (df + 0.5) + 1.0).ln();

            let weight = idf * (tf * (k1 + 1.0)) / (tf + k1 * (1.0 - b + b * (doc_len / avgdl)));
            vector.insert(term.clone(), weight);
        }

        vector
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::index::types::NoteMetadata;
    use crate::lib::note::NoteType;
    use std::collections::HashSet;

    fn create_test_index() -> Index {
        let mut index = Index::new();

        // Note 1: Apple Banana Cherry
        let id1 = "qp-1".to_string();
        let terms1: HashSet<_> = vec!["apple", "banana", "cherry"]
            .into_iter()
            .map(String::from)
            .collect();
        index.note_terms.insert(id1.clone(), terms1.clone());
        index.doc_lengths.insert(id1.clone(), 3);
        index.metadata.insert(
            id1.clone(),
            NoteMetadata {
                id: id1.clone(),
                title: "Note 1".to_string(),
                note_type: NoteType::Permanent,
                tags: vec![],
                path: "1.md".to_string(),
                created: None,
                updated: None,
            },
        );

        // Note 2: Apple Banana Date
        let id2 = "qp-2".to_string();
        let terms2: HashSet<_> = vec!["apple", "banana", "date"]
            .into_iter()
            .map(String::from)
            .collect();
        index.note_terms.insert(id2.clone(), terms2.clone());
        index.doc_lengths.insert(id2.clone(), 3);
        index.metadata.insert(
            id2.clone(),
            NoteMetadata {
                id: id2.clone(),
                title: "Note 2".to_string(),
                note_type: NoteType::Permanent,
                tags: vec![],
                path: "2.md".to_string(),
                created: None,
                updated: None,
            },
        );

        // Stats
        index.total_docs = 2;
        index.total_len = 6;
        for term in vec!["apple", "banana", "cherry", "date"] {
            let df = if term == "apple" || term == "banana" {
                2
            } else {
                1
            };
            index.term_df.insert(term.to_string(), df);
        }

        index
    }

    #[test]
    fn test_similarity_calculation() {
        let index = create_test_index();
        let engine = SimilarityEngine::new(&index);

        let score = engine.calculate_similarity("qp-1", "qp-2");
        assert!(score > 0.0);
        assert!(score < 1.0);

        let self_score = engine.calculate_similarity("qp-1", "qp-1");
        assert!((self_score - 1.0).abs() < 1e-9);
    }
}
