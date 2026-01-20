//! Similarity engine for finding related notes and duplicates

use crate::lib::index::types::Index;
use std::collections::HashMap;

/// Similarity score between two notes
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
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

    /// Calculate cosine similarity between two notes using TF-IDF vectors
    pub fn calculate_similarity(&self, note_id_a: &str, note_id_b: &str) -> f64 {
        let term_freqs_a = match self.index.note_terms.get(note_id_a) {
            Some(t) => t,
            None => return 0.0,
        };
        let term_freqs_b = match self.index.note_terms.get(note_id_b) {
            Some(t) => t,
            None => return 0.0,
        };

        if term_freqs_a.is_empty() || term_freqs_b.is_empty() {
            return 0.0;
        }

        // Get TF-IDF weighted vectors for both notes
        let vec_a = self.get_tfidf_vector(term_freqs_a);
        let vec_b = self.get_tfidf_vector(term_freqs_b);

        self.cosine_similarity(&vec_a, &vec_b)
    }

    /// Get top N similar notes for a given note
    #[allow(dead_code)]
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

    /// Get TF-IDF weighted vector for a note
    fn get_tfidf_vector(&self, term_freqs: &HashMap<String, f64>) -> HashMap<String, f64> {
        let mut vector = HashMap::new();
        let total_docs = self.index.total_docs as f64;

        for (term, &tf) in term_freqs {
            let df = *self.index.term_df.get(term).unwrap_or(&1) as f64;
            // IDF formula with smoothing to avoid zero: log((N + 1) / (df + 1)) + 1
            // This ensures IDF is always positive even when df == N
            let idf = ((total_docs + 1.0) / (df + 1.0)).ln() + 1.0;

            // TF-IDF weight = TF * IDF
            // TF already includes field weighting from index builder
            let weight = tf * idf;
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

    fn create_test_index() -> Index {
        let mut index = Index::new();

        // Note 1: Apple Banana Cherry (each term appears once with weight 1.0)
        let id1 = "qp-1".to_string();
        let mut term_freqs1 = HashMap::new();
        term_freqs1.insert("apple".to_string(), 1.0);
        term_freqs1.insert("banana".to_string(), 1.0);
        term_freqs1.insert("cherry".to_string(), 1.0);
        index.note_terms.insert(id1.clone(), term_freqs1);
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
                value: None,
            },
        );

        // Note 2: Apple Banana Date (each term appears once with weight 1.0)
        let id2 = "qp-2".to_string();
        let mut term_freqs2 = HashMap::new();
        term_freqs2.insert("apple".to_string(), 1.0);
        term_freqs2.insert("banana".to_string(), 1.0);
        term_freqs2.insert("date".to_string(), 1.0);
        index.note_terms.insert(id2.clone(), term_freqs2);
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
                value: None,
            },
        );

        // Stats
        index.total_docs = 2;
        index.total_len = 6;
        for term in ["apple", "banana", "cherry", "date"] {
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

    #[test]
    fn test_similarity_with_stemming() {
        let mut index = Index::new();

        // Note 1: "The graphs are important"
        let id1 = "qp-1".to_string();
        let mut term_freqs1 = HashMap::new();
        // After stemming: "the", "graph", "are", "important"
        // Stop words removed: "the", "are"
        // Result: "graph" (weight from body: 1.0)
        term_freqs1.insert("graph".to_string(), 1.0);
        term_freqs1.insert("import".to_string(), 1.0);
        index.note_terms.insert(id1.clone(), term_freqs1);
        index.doc_lengths.insert(id1.clone(), 2);
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
                value: None,
            },
        );

        // Note 2: "A graph is useful"
        let id2 = "qp-2".to_string();
        let mut term_freqs2 = HashMap::new();
        // After stemming: "a", "graph", "is", "use"
        // Stop words removed: "a", "is"
        // Result: "graph" (weight from body: 1.0), "use" (weight from body: 1.0)
        term_freqs2.insert("graph".to_string(), 1.0);
        term_freqs2.insert("use".to_string(), 1.0);
        index.note_terms.insert(id2.clone(), term_freqs2);
        index.doc_lengths.insert(id2.clone(), 2);
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
                value: None,
            },
        );

        // Stats
        index.total_docs = 2;
        index.total_len = 4;
        index.term_df.insert("graph".to_string(), 2);
        index.term_df.insert("import".to_string(), 1);
        index.term_df.insert("use".to_string(), 1);

        let engine = SimilarityEngine::new(&index);
        let score = engine.calculate_similarity("qp-1", "qp-2");

        // Both notes share "graph" after stemming, so similarity should be > 0
        assert!(
            score > 0.0,
            "Similarity should be > 0 when sharing stemmed terms"
        );
    }
}
