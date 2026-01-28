use crate::lib::index::types::Index;
use crate::lib::similarity::SimilarityResult;

/// Find notes that share tags with the given note
///
/// Uses Jaccard similarity to rank notes by tag overlap:
/// - Score = intersection / union
/// - Returns notes sorted by score in descending order
pub fn find_by_shared_tags(index: &Index, note_id: &str, limit: usize) -> Vec<SimilarityResult> {
    let mut results = Vec::new();

    let tags = match index.get_metadata(note_id) {
        Some(meta) => &meta.tags,
        None => return results,
    };

    if tags.is_empty() {
        return results;
    }

    for (other_id, other_meta) in &index.metadata {
        if other_id == note_id {
            continue;
        }

        let shared_count = tags.iter().filter(|t| other_meta.tags.contains(t)).count();

        if shared_count > 0 {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::index::types::NoteMetadata;
    use crate::lib::note::NoteType;

    fn create_test_index() -> Index {
        let mut index = Index::new();

        index.metadata.insert(
            "qp-1".to_string(),
            NoteMetadata {
                id: "qp-1".to_string(),
                title: "Note 1".to_string(),
                note_type: NoteType::from(NoteType::PERMANENT),
                tags: vec!["rust".to_string(), "programming".to_string()],
                path: "1.md".to_string(),
                created: None,
                updated: None,
                value: None,
            },
        );

        index.metadata.insert(
            "qp-2".to_string(),
            NoteMetadata {
                id: "qp-2".to_string(),
                title: "Note 2".to_string(),
                note_type: NoteType::from(NoteType::PERMANENT),
                tags: vec!["rust".to_string(), "systems".to_string()],
                path: "2.md".to_string(),
                created: None,
                updated: None,
                value: None,
            },
        );

        index.metadata.insert(
            "qp-3".to_string(),
            NoteMetadata {
                id: "qp-3".to_string(),
                title: "Note3".to_string(),
                note_type: NoteType::from(NoteType::PERMANENT),
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

        index.metadata.insert(
            "qp-4".to_string(),
            NoteMetadata {
                id: "qp-4".to_string(),
                title: "Note 4".to_string(),
                note_type: NoteType::from(NoteType::PERMANENT),
                tags: vec!["python".to_string(), "programming".to_string()],
                path: "4.md".to_string(),
                created: None,
                updated: None,
                value: None,
            },
        );

        index.metadata.insert(
            "qp-5".to_string(),
            NoteMetadata {
                id: "qp-5".to_string(),
                title: "Note 5".to_string(),
                note_type: NoteType::from(NoteType::PERMANENT),
                tags: vec!["java".to_string()],
                path: "5.md".to_string(),
                created: None,
                updated: None,
                value: None,
            },
        );

        index
    }

    #[test]
    fn test_find_by_shared_tags() {
        let index = create_test_index();
        let results = find_by_shared_tags(&index, "qp-1", 100);

        assert_eq!(results.len(), 3);

        assert_eq!(results[0].id, "qp-3");
        assert!((results[0].score - 2.0 / 3.0).abs() < 1e-9);

        assert!(results
            .iter()
            .any(|r| r.id == "qp-2" && (r.score - 1.0 / 3.0).abs() < 1e-9));
        assert!(results
            .iter()
            .any(|r| r.id == "qp-4" && (r.score - 1.0 / 3.0).abs() < 1e-9));

        assert!(!results.iter().any(|r| r.id == "qp-5"));
    }
}
