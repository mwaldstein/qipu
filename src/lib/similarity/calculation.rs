use crate::lib::index::types::Index;
use crate::lib::similarity::tfidf;

/// Calculate cosine similarity between two notes using TF-IDF vectors
///
/// This function:
/// 1. Looks up the term frequency maps for both notes
/// 2. Converts each to TF-IDF vectors using the index statistics
/// 3. Computes the cosine similarity between the vectors
/// 4. Returns a score between 0.0 (no similarity) and 1.0 (identical)
///
/// Field weighting (title: 2.0, tags: 1.5, body: 1.0) is already applied
/// during indexing, so this function simply uses the pre-weighted term frequencies.
pub fn calculate_similarity(index: &Index, note_id_a: &str, note_id_b: &str) -> f64 {
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
    use crate::lib::index::types::NoteMetadata;
    use crate::lib::note::NoteType;
    use std::collections::HashMap;

    fn create_test_index() -> Index {
        let mut index = Index::new();

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
                note_type: NoteType::from(NoteType::PERMANENT),
                tags: vec![],
                path: "1.md".to_string(),
                created: None,
                updated: None,
                value: None,
            },
        );

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
                note_type: NoteType::from(NoteType::PERMANENT),
                tags: vec![],
                path: "2.md".to_string(),
                created: None,
                updated: None,
                value: None,
            },
        );

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
    fn test_basic_similarity() {
        let index = create_test_index();

        let score = calculate_similarity(&index, "qp-1", "qp-2");
        assert!(score > 0.0);
        assert!(score < 1.0);

        let self_score = calculate_similarity(&index, "qp-1", "qp-1");
        assert!((self_score - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_missing_notes() {
        let index = create_test_index();

        let score = calculate_similarity(&index, "qp-1", "nonexistent");
        assert_eq!(score, 0.0);

        let score = calculate_similarity(&index, "nonexistent", "qp-1");
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_empty_term_vectors() {
        let mut index = Index::new();

        index.note_terms.insert("qp-1".to_string(), HashMap::new());
        index.doc_lengths.insert("qp-1".to_string(), 0);
        index.metadata.insert(
            "qp-1".to_string(),
            NoteMetadata {
                id: "qp-1".to_string(),
                title: "Empty Note".to_string(),
                note_type: NoteType::from(NoteType::PERMANENT),
                tags: vec![],
                path: "1.md".to_string(),
                created: None,
                updated: None,
                value: None,
            },
        );
        index.note_terms.insert("qp-2".to_string(), HashMap::new());
        index.doc_lengths.insert("qp-2".to_string(), 0);
        index.metadata.insert(
            "qp-2".to_string(),
            NoteMetadata {
                id: "qp-2".to_string(),
                title: "Empty Note 2".to_string(),
                note_type: NoteType::from(NoteType::PERMANENT),
                tags: vec![],
                path: "2.md".to_string(),
                created: None,
                updated: None,
                value: None,
            },
        );
        index.total_docs = 2;

        let score = calculate_similarity(&index, "qp-1", "qp-2");
        assert_eq!(score, 0.0);
    }
}
