use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub avg_title_length: f64,
    pub avg_body_length: f64,
    pub avg_tags_per_note: f64,
    pub notes_without_tags: usize,
    pub links_per_note: f64,
    pub orphan_notes: usize,
    pub link_type_diversity: usize,
    pub type_distribution: HashMap<String, usize>,
    pub total_notes: usize,
    pub total_links: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExportNote {
    id: String,
    title: String,
    #[serde(rename = "type")]
    note_type: String,
    tags: Vec<String>,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExportData {
    notes: Vec<ExportNote>,
}

pub struct StoreAnalyzer;

impl StoreAnalyzer {
    pub fn analyze(export_json: &str) -> Result<QualityMetrics> {
        let data: ExportData = serde_json::from_str(export_json)?;
        Self::compute_metrics(&data.notes)
    }

    fn compute_metrics(notes: &[ExportNote]) -> Result<QualityMetrics> {
        if notes.is_empty() {
            return Ok(QualityMetrics {
                avg_title_length: 0.0,
                avg_body_length: 0.0,
                avg_tags_per_note: 0.0,
                notes_without_tags: 0,
                links_per_note: 0.0,
                orphan_notes: 0,
                link_type_diversity: 0,
                type_distribution: HashMap::new(),
                total_notes: 0,
                total_links: 0,
            });
        }

        let total_notes = notes.len();
        let mut total_title_length = 0;
        let mut total_body_length = 0;
        let mut total_tags = 0;
        let mut notes_without_tags = 0;
        let mut type_distribution = HashMap::new();
        let mut link_map = HashMap::new();

        let link_regex = Regex::new(r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]").unwrap();

        for note in notes {
            total_title_length += note.title.len();
            total_body_length += note.content.len();

            let tag_count = note.tags.len();
            total_tags += tag_count;
            if tag_count == 0 {
                notes_without_tags += 1;
            }

            *type_distribution.entry(note.note_type.clone()).or_insert(0) += 1;

            let from_id = note.id.clone();
            if !link_map.contains_key(&from_id) {
                link_map.insert(from_id.clone(), Vec::new());
            }

            for caps in link_regex.captures_iter(&note.content) {
                let target_id = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
                if !target_id.is_empty() {
                    link_map
                        .get_mut(&from_id)
                        .unwrap()
                        .push(target_id.to_string());
                }
            }
        }

        let total_links: usize = link_map.values().map(|v| v.len()).sum();
        let avg_title_length = total_title_length as f64 / total_notes as f64;
        let avg_body_length = total_body_length as f64 / total_notes as f64;
        let avg_tags_per_note = total_tags as f64 / total_notes as f64;
        let links_per_note = total_links as f64 / total_notes as f64;

        let orphan_notes = notes
            .iter()
            .filter(|note| {
                let id = &note.id;
                link_map.get(id).map_or(true, |v| v.is_empty())
            })
            .count();

        let link_type_diversity = 1;

        Ok(QualityMetrics {
            avg_title_length,
            avg_body_length,
            avg_tags_per_note,
            notes_without_tags,
            links_per_note,
            orphan_notes,
            link_type_diversity,
            type_distribution,
            total_notes,
            total_links,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_export_json(notes: Vec<ExportNote>) -> String {
        let data = ExportData { notes };
        serde_json::to_string(&data).unwrap()
    }

    fn make_note(
        id: &str,
        title: &str,
        note_type: &str,
        tags: Vec<&str>,
        content: &str,
    ) -> ExportNote {
        ExportNote {
            id: id.to_string(),
            title: title.to_string(),
            note_type: note_type.to_string(),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            content: content.to_string(),
        }
    }

    #[test]
    fn test_analyze_empty_export() {
        let json = create_export_json(vec![]);
        let metrics = StoreAnalyzer::analyze(&json).unwrap();

        assert_eq!(metrics.total_notes, 0);
        assert_eq!(metrics.total_links, 0);
        assert_eq!(metrics.avg_title_length, 0.0);
        assert_eq!(metrics.avg_body_length, 0.0);
        assert_eq!(metrics.avg_tags_per_note, 0.0);
        assert_eq!(metrics.notes_without_tags, 0);
        assert_eq!(metrics.links_per_note, 0.0);
        assert_eq!(metrics.orphan_notes, 0);
        assert!(metrics.type_distribution.is_empty());
    }

    #[test]
    fn test_analyze_single_note() {
        let notes = vec![make_note(
            "qp-001-test",
            "Test Note",
            "fleeting",
            vec!["test"],
            "This is content",
        )];
        let json = create_export_json(notes);
        let metrics = StoreAnalyzer::analyze(&json).unwrap();

        assert_eq!(metrics.total_notes, 1);
        assert_eq!(metrics.avg_title_length, 9.0);
        assert_eq!(metrics.avg_body_length, 15.0);
        assert_eq!(metrics.avg_tags_per_note, 1.0);
        assert_eq!(metrics.notes_without_tags, 0);
        assert_eq!(metrics.total_links, 0);
        assert_eq!(metrics.links_per_note, 0.0);
    }

    #[test]
    fn test_orphan_notes() {
        let notes = vec![
            make_note("qp-001", "Note 1", "fleeting", vec![], "Link to [[qp-002]]"),
            make_note("qp-002", "Note 2", "fleeting", vec![], "Linked from qp-001"),
            make_note(
                "qp-003",
                "Note 3",
                "fleeting",
                vec![],
                "No links, not linked to",
            ),
        ];
        let json = create_export_json(notes);
        let metrics = StoreAnalyzer::analyze(&json).unwrap();

        assert_eq!(metrics.total_links, 1);
        assert_eq!(metrics.orphan_notes, 2);
    }

    #[test]
    fn test_type_distribution() {
        let notes = vec![
            make_note("qp-001", "Note 1", "fleeting", vec![], "Content"),
            make_note("qp-002", "Note 2", "fleeting", vec![], "Content"),
            make_note("qp-003", "Note 3", "permanent", vec![], "Content"),
            make_note("qp-004", "Note 4", "literature", vec![], "Content"),
        ];
        let json = create_export_json(notes);
        let metrics = StoreAnalyzer::analyze(&json).unwrap();

        assert_eq!(metrics.type_distribution.get("fleeting"), Some(&2));
        assert_eq!(metrics.type_distribution.get("permanent"), Some(&1));
        assert_eq!(metrics.type_distribution.get("literature"), Some(&1));
    }

    #[test]
    fn test_links_from_wiki_syntax() {
        let notes = vec![
            make_note("qp-001", "Note 1", "fleeting", vec![], "Link to [[qp-002]]"),
            make_note(
                "qp-002",
                "Note 2",
                "fleeting",
                vec![],
                "Link to [[qp-001]] and [[qp-003]]",
            ),
            make_note("qp-003", "Note 3", "fleeting", vec![], "No links"),
        ];
        let json = create_export_json(notes);
        let metrics = StoreAnalyzer::analyze(&json).unwrap();

        assert_eq!(metrics.total_links, 3);
        assert_eq!(metrics.links_per_note, 1.0);
    }

    #[test]
    fn test_link_with_special_characters() {
        let notes = vec![make_note(
            "qp-001",
            "Note 1",
            "fleeting",
            vec![],
            "Link to [[qp-002_test-special.chars]]",
        )];
        let json = create_export_json(notes);
        let metrics = StoreAnalyzer::analyze(&json).unwrap();

        assert_eq!(metrics.total_links, 1);
    }

    #[test]
    fn test_link_with_display_text() {
        let notes = vec![make_note(
            "qp-001",
            "Note 1",
            "fleeting",
            vec![],
            "Link to [[qp-002|display text here]]",
        )];
        let json = create_export_json(notes);
        let metrics = StoreAnalyzer::analyze(&json).unwrap();

        assert_eq!(metrics.total_links, 1);
    }

    #[test]
    fn test_empty_link_target() {
        let notes = vec![make_note(
            "qp-001",
            "Note 1",
            "fleeting",
            vec![],
            "Link to [[]]",
        )];
        let json = create_export_json(notes);
        let metrics = StoreAnalyzer::analyze(&json).unwrap();

        assert_eq!(metrics.total_links, 0);
    }

    #[test]
    fn test_link_with_only_display_text() {
        let notes = vec![make_note(
            "qp-001",
            "Note 1",
            "fleeting",
            vec![],
            "Link to [[|only display text]]",
        )];
        let json = create_export_json(notes);
        let metrics = StoreAnalyzer::analyze(&json).unwrap();

        assert_eq!(metrics.total_links, 0);
    }

    #[test]
    fn test_malformed_unbalanced_brackets() {
        let notes = vec![make_note(
            "qp-001",
            "Note 1",
            "fleeting",
            vec![],
            "Link to [[qp-002 and [[qp-003 and [[qp-004",
        )];
        let json = create_export_json(notes);
        let metrics = StoreAnalyzer::analyze(&json).unwrap();

        assert_eq!(metrics.total_links, 0);
    }

    #[test]
    fn test_link_with_multiple_pipes() {
        let notes = vec![make_note(
            "qp-001",
            "Note 1",
            "fleeting",
            vec![],
            "Link to [[qp-002|first|second|third]]",
        )];
        let json = create_export_json(notes);
        let metrics = StoreAnalyzer::analyze(&json).unwrap();

        assert_eq!(metrics.total_links, 1);
    }

    #[test]
    fn test_links_with_whitespace_variations() {
        let notes = vec![make_note(
            "qp-001",
            "Note 1",
            "fleeting",
            vec![],
            "[[  qp-002  ]] and [[qp-003| display text  ]]",
        )];
        let json = create_export_json(notes);
        let metrics = StoreAnalyzer::analyze(&json).unwrap();

        assert_eq!(metrics.total_links, 2);
    }

    #[test]
    fn test_link_with_unicode_characters() {
        let notes = vec![make_note(
            "qp-001",
            "Note 1",
            "fleeting",
            vec![],
            "Link to [[qp-002-日本語-тест]]",
        )];
        let json = create_export_json(notes);
        let metrics = StoreAnalyzer::analyze(&json).unwrap();

        assert_eq!(metrics.total_links, 1);
    }

    #[test]
    fn test_links_at_various_positions() {
        let notes = vec![make_note(
            "qp-001",
            "Note 1",
            "fleeting",
            vec![],
            "[[qp-002]] at start, middle [[qp-003]] and end [[qp-004]]",
        )];
        let json = create_export_json(notes);
        let metrics = StoreAnalyzer::analyze(&json).unwrap();

        assert_eq!(metrics.total_links, 3);
    }

    #[test]
    fn test_multiple_consecutive_links() {
        let notes = vec![make_note(
            "qp-001",
            "Note 1",
            "fleeting",
            vec![],
            "[[qp-002]][[qp-003]][[qp-004]]",
        )];
        let json = create_export_json(notes);
        let metrics = StoreAnalyzer::analyze(&json).unwrap();

        assert_eq!(metrics.total_links, 3);
    }

    #[test]
    fn test_link_with_newlines() {
        let notes = vec![make_note(
            "qp-001",
            "Note 1",
            "fleeting",
            vec![],
            "Link to [[qp-\n002]]",
        )];
        let json = create_export_json(notes);
        let metrics = StoreAnalyzer::analyze(&json).unwrap();

        assert_eq!(metrics.total_links, 1);
    }

    #[test]
    fn test_links_with_pipe_in_display_text() {
        let notes = vec![make_note(
            "qp-001",
            "Note 1",
            "fleeting",
            vec![],
            "[[qp-002|A | B | C]]",
        )];
        let json = create_export_json(notes);
        let metrics = StoreAnalyzer::analyze(&json).unwrap();

        assert_eq!(metrics.total_links, 1);
    }

    #[test]
    fn test_duplicate_links_same_target() {
        let notes = vec![make_note(
            "qp-001",
            "Note 1",
            "fleeting",
            vec![],
            "Link to [[qp-002]] and again [[qp-002]]",
        )];
        let json = create_export_json(notes);
        let metrics = StoreAnalyzer::analyze(&json).unwrap();

        assert_eq!(metrics.total_links, 2);
    }

    #[test]
    fn test_link_ending_with_bracket_in_content() {
        let notes = vec![make_note(
            "qp-001",
            "Note 1",
            "fleeting",
            vec![],
            "Array syntax: [1, 2, 3] and link [[qp-002]",
        )];
        let json = create_export_json(notes);
        let metrics = StoreAnalyzer::analyze(&json).unwrap();

        assert_eq!(metrics.total_links, 0);
    }
}
