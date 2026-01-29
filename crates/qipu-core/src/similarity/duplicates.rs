use crate::index::types::Index;
use crate::similarity::tfidf;

/// Find all near-duplicates in the store
///
/// Returns a vector of tuples (note_id_a, note_id_b, similarity_score) for
/// all pairs of notes with similarity >= threshold. Results are sorted by
/// similarity score in descending order.
pub fn find_all_duplicates(index: &Index, threshold: f64) -> Vec<(String, String, f64)> {
    let mut duplicates = Vec::new();
    let ids: Vec<_> = index.metadata.keys().cloned().collect();

    for i in 0..ids.len() {
        for j in i + 1..ids.len() {
            let score = calculate_similarity(index, &ids[i], &ids[j]);
            if score >= threshold {
                duplicates.push((ids[i].clone(), ids[j].clone(), score));
            }
        }
    }

    duplicates.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
    duplicates
}

/// Calculate cosine similarity between two notes using TF-IDF vectors
fn calculate_similarity(index: &Index, note_id_a: &str, note_id_b: &str) -> f64 {
    let term_freqs_a = match index.note_terms.get(note_id_a) {
        Some(t) => t,
        None => return 0.0,
    };
    let term_freqs_b = match index.note_terms.get(note_id_b) {
        Some(t) => t,
        None => return 0.0,
    };

    if term_freqs_a.is_empty() || term_freqs_b.is_empty() {
        return 0.0;
    }

    let vec_a = tfidf::get_tfidf_vector(index, term_freqs_a);
    let vec_b = tfidf::get_tfidf_vector(index, term_freqs_b);

    tfidf::cosine_similarity(&vec_a, &vec_b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::types::NoteMetadata;
    use crate::note::NoteType;
    use std::collections::HashMap;

    fn create_test_index() -> Index {
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
            id1,
            NoteMetadata {
                id: "qp-1".to_string(),
                title: "Fruit List".to_string(),
                note_type: NoteType::from(NoteType::PERMANENT),
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
            id2,
            NoteMetadata {
                id: "qp-2".to_string(),
                title: "Fruit List Copy".to_string(),
                note_type: NoteType::from(NoteType::PERMANENT),
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
        term_freqs3.insert("fig".to_string(), 1.0);
        index.note_terms.insert(id3.clone(), term_freqs3);
        index.doc_lengths.insert(id3.clone(), 5);
        index.metadata.insert(
            id3,
            NoteMetadata {
                id: "qp-3".to_string(),
                title: "Similar Fruits".to_string(),
                note_type: NoteType::from(NoteType::PERMANENT),
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

        index
    }

    #[test]
    fn test_default_threshold_duplicates() {
        // Test the 0.85 threshold mentioned in spec for duplicate detection
        let index = create_test_index();

        // Test find_all_duplicates with 0.85 threshold (spec default)
        let duplicates = find_all_duplicates(&index, 0.85);

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
}
