//! Similarity engine for finding related notes and duplicates

mod tfidf;

mod duplicates;

mod tags;

mod graph;

mod calculation;

pub use duplicates::find_all_duplicates;

use crate::index::types::Index;

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

    /// Calculate cosine similarity between two notes using TF-IDF vectors
    pub fn calculate_similarity(&self, note_id_a: &str, note_id_b: &str) -> f64 {
        calculation::calculate_similarity(self.index, note_id_a, note_id_b)
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

    /// Find notes that share tags with the given note
    pub fn find_by_shared_tags(&self, note_id: &str, limit: usize) -> Vec<SimilarityResult> {
        tags::find_by_shared_tags(self.index, note_id, limit)
    }

    /// Find notes within 2 hops in the link graph
    pub fn find_by_2hop_neighborhood(&self, note_id: &str, limit: usize) -> Vec<SimilarityResult> {
        graph::find_by_2hop_neighborhood(self.index, note_id, limit)
    }
}

#[cfg(test)]
mod tests;
