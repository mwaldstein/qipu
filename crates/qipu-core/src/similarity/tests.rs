#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::index::types::Index;
    use crate::index::types::NoteMetadata;
    use crate::note::{Note, NoteFrontmatter, NoteType};
    use crate::similarity::SimilarityEngine;
    use crate::text::tokenize_with_stemming;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn test_similarity_with_stemming() {
        let note1 = create_note("qp-1", "Graphs Important", vec![], "graphs are important");
        let note2 = create_note("qp-2", "Graph Useful", vec![], "a graph is useful");

        let index = build_index_from_notes(&[note1, note2], true);
        let engine = SimilarityEngine::new(&index);
        let score = engine.calculate_similarity("qp-1", "qp-2");

        assert!(
            score > 0.0,
            "Similarity should be > 0 when sharing stemmed terms"
        );
    }

    #[test]
    fn test_default_threshold_related_notes() {
        let note1 = create_note("qp-1", "", vec![], "machine learning algorithm");
        let note2 = create_note("qp-2", "", vec![], "machine learning neural");
        let note3 = create_note("qp-3", "", vec![], "machine vision image");

        let index = build_index_from_notes(&[note1, note2, note3], false);
        let engine = SimilarityEngine::new(&index);

        let results = engine.find_similar("qp-1", 10, 0.3);

        assert!(
            results.iter().any(|r| r.id == "qp-2"),
            "Should find high-similarity note with threshold 0.3"
        );

        for result in &results {
            assert!(
                result.score >= 0.3,
                "All results should have score >= 0.3, got {}",
                result.score
            );
        }
    }

    #[test]
    fn test_field_weighting_tags_vs_body() {
        let note1 = create_note("qp-1", "Languages", vec!["rust"], "programming");
        let note2 = create_note("qp-2", "Systems", vec!["programming"], "rust");

        let index = build_index_from_notes(&[note1, note2], false);
        let engine = SimilarityEngine::new(&index);

        let score = engine.calculate_similarity("qp-1", "qp-2");
        assert!(
            score > 0.0 && score < 1.0,
            "Different field distributions (tags vs body) should have 0 < similarity < 1.0, got {}",
            score
        );
    }

    #[test]
    fn test_field_weighting_title_vs_tags() {
        let note1 = create_note("qp-1", "Machine Learning", vec!["learning"], "");
        let note2 = create_note("qp-2", "Learning Systems", vec!["machine"], "");

        let index = build_index_from_notes(&[note1, note2], false);
        let engine = SimilarityEngine::new(&index);

        let score = engine.calculate_similarity("qp-1", "qp-2");
        assert!(
            score > 0.0 && score < 1.0,
            "Different field distributions (title vs tags) should have 0 < similarity < 1.0, got {}",
            score
        );
    }

    #[test]
    fn test_field_weighting_title_vs_body() {
        let note1 = create_note("qp-1", "Quantum", vec![], "computing");
        let note2 = create_note("qp-2", "Computing", vec![], "quantum");
        let note3 = create_note("qp-3", "Quantum", vec![], "computing");

        let index = build_index_from_notes(&[note1, note2, note3], false);
        let engine = SimilarityEngine::new(&index);

        let score_1_2 = engine.calculate_similarity("qp-1", "qp-2");
        assert!(
            score_1_2 > 0.0 && score_1_2 < 1.0,
            "Different field distributions should have 0 < similarity < 1.0, got {}",
            score_1_2
        );

        let score_1_3 = engine.calculate_similarity("qp-1", "qp-3");
        assert!(
            (score_1_3 - 1.0).abs() < 1e-9,
            "Identical field distributions should have similarity = 1.0, got {}",
            score_1_3
        );

        let self_score = engine.calculate_similarity("qp-1", "qp-1");
        assert!(
            (self_score - 1.0).abs() < 1e-9,
            "Self-similarity should be 1.0"
        );
    }

    #[test]
    fn test_field_weighting_combined() {
        let note1 = create_note("qp-1", "System Architecture", vec!["system"], "design");
        let note2 = create_note("qp-2", "System Design", vec![], "");

        let index = build_index_from_notes(&[note1, note2], false);
        let engine = SimilarityEngine::new(&index);

        let score = engine.calculate_similarity("qp-1", "qp-2");

        assert!(
            score > 0.0 && score < 1.0,
            "Combined field weights with different distributions should produce non-perfect similarity, got {}",
            score
        );

        let self_score_1 = engine.calculate_similarity("qp-1", "qp-1");
        assert!(
            (self_score_1 - 1.0).abs() < 1e-9,
            "Self-similarity should always be 1.0"
        );
    }

    fn create_note(id: &str, title: &str, tags: Vec<&str>, body: &str) -> Note {
        let frontmatter = NoteFrontmatter::new(id.to_string(), title.to_string())
            .with_type(NoteType::from(NoteType::PERMANENT))
            .with_tags(tags);
        Note::new(frontmatter, body)
    }

    fn build_index_from_notes(notes: &[Note], use_stemming: bool) -> Index {
        let mut index = Index::new();
        let mut all_unique_terms: HashSet<String> = HashSet::new();

        for note in notes {
            let mut term_freqs: HashMap<String, f64> = HashMap::new();

            for term in tokenize_with_stemming(note.title(), use_stemming) {
                *term_freqs.entry(term).or_insert(0.0) += 2.0;
            }

            for tag in &note.frontmatter.tags {
                for term in tokenize_with_stemming(tag, use_stemming) {
                    *term_freqs.entry(term).or_insert(0.0) += 1.5;
                }
            }

            for term in tokenize_with_stemming(&note.body, use_stemming) {
                *term_freqs.entry(term).or_insert(0.0) += 1.0;
            }

            let word_count = term_freqs.values().map(|&f| f as usize).sum();
            let unique_terms: HashSet<String> = term_freqs.keys().cloned().collect();

            index.total_docs += 1;
            index.total_len += word_count;
            index.doc_lengths.insert(note.id().to_string(), word_count);

            for term in &unique_terms {
                *index.term_df.entry(term.clone()).or_insert(0) += 1;
                all_unique_terms.insert(term.clone());
            }
            index.note_terms.insert(note.id().to_string(), term_freqs);

            let meta = NoteMetadata {
                id: note.id().to_string(),
                title: note.title().to_string(),
                note_type: note.note_type(),
                tags: note.frontmatter.tags.clone(),
                path: format!("{}.md", note.id()),
                created: None,
                updated: None,
                value: None,
            };

            index.tags.insert(note.id().to_string(), meta.tags.clone());

            for tag in &meta.tags {
                index
                    .tags
                    .entry(tag.clone())
                    .or_default()
                    .push(meta.id.clone());
            }

            index.metadata.insert(meta.id.clone(), meta);
        }

        index
    }

    #[test]
    fn test_tfidf_weights_real_notes_field_weighting() {
        let note1 = create_note(
            "qp-1",
            "Machine Learning",
            vec!["ai", "algorithms"],
            "Neural networks and deep learning techniques",
        );

        let note2 = create_note(
            "qp-2",
            "Machine Learning",
            vec!["ai", "data"],
            "Statistical methods and regression analysis",
        );

        let note3 = create_note(
            "qp-3",
            "Cooking Recipes",
            vec!["food"],
            "Baking and cooking techniques",
        );

        let index = build_index_from_notes(&[note1, note2, note3], false);

        let engine = SimilarityEngine::new(&index);

        let score_1_2 = engine.calculate_similarity("qp-1", "qp-2");
        let score_1_3 = engine.calculate_similarity("qp-1", "qp-3");

        assert!(
            score_1_2 > score_1_3,
            "Notes with similar titles and tags should have higher similarity than unrelated notes: \
             score_1_2={}, score_1_3={}",
            score_1_2,
            score_1_3
        );

        assert!(
            score_1_2 > 0.0 && score_1_2 < 1.0,
            "Similar notes should have 0 < similarity < 1.0, got {}",
            score_1_2
        );
    }

    #[test]
    fn test_tfidf_weights_title_dominance() {
        let note1 = create_note(
            "qp-1",
            "Quantum Computing",
            vec![],
            "Basic information about computers",
        );

        let note2 = create_note(
            "qp-2",
            "Quantum Computing",
            vec![],
            "Advanced topics in technology",
        );

        let note3 = create_note(
            "qp-3",
            "Cooking Recipes",
            vec![],
            "Recipes and food preparation",
        );

        let index = build_index_from_notes(&[note1, note2, note3], false);

        let engine = SimilarityEngine::new(&index);

        let score_1_2 = engine.calculate_similarity("qp-1", "qp-2");
        let score_1_3 = engine.calculate_similarity("qp-1", "qp-3");

        assert!(
            score_1_2 > score_1_3,
            "Notes sharing title should have higher similarity due to title weight (2.0)"
        );

        let term_freqs_1 = index.note_terms.get("qp-1").unwrap();
        let term_freqs_2 = index.note_terms.get("qp-2").unwrap();

        assert_eq!(
            term_freqs_1.get("quantum").unwrap(),
            &2.0,
            "Title terms should have weight 2.0"
        );
        assert_eq!(
            term_freqs_2.get("quantum").unwrap(),
            &2.0,
            "Title terms should have weight 2.0"
        );
    }

    #[test]
    fn test_tfidf_weights_tag_weighting() {
        let note1 = create_note(
            "qp-1",
            "Programming",
            vec!["rust", "systems"],
            "About software",
        );

        let note2 = create_note(
            "qp-2",
            "Programming",
            vec!["rust", "data"],
            "About information",
        );

        let note3 = create_note("qp-3", "Cooking", vec!["food"], "About recipes");

        let index = build_index_from_notes(&[note1, note2, note3], false);

        let engine = SimilarityEngine::new(&index);

        let score_1_2 = engine.calculate_similarity("qp-1", "qp-2");
        let score_1_3 = engine.calculate_similarity("qp-1", "qp-3");

        assert!(
            score_1_2 > score_1_3,
            "Notes sharing tags should have higher similarity due to tag weight (1.5)"
        );

        let term_freqs_1 = index.note_terms.get("qp-1").unwrap();
        let term_freqs_2 = index.note_terms.get("qp-2").unwrap();

        assert_eq!(
            term_freqs_1.get("rust").unwrap(),
            &1.5,
            "Tag terms should have weight 1.5"
        );
        assert_eq!(
            term_freqs_2.get("rust").unwrap(),
            &1.5,
            "Tag terms should have weight 1.5"
        );
    }

    #[test]
    fn test_tfidf_weights_combined_fields() {
        let note1 = create_note(
            "qp-1",
            "System Design",
            vec!["architecture"],
            "Design patterns and software architecture",
        );

        let note2 = create_note(
            "qp-2",
            "System Design",
            vec!["architecture"],
            "Design principles and system architecture",
        );

        let note3 = create_note(
            "qp-3",
            "Cooking Food",
            vec![],
            "Food preparation and recipes",
        );

        let index = build_index_from_notes(&[note1, note2, note3], false);

        let engine = SimilarityEngine::new(&index);

        let score_1_2 = engine.calculate_similarity("qp-1", "qp-2");
        let score_1_3 = engine.calculate_similarity("qp-1", "qp-3");

        assert!(
            score_1_2 > score_1_3,
            "Notes sharing title, tags, and body terms should have highest similarity"
        );

        let term_freqs_1 = index.note_terms.get("qp-1").unwrap();

        assert_eq!(
            term_freqs_1.get("system").unwrap(),
            &2.0,
            "Title term 'system' should have weight 2.0"
        );
        assert_eq!(
            term_freqs_1.get("design").unwrap(),
            &3.0,
            "Combined weight for 'design': title (2.0) + body (1.0) = 3.0"
        );
        assert_eq!(
            term_freqs_1.get("architecture").unwrap(),
            &2.5,
            "Combined weight for 'architecture': tag (1.5) + body (1.0) = 2.5"
        );
    }

    #[test]
    fn test_tfidf_idf_rare_terms() {
        let note1 = create_note(
            "qp-1",
            "Zettelkasten Methods",
            vec![],
            "Using zettelkasten for knowledge management",
        );

        let note2 = create_note(
            "qp-2",
            "Zettelkasten Methods",
            vec![],
            "Zettelkasten ontology and linking",
        );

        let note3 = create_note(
            "qp-3",
            "Note Systems",
            vec![],
            "Taking notes and organizing information",
        );

        let index = build_index_from_notes(&[note1, note2, note3], false);

        let engine = SimilarityEngine::new(&index);

        let score_1_2 = engine.calculate_similarity("qp-1", "qp-2");

        assert!(
            score_1_2 > 0.0,
            "Notes sharing rare term 'zettelkasten' should have higher similarity due to IDF"
        );

        let df_zettelkasten = index.term_df.get("zettelkasten").unwrap();
        assert_eq!(
            *df_zettelkasten, 2,
            "Rare term 'zettelkasten' should appear in 2 documents"
        );

        let df_note = index.term_df.get("note").unwrap();
        assert_eq!(
            *df_note, 1,
            "Common term 'note' (from note1 body and note3 title) should have appropriate DF"
        );
    }

    #[test]
    fn test_tfidf_with_stemming() {
        let note1 = create_note(
            "qp-1",
            "Graph Theory",
            vec![],
            "Graphs and their properties",
        );

        let note2 = create_note(
            "qp-2",
            "Graph Analysis",
            vec![],
            "Analyzing graph data structures",
        );

        let note3 = create_note(
            "qp-3",
            "Tree Structures",
            vec![],
            "Trees and forest data structures",
        );

        let index = build_index_from_notes(&[note1, note2, note3], true);

        let engine = SimilarityEngine::new(&index);

        let score_1_2 = engine.calculate_similarity("qp-1", "qp-2");
        let score_1_3 = engine.calculate_similarity("qp-1", "qp-3");

        assert!(
            score_1_2 > score_1_3,
            "With stemming, 'graph' and 'graphs' should match, giving higher similarity"
        );

        let term_freqs_1 = index.note_terms.get("qp-1").unwrap();
        let term_freqs_2 = index.note_terms.get("qp-2").unwrap();

        assert!(
            term_freqs_1.contains_key("graph"),
            "Note 1 should contain stemmed term 'graph'"
        );
        assert!(
            term_freqs_2.contains_key("graph"),
            "Note 2 should contain stemmed term 'graph'"
        );
    }
}
