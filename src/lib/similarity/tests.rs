#[cfg(test)]
mod tests {
    use crate::lib::index::types::Index;
    use crate::lib::index::types::NoteMetadata;
    use crate::lib::note::NoteType;
    use crate::lib::similarity::SimilarityEngine;
    use std::collections::HashMap;

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
