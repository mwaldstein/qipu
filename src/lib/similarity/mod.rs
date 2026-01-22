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

    /// Find notes that share tags with the given note
    pub fn find_by_shared_tags(&self, note_id: &str, limit: usize) -> Vec<SimilarityResult> {
        let mut results = Vec::new();

        let tags = match self.index.get_metadata(note_id) {
            Some(meta) => &meta.tags,
            None => return results,
        };

        if tags.is_empty() {
            return results;
        }

        for (other_id, other_meta) in &self.index.metadata {
            if other_id == note_id {
                continue;
            }

            // Count shared tags
            let shared_count = tags.iter().filter(|t| other_meta.tags.contains(t)).count();

            if shared_count > 0 {
                // Score based on Jaccard similarity: intersection / union
                let union_count = tags.len() + other_meta.tags.len() - shared_count;
                let score = shared_count as f64 / union_count as f64;

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

    /// Find notes within 2 hops in the link graph
    pub fn find_by_2hop_neighborhood(&self, note_id: &str, limit: usize) -> Vec<SimilarityResult> {
        let mut results = Vec::new();
        let mut neighbor_counts: HashMap<String, usize> = HashMap::new();

        // Get 1-hop neighbors
        let outbound = self.index.get_outbound_edges(note_id);
        let inbound = self.index.get_inbound_edges(note_id);

        let mut one_hop = std::collections::HashSet::new();
        for edge in outbound {
            one_hop.insert(edge.to.clone());
        }
        for edge in inbound {
            one_hop.insert(edge.from.clone());
        }

        // Get 2-hop neighbors (linked to 1-hop neighbors)
        for neighbor_id in &one_hop {
            let outbound = self.index.get_outbound_edges(neighbor_id);
            let inbound = self.index.get_inbound_edges(neighbor_id);

            for edge in outbound {
                if edge.to != note_id && !one_hop.contains(&edge.to) {
                    *neighbor_counts.entry(edge.to.clone()).or_insert(0) += 1;
                }
            }
            for edge in inbound {
                if edge.from != note_id && !one_hop.contains(&edge.from) {
                    *neighbor_counts.entry(edge.from.clone()).or_insert(0) += 1;
                }
            }
        }

        // Convert to results, score based on number of 2-hop paths
        for (id, count) in neighbor_counts {
            results.push(SimilarityResult {
                id,
                score: count as f64,
            });
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(limit);
        results
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

    #[test]
    fn test_find_by_shared_tags() {
        let mut index = Index::new();

        // Note 1: tags = ["rust", "programming"]
        index.metadata.insert(
            "qp-1".to_string(),
            NoteMetadata {
                id: "qp-1".to_string(),
                title: "Note 1".to_string(),
                note_type: NoteType::Permanent,
                tags: vec!["rust".to_string(), "programming".to_string()],
                path: "1.md".to_string(),
                created: None,
                updated: None,
                value: None,
            },
        );

        // Note 2: tags = ["rust", "systems"] - shares "rust"
        index.metadata.insert(
            "qp-2".to_string(),
            NoteMetadata {
                id: "qp-2".to_string(),
                title: "Note 2".to_string(),
                note_type: NoteType::Permanent,
                tags: vec!["rust".to_string(), "systems".to_string()],
                path: "2.md".to_string(),
                created: None,
                updated: None,
                value: None,
            },
        );

        // Note 3: tags = ["rust", "programming", "systems"] - shares both tags with Note 1
        index.metadata.insert(
            "qp-3".to_string(),
            NoteMetadata {
                id: "qp-3".to_string(),
                title: "Note 3".to_string(),
                note_type: NoteType::Permanent,
                tags: vec![
                    "rust".to_string(),
                    "programming".to_string(),
                    "systems".to_string(),
                ],
                path: "3.md".to_string(),
                created: None,
                updated: None,
                value: None,
            },
        );

        // Note 4: tags = ["python", "programming"] - shares only "programming"
        index.metadata.insert(
            "qp-4".to_string(),
            NoteMetadata {
                id: "qp-4".to_string(),
                title: "Note 4".to_string(),
                note_type: NoteType::Permanent,
                tags: vec!["python".to_string(), "programming".to_string()],
                path: "4.md".to_string(),
                created: None,
                updated: None,
                value: None,
            },
        );

        // Note 5: tags = ["java"] - shares nothing
        index.metadata.insert(
            "qp-5".to_string(),
            NoteMetadata {
                id: "qp-5".to_string(),
                title: "Note 5".to_string(),
                note_type: NoteType::Permanent,
                tags: vec!["java".to_string()],
                path: "5.md".to_string(),
                created: None,
                updated: None,
                value: None,
            },
        );

        let engine = SimilarityEngine::new(&index);
        let results = engine.find_by_shared_tags("qp-1", 100);

        // Should find qp-2, qp-3, and qp-4 (all share at least one tag)
        assert_eq!(results.len(), 3);

        // qp-3 should have the highest score (Jaccard = 2/3 = 0.666...)
        assert_eq!(results[0].id, "qp-3");
        assert!((results[0].score - 2.0 / 3.0).abs() < 1e-9);

        // qp-2 and qp-4 should have equal scores (each Jaccard = 1/3 = 0.333...)
        assert!(results
            .iter()
            .any(|r| r.id == "qp-2" && (r.score - 1.0 / 3.0).abs() < 1e-9));
        assert!(results
            .iter()
            .any(|r| r.id == "qp-4" && (r.score - 1.0 / 3.0).abs() < 1e-9));

        // qp-5 should not be in results (no shared tags)
        assert!(!results.iter().any(|r| r.id == "qp-5"));
    }

    #[test]
    fn test_find_by_2hop_neighborhood() {
        let mut index = Index::new();

        // Create a graph:
        // qp-1 -> qp-2 -> qp-3
        // qp-1 -> qp-4 -> qp-3
        // qp-5 is isolated

        index.edges.push(crate::lib::index::types::Edge {
            from: "qp-1".to_string(),
            to: "qp-2".to_string(),
            link_type: crate::lib::note::LinkType::from("related"),
            source: crate::lib::index::types::LinkSource::Inline,
        });
        index.edges.push(crate::lib::index::types::Edge {
            from: "qp-2".to_string(),
            to: "qp-3".to_string(),
            link_type: crate::lib::note::LinkType::from("related"),
            source: crate::lib::index::types::LinkSource::Inline,
        });
        index.edges.push(crate::lib::index::types::Edge {
            from: "qp-1".to_string(),
            to: "qp-4".to_string(),
            link_type: crate::lib::note::LinkType::from("related"),
            source: crate::lib::index::types::LinkSource::Inline,
        });
        index.edges.push(crate::lib::index::types::Edge {
            from: "qp-4".to_string(),
            to: "qp-3".to_string(),
            link_type: crate::lib::note::LinkType::from("related"),
            source: crate::lib::index::types::LinkSource::Inline,
        });

        // Add metadata for all notes
        for id in ["qp-1", "qp-2", "qp-3", "qp-4", "qp-5"] {
            index.metadata.insert(
                id.to_string(),
                NoteMetadata {
                    id: id.to_string(),
                    title: format!("Note {}", id),
                    note_type: NoteType::Permanent,
                    tags: vec![],
                    path: format!("{}.md", id),
                    created: None,
                    updated: None,
                    value: None,
                },
            );
        }

        let engine = SimilarityEngine::new(&index);
        let results = engine.find_by_2hop_neighborhood("qp-1", 100);

        // qp-3 is 2 hops away (via qp-2 and via qp-4), so score = 2
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "qp-3");
        assert_eq!(results[0].score, 2.0);

        // qp-5 is isolated, should not be in results
        assert!(!results.iter().any(|r| r.id == "qp-5"));
    }

    #[test]
    fn test_field_weighting_title_vs_body() {
        let mut index = Index::new();

        // Note 1: "quantum" in title (2.0) + "computing" in body (1.0)
        let id1 = "qp-1".to_string();
        let mut term_freqs1 = HashMap::new();
        term_freqs1.insert("quantum".to_string(), 2.0); // Title weight
        term_freqs1.insert("computing".to_string(), 1.0); // Body weight
        index.note_terms.insert(id1.clone(), term_freqs1);
        index.doc_lengths.insert(id1.clone(), 2);
        index.metadata.insert(
            id1.clone(),
            NoteMetadata {
                id: id1.clone(),
                title: "Quantum Computing".to_string(),
                note_type: NoteType::Permanent,
                tags: vec![],
                path: "1.md".to_string(),
                created: None,
                updated: None,
                value: None,
            },
        );

        // Note 2: "quantum" in body (1.0) + "computing" in title (2.0)
        // This reverses the field placement, creating a different vector orientation
        let id2 = "qp-2".to_string();
        let mut term_freqs2 = HashMap::new();
        term_freqs2.insert("quantum".to_string(), 1.0); // Body weight
        term_freqs2.insert("computing".to_string(), 2.0); // Title weight
        index.note_terms.insert(id2.clone(), term_freqs2);
        index.doc_lengths.insert(id2.clone(), 2);
        index.metadata.insert(
            id2.clone(),
            NoteMetadata {
                id: id2.clone(),
                title: "Computing Systems".to_string(),
                note_type: NoteType::Permanent,
                tags: vec![],
                path: "2.md".to_string(),
                created: None,
                updated: None,
                value: None,
            },
        );

        // Note 3: Both terms with same weights as Note 1
        let id3 = "qp-3".to_string();
        let mut term_freqs3 = HashMap::new();
        term_freqs3.insert("quantum".to_string(), 2.0); // Title weight
        term_freqs3.insert("computing".to_string(), 1.0); // Body weight
        index.note_terms.insert(id3.clone(), term_freqs3);
        index.doc_lengths.insert(id3.clone(), 2);
        index.metadata.insert(
            id3.clone(),
            NoteMetadata {
                id: id3.clone(),
                title: "Quantum Mechanics".to_string(),
                note_type: NoteType::Permanent,
                tags: vec![],
                path: "3.md".to_string(),
                created: None,
                updated: None,
                value: None,
            },
        );

        // Stats
        index.total_docs = 3;
        index.total_len = 6;
        index.term_df.insert("quantum".to_string(), 3);
        index.term_df.insert("computing".to_string(), 3);

        let engine = SimilarityEngine::new(&index);

        // Note 1 vs Note 2: Different field distributions should produce < 1.0 similarity
        let score_1_2 = engine.calculate_similarity("qp-1", "qp-2");
        assert!(
            score_1_2 > 0.0 && score_1_2 < 1.0,
            "Different field distributions should have 0 < similarity < 1.0, got {}",
            score_1_2
        );

        // Note 1 vs Note 3: Identical field distributions should produce 1.0 similarity
        let score_1_3 = engine.calculate_similarity("qp-1", "qp-3");
        assert!(
            (score_1_3 - 1.0).abs() < 1e-9,
            "Identical field distributions should have similarity = 1.0, got {}",
            score_1_3
        );

        // Self-similarity should always be 1.0
        let self_score = engine.calculate_similarity("qp-1", "qp-1");
        assert!(
            (self_score - 1.0).abs() < 1e-9,
            "Self-similarity should be 1.0"
        );
    }

    #[test]
    fn test_field_weighting_tags_vs_body() {
        let mut index = Index::new();

        // Note 1: "rust" in tags (1.5) + "programming" in body (1.0)
        let id1 = "qp-1".to_string();
        let mut term_freqs1 = HashMap::new();
        term_freqs1.insert("rust".to_string(), 1.5); // Tags weight
        term_freqs1.insert("programming".to_string(), 1.0); // Body weight
        index.note_terms.insert(id1.clone(), term_freqs1);
        index.doc_lengths.insert(id1.clone(), 2);
        index.metadata.insert(
            id1.clone(),
            NoteMetadata {
                id: id1.clone(),
                title: "Languages".to_string(),
                note_type: NoteType::Permanent,
                tags: vec!["rust".to_string()],
                path: "1.md".to_string(),
                created: None,
                updated: None,
                value: None,
            },
        );

        // Note 2: "rust" in body (1.0) + "programming" in tags (1.5)
        // This reverses the field placement
        let id2 = "qp-2".to_string();
        let mut term_freqs2 = HashMap::new();
        term_freqs2.insert("rust".to_string(), 1.0); // Body weight
        term_freqs2.insert("programming".to_string(), 1.5); // Tags weight
        index.note_terms.insert(id2.clone(), term_freqs2);
        index.doc_lengths.insert(id2.clone(), 2);
        index.metadata.insert(
            id2.clone(),
            NoteMetadata {
                id: id2.clone(),
                title: "Systems".to_string(),
                note_type: NoteType::Permanent,
                tags: vec!["programming".to_string()],
                path: "2.md".to_string(),
                created: None,
                updated: None,
                value: None,
            },
        );

        // Stats
        index.total_docs = 2;
        index.total_len = 4;
        index.term_df.insert("rust".to_string(), 2);
        index.term_df.insert("programming".to_string(), 2);

        let engine = SimilarityEngine::new(&index);

        // Different field distributions should produce < 1.0 similarity
        let score = engine.calculate_similarity("qp-1", "qp-2");
        assert!(
            score > 0.0 && score < 1.0,
            "Different field distributions (tags vs body) should have 0 < similarity < 1.0, got {}",
            score
        );
    }

    #[test]
    fn test_field_weighting_title_vs_tags() {
        let mut index = Index::new();

        // Note 1: "machine" in title (2.0) + "learning" in tags (1.5)
        let id1 = "qp-1".to_string();
        let mut term_freqs1 = HashMap::new();
        term_freqs1.insert("machine".to_string(), 2.0); // Title weight
        term_freqs1.insert("learning".to_string(), 1.5); // Tags weight
        index.note_terms.insert(id1.clone(), term_freqs1);
        index.doc_lengths.insert(id1.clone(), 2);
        index.metadata.insert(
            id1.clone(),
            NoteMetadata {
                id: id1.clone(),
                title: "Machine Learning".to_string(),
                note_type: NoteType::Permanent,
                tags: vec!["learning".to_string()],
                path: "1.md".to_string(),
                created: None,
                updated: None,
                value: None,
            },
        );

        // Note 2: "machine" in tags (1.5) + "learning" in title (2.0)
        // This reverses the field placement
        let id2 = "qp-2".to_string();
        let mut term_freqs2 = HashMap::new();
        term_freqs2.insert("machine".to_string(), 1.5); // Tags weight
        term_freqs2.insert("learning".to_string(), 2.0); // Title weight
        index.note_terms.insert(id2.clone(), term_freqs2);
        index.doc_lengths.insert(id2.clone(), 2);
        index.metadata.insert(
            id2.clone(),
            NoteMetadata {
                id: id2.clone(),
                title: "Learning Systems".to_string(),
                note_type: NoteType::Permanent,
                tags: vec!["machine".to_string()],
                path: "2.md".to_string(),
                created: None,
                updated: None,
                value: None,
            },
        );

        // Stats
        index.total_docs = 2;
        index.total_len = 4;
        index.term_df.insert("machine".to_string(), 2);
        index.term_df.insert("learning".to_string(), 2);

        let engine = SimilarityEngine::new(&index);

        // Different field distributions should produce < 1.0 similarity
        let score = engine.calculate_similarity("qp-1", "qp-2");
        assert!(
            score > 0.0 && score < 1.0,
            "Different field distributions (title vs tags) should have 0 < similarity < 1.0, got {}",
            score
        );
    }

    #[test]
    fn test_default_threshold_related_notes() {
        // Test the 0.3 threshold mentioned in spec for "related notes" (context expansion)
        let mut index = Index::new();

        // Create notes with varying degrees of similarity
        // Note 1: Base note with terms "machine", "learning", "algorithm"
        let id1 = "qp-1".to_string();
        let mut term_freqs1 = HashMap::new();
        term_freqs1.insert("machine".to_string(), 1.0);
        term_freqs1.insert("learning".to_string(), 1.0);
        term_freqs1.insert("algorithm".to_string(), 1.0);
        index.note_terms.insert(id1.clone(), term_freqs1);
        index.doc_lengths.insert(id1.clone(), 3);
        index.metadata.insert(
            id1.clone(),
            NoteMetadata {
                id: id1.clone(),
                title: "Machine Learning".to_string(),
                note_type: NoteType::Permanent,
                tags: vec![],
                path: "1.md".to_string(),
                created: None,
                updated: None,
                value: None,
            },
        );

        // Note 2: High similarity - shares 2 out of 3 terms
        let id2 = "qp-2".to_string();
        let mut term_freqs2 = HashMap::new();
        term_freqs2.insert("machine".to_string(), 1.0);
        term_freqs2.insert("learning".to_string(), 1.0);
        term_freqs2.insert("neural".to_string(), 1.0);
        index.note_terms.insert(id2.clone(), term_freqs2);
        index.doc_lengths.insert(id2.clone(), 3);
        index.metadata.insert(
            id2.clone(),
            NoteMetadata {
                id: id2.clone(),
                title: "Neural Networks".to_string(),
                note_type: NoteType::Permanent,
                tags: vec![],
                path: "2.md".to_string(),
                created: None,
                updated: None,
                value: None,
            },
        );

        // Note 3: Low similarity - shares only 1 out of 3 terms
        let id3 = "qp-3".to_string();
        let mut term_freqs3 = HashMap::new();
        term_freqs3.insert("machine".to_string(), 1.0);
        term_freqs3.insert("vision".to_string(), 1.0);
        term_freqs3.insert("image".to_string(), 1.0);
        index.note_terms.insert(id3.clone(), term_freqs3);
        index.doc_lengths.insert(id3.clone(), 3);
        index.metadata.insert(
            id3.clone(),
            NoteMetadata {
                id: id3.clone(),
                title: "Computer Vision".to_string(),
                note_type: NoteType::Permanent,
                tags: vec![],
                path: "3.md".to_string(),
                created: None,
                updated: None,
                value: None,
            },
        );

        // Stats
        index.total_docs = 3;
        index.total_len = 9;
        for term in [
            "machine",
            "learning",
            "algorithm",
            "neural",
            "vision",
            "image",
        ] {
            let df = match term {
                "machine" => 3,
                "learning" => 2,
                _ => 1,
            };
            index.term_df.insert(term.to_string(), df);
        }

        let engine = SimilarityEngine::new(&index);

        // Test find_similar with 0.3 threshold (spec default for related notes)
        let results = engine.find_similar("qp-1", 10, 0.3);

        // Should find qp-2 (high similarity > 0.3)
        assert!(
            results.iter().any(|r| r.id == "qp-2"),
            "Should find high-similarity note with threshold 0.3"
        );

        // Verify scores are in expected ranges
        for result in &results {
            assert!(
                result.score >= 0.3,
                "All results should have score >= 0.3, got {}",
                result.score
            );
        }
    }

    #[test]
    fn test_default_threshold_duplicates() {
        // Test the 0.85 threshold mentioned in spec for duplicate detection
        let mut index = Index::new();

        // Note 1: Original note
        let id1 = "qp-1".to_string();
        let mut term_freqs1 = HashMap::new();
        term_freqs1.insert("apple".to_string(), 1.0);
        term_freqs1.insert("banana".to_string(), 1.0);
        term_freqs1.insert("cherry".to_string(), 1.0);
        term_freqs1.insert("date".to_string(), 1.0);
        term_freqs1.insert("elderberry".to_string(), 1.0);
        index.note_terms.insert(id1.clone(), term_freqs1);
        index.doc_lengths.insert(id1.clone(), 5);
        index.metadata.insert(
            id1.clone(),
            NoteMetadata {
                id: id1.clone(),
                title: "Fruit List".to_string(),
                note_type: NoteType::Permanent,
                tags: vec![],
                path: "1.md".to_string(),
                created: None,
                updated: None,
                value: None,
            },
        );

        // Note 2: Near-duplicate (shares 5/5 terms = 100% identical)
        let id2 = "qp-2".to_string();
        let mut term_freqs2 = HashMap::new();
        term_freqs2.insert("apple".to_string(), 1.0);
        term_freqs2.insert("banana".to_string(), 1.0);
        term_freqs2.insert("cherry".to_string(), 1.0);
        term_freqs2.insert("date".to_string(), 1.0);
        term_freqs2.insert("elderberry".to_string(), 1.0);
        index.note_terms.insert(id2.clone(), term_freqs2);
        index.doc_lengths.insert(id2.clone(), 5);
        index.metadata.insert(
            id2.clone(),
            NoteMetadata {
                id: id2.clone(),
                title: "Fruit List Copy".to_string(),
                note_type: NoteType::Permanent,
                tags: vec![],
                path: "2.md".to_string(),
                created: None,
                updated: None,
                value: None,
            },
        );

        // Note 3: High similarity but not duplicate (shares 4/5 terms)
        let id3 = "qp-3".to_string();
        let mut term_freqs3 = HashMap::new();
        term_freqs3.insert("apple".to_string(), 1.0);
        term_freqs3.insert("banana".to_string(), 1.0);
        term_freqs3.insert("cherry".to_string(), 1.0);
        term_freqs3.insert("date".to_string(), 1.0);
        term_freqs3.insert("fig".to_string(), 1.0); // Different term
        index.note_terms.insert(id3.clone(), term_freqs3);
        index.doc_lengths.insert(id3.clone(), 5);
        index.metadata.insert(
            id3.clone(),
            NoteMetadata {
                id: id3.clone(),
                title: "Similar Fruits".to_string(),
                note_type: NoteType::Permanent,
                tags: vec![],
                path: "3.md".to_string(),
                created: None,
                updated: None,
                value: None,
            },
        );

        // Stats
        index.total_docs = 3;
        index.total_len = 15;
        for term in ["apple", "banana", "cherry", "date", "elderberry", "fig"] {
            let df = match term {
                "apple" | "banana" | "cherry" | "date" => 3,
                "elderberry" => 2,
                "fig" => 1,
                _ => 1,
            };
            index.term_df.insert(term.to_string(), df);
        }

        let engine = SimilarityEngine::new(&index);

        // Test find_all_duplicates with 0.85 threshold (spec default)
        let duplicates = engine.find_all_duplicates(0.85);

        // Should find qp-1 and qp-2 as duplicates (100% identical)
        assert!(
            duplicates
                .iter()
                .any(|(a, b, _)| (a == "qp-1" && b == "qp-2") || (a == "qp-2" && b == "qp-1")),
            "Should find identical notes as duplicates"
        );

        // Verify all duplicate scores are >= 0.85
        for (_, _, score) in &duplicates {
            assert!(
                *score >= 0.85,
                "All duplicates should have score >= 0.85, got {}",
                score
            );
        }

        // The identical pair (qp-1, qp-2) should have score very close to 1.0
        let identical_score = duplicates
            .iter()
            .find(|(a, b, _)| (a == "qp-1" && b == "qp-2") || (a == "qp-2" && b == "qp-1"))
            .map(|(_, _, s)| *s)
            .unwrap();
        assert!(
            (identical_score - 1.0).abs() < 0.01,
            "Identical notes should have similarity score very close to 1.0, got {}",
            identical_score
        );
    }

    #[test]
    fn test_field_weighting_combined() {
        // Test that field weights combine correctly when a term appears in multiple fields
        let mut index = Index::new();

        // Note 1: "system" in all three fields (title 2.0 + tags 1.5 + body 1.0 = 4.5)
        //         "design" only in body (1.0)
        let id1 = "qp-1".to_string();
        let mut term_freqs1 = HashMap::new();
        term_freqs1.insert("system".to_string(), 4.5);
        term_freqs1.insert("design".to_string(), 1.0);
        index.note_terms.insert(id1.clone(), term_freqs1);
        index.doc_lengths.insert(id1.clone(), 2);
        index.metadata.insert(
            id1.clone(),
            NoteMetadata {
                id: id1.clone(),
                title: "System Architecture".to_string(),
                note_type: NoteType::Permanent,
                tags: vec!["system".to_string()],
                path: "1.md".to_string(),
                created: None,
                updated: None,
                value: None,
            },
        );

        // Note 2: "system" only in title (2.0)
        //         "design" only in title (2.0)
        let id2 = "qp-2".to_string();
        let mut term_freqs2 = HashMap::new();
        term_freqs2.insert("system".to_string(), 2.0);
        term_freqs2.insert("design".to_string(), 2.0);
        index.note_terms.insert(id2.clone(), term_freqs2);
        index.doc_lengths.insert(id2.clone(), 2);
        index.metadata.insert(
            id2.clone(),
            NoteMetadata {
                id: id2.clone(),
                title: "System Design".to_string(),
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
        index.term_df.insert("system".to_string(), 2);
        index.term_df.insert("design".to_string(), 2);

        let engine = SimilarityEngine::new(&index);

        let score = engine.calculate_similarity("qp-1", "qp-2");

        // Different weight distributions should produce non-perfect similarity
        assert!(
            score > 0.0 && score < 1.0,
            "Combined field weights with different distributions should produce non-perfect similarity, got {}",
            score
        );

        // Self-similarity should always be 1.0
        let self_score_1 = engine.calculate_similarity("qp-1", "qp-1");
        assert!(
            (self_score_1 - 1.0).abs() < 1e-9,
            "Self-similarity should always be 1.0"
        );
    }
}
