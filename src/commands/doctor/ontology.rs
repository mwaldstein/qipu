use super::types::{DoctorResult, Issue, Severity};
use crate::lib::config::LinkTypeConfig;
use crate::lib::config::OntologyConfig;
use crate::lib::note::Note;
use crate::lib::ontology::Ontology;
use crate::lib::store::Store;

pub fn check_ontology(store: &Store, notes: &[Note], result: &mut DoctorResult) {
    let config = store.config();
    let ontology = Ontology::from_config_with_graph(&config.ontology, &config.graph);

    check_note_types(&ontology, notes, result);
    check_link_types(&ontology, notes, result);
    check_missing_usage_guidance(&config.ontology, result);
    check_deprecated_graph_types(&config.graph.types, result);
}

fn check_note_types(ontology: &Ontology, notes: &[Note], result: &mut DoctorResult) {
    for note in notes {
        let note_type = note.note_type();
        let note_type_str = note_type.as_str();

        if !ontology.is_valid_note_type(note_type_str) {
            result.add_issue(Issue {
                severity: Severity::Error,
                category: "invalid-note-type".to_string(),
                message: format!("Invalid note type '{}'", note_type_str),
                note_id: Some(note.id().to_string()),
                path: note.path.as_ref().map(|p| p.display().to_string()),
                fixable: false,
            });
        }
    }
}

fn check_link_types(ontology: &Ontology, notes: &[Note], result: &mut DoctorResult) {
    for note in notes {
        for link in &note.frontmatter.links {
            let link_type_str = link.link_type.as_str();

            if !ontology.is_valid_link_type(link_type_str) {
                result.add_issue(Issue {
                    severity: Severity::Error,
                    category: "invalid-link-type".to_string(),
                    message: format!(
                        "Invalid link type '{}' in note '{}' pointing to '{}'",
                        link_type_str,
                        note.id(),
                        link.id
                    ),
                    note_id: Some(note.id().to_string()),
                    path: note.path.as_ref().map(|p| p.display().to_string()),
                    fixable: false,
                });
            }
        }
    }
}

fn check_missing_usage_guidance(ontology_config: &OntologyConfig, result: &mut DoctorResult) {
    for (name, config) in &ontology_config.note_types {
        if config.usage.is_none() {
            result.add_issue(Issue {
                severity: Severity::Warning,
                category: "missing-usage-guidance".to_string(),
                message: format!(
                    "Custom note type '{}' missing usage guidance (add [ontology.note_types.{}.usage])",
                    name, name
                ),
                note_id: None,
                path: None,
                fixable: false,
            });
        }
    }

    for (name, config) in &ontology_config.link_types {
        if config.usage.is_none() {
            result.add_issue(Issue {
                severity: Severity::Warning,
                category: "missing-usage-guidance".to_string(),
                message: format!(
                    "Custom link type '{}' missing usage guidance (add [ontology.link_types.{}.usage])",
                    name, name
                ),
                note_id: None,
                path: None,
                fixable: false,
            });
        }
    }
}

fn check_deprecated_graph_types(
    graph_types: &std::collections::HashMap<String, LinkTypeConfig>,
    result: &mut DoctorResult,
) {
    if !graph_types.is_empty() {
        for name in graph_types.keys() {
            result.add_issue(Issue {
                severity: Severity::Warning,
                category: "deprecated-config".to_string(),
                message: format!(
                    "Deprecated configuration [graph.types.{}] - use [ontology.link_types.{}] instead",
                    name, name
                ),
                note_id: None,
                path: None,
                fixable: false,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::config::{LinkTypeConfig, NoteTypeConfig, OntologyConfig, OntologyMode};
    use crate::lib::note::{Note, NoteFrontmatter, TypedLink};
    use tempfile::tempdir;

    #[test]
    fn test_check_valid_note_types() {
        let dir = tempdir().unwrap();
        let store_path = dir.path().join(".qipu");
        std::fs::create_dir_all(&store_path).unwrap();

        let notes = vec![Note {
            frontmatter: NoteFrontmatter {
                id: "test1".to_string(),
                title: "Test Note 1".to_string(),
                note_type: Some(crate::lib::note::NoteType::from("fleeting")),
                ..NoteFrontmatter::new("test1".to_string(), "Test Note 1".to_string())
            },
            path: Some(store_path.join("notes/test1.md")),
            body: "Test content".to_string(),
        }];

        let mut result = DoctorResult::new();
        let ontology = Ontology::from_config(&OntologyConfig::default());
        check_note_types(&ontology, &notes, &mut result);

        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_check_invalid_note_type() {
        let dir = tempdir().unwrap();
        let store_path = dir.path().join(".qipu");
        std::fs::create_dir_all(&store_path).unwrap();

        let notes = vec![Note {
            frontmatter: NoteFrontmatter {
                id: "test1".to_string(),
                title: "Test Note 1".to_string(),
                note_type: Some(crate::lib::note::NoteType::from("invalid-type")),
                ..NoteFrontmatter::new("test1".to_string(), "Test Note 1".to_string())
            },
            path: Some(store_path.join("notes/test1.md")),
            body: "Test content".to_string(),
        }];

        let mut result = DoctorResult::new();
        let ontology = Ontology::from_config(&OntologyConfig::default());
        check_note_types(&ontology, &notes, &mut result);

        assert_eq!(result.error_count, 1);
        assert_eq!(result.issues[0].category, "invalid-note-type");
    }

    #[test]
    fn test_check_valid_link_types() {
        let dir = tempdir().unwrap();
        let store_path = dir.path().join(".qipu");
        std::fs::create_dir_all(&store_path).unwrap();

        let notes = vec![Note {
            frontmatter: NoteFrontmatter {
                id: "test1".to_string(),
                title: "Test Note 1".to_string(),
                note_type: Some(crate::lib::note::NoteType::from("fleeting")),
                links: vec![TypedLink {
                    id: "test2".to_string(),
                    link_type: crate::lib::note::LinkType::from("related"),
                }],
                ..NoteFrontmatter::new("test1".to_string(), "Test Note 1".to_string())
            },
            path: Some(store_path.join("notes/test1.md")),
            body: "Test content".to_string(),
        }];

        let mut result = DoctorResult::new();
        let ontology = Ontology::from_config(&OntologyConfig::default());
        check_link_types(&ontology, &notes, &mut result);

        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_check_invalid_link_type() {
        let dir = tempdir().unwrap();
        let store_path = dir.path().join(".qipu");
        std::fs::create_dir_all(&store_path).unwrap();

        let notes = vec![Note {
            frontmatter: NoteFrontmatter {
                id: "test1".to_string(),
                title: "Test Note 1".to_string(),
                note_type: Some(crate::lib::note::NoteType::from("fleeting")),
                links: vec![TypedLink {
                    id: "test2".to_string(),
                    link_type: crate::lib::note::LinkType::from("invalid-type"),
                }],
                ..NoteFrontmatter::new("test1".to_string(), "Test Note 1".to_string())
            },
            path: Some(store_path.join("notes/test1.md")),
            body: "Test content".to_string(),
        }];

        let mut result = DoctorResult::new();
        let ontology = Ontology::from_config(&OntologyConfig::default());
        check_link_types(&ontology, &notes, &mut result);

        assert_eq!(result.error_count, 1);
        assert_eq!(result.issues[0].category, "invalid-link-type");
    }

    #[test]
    fn test_check_missing_usage_guidance() {
        let mut result = DoctorResult::new();

        let ontology_config = OntologyConfig {
            mode: OntologyMode::Extended,
            note_types: [(
                "custom-type".to_string(),
                NoteTypeConfig {
                    description: Some("A custom type".to_string()),
                    usage: None,
                },
            )]
            .into_iter()
            .collect(),
            link_types: [(
                "custom-link".to_string(),
                LinkTypeConfig {
                    inverse: None,
                    description: Some("A custom link".to_string()),
                    cost: 1.0,
                    usage: None,
                },
            )]
            .into_iter()
            .collect(),
        };

        check_missing_usage_guidance(&ontology_config, &mut result);

        assert_eq!(result.warning_count, 2);
        assert!(result.issues.iter().any(|i| i
            .message
            .contains("Custom note type 'custom-type' missing usage guidance")));
        assert!(result.issues.iter().any(|i| i
            .message
            .contains("Custom link type 'custom-link' missing usage guidance")));
    }

    #[test]
    fn test_check_missing_usage_guidance_with_usage() {
        let mut result = DoctorResult::new();

        let ontology_config = OntologyConfig {
            mode: OntologyMode::Extended,
            note_types: [(
                "custom-type".to_string(),
                NoteTypeConfig {
                    description: Some("A custom type".to_string()),
                    usage: Some("Use for X".to_string()),
                },
            )]
            .into_iter()
            .collect(),
            link_types: [(
                "custom-link".to_string(),
                LinkTypeConfig {
                    inverse: None,
                    description: Some("A custom link".to_string()),
                    cost: 1.0,
                    usage: Some("Use for Y".to_string()),
                },
            )]
            .into_iter()
            .collect(),
        };

        check_missing_usage_guidance(&ontology_config, &mut result);

        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_check_deprecated_graph_types() {
        let mut result = DoctorResult::new();

        let graph_types = [("custom-link".to_string(), LinkTypeConfig::default())]
            .into_iter()
            .collect();

        check_deprecated_graph_types(&graph_types, &mut result);

        assert_eq!(result.warning_count, 1);
        assert_eq!(result.issues[0].category, "deprecated-config");
        assert!(result.issues[0]
            .message
            .contains("[graph.types.custom-link]"));
    }

    #[test]
    fn test_check_deprecated_graph_types_empty() {
        let mut result = DoctorResult::new();

        let graph_types: std::collections::HashMap<String, LinkTypeConfig> =
            std::collections::HashMap::new();

        check_deprecated_graph_types(&graph_types, &mut result);

        assert_eq!(result.warning_count, 0);
    }
}
