//! Ontology resolution for custom note and link types
//!
//! Provides resolution logic for merging custom types with the standard ontology
//! based on the configured resolution mode (default, extended, or replacement).

use std::collections::{HashMap, HashSet};

use crate::lib::config::{GraphConfig, OntologyConfig, OntologyMode};
use crate::lib::note::LinkType;

/// Standard note types (built-in)
#[allow(dead_code)]
const STANDARD_NOTE_TYPES: &[&str] = &["fleeting", "literature", "permanent", "moc"];

/// Standard link type inverses (built-in)
#[allow(dead_code)]
const STANDARD_LINK_INVERSES: &[(&str, &str)] = &[
    ("related", "related"),
    ("derived-from", "derived-to"),
    ("derived-to", "derived-from"),
    ("supports", "supported-by"),
    ("supported-by", "supports"),
    ("contradicts", "contradicted-by"),
    ("contradicted-by", "contradicts"),
    ("part-of", "has-part"),
    ("has-part", "part-of"),
    ("answers", "answered-by"),
    ("answered-by", "answers"),
    ("refines", "refined-by"),
    ("refined-by", "refines"),
    ("same-as", "same-as"),
    ("alias-of", "has-alias"),
    ("has-alias", "alias-of"),
    ("follows", "precedes"),
    ("precedes", "follows"),
];

/// Resolved ontology combining standard and custom types
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Ontology {
    /// All valid note types
    note_types: HashSet<String>,
    /// All valid link types
    link_types: HashSet<String>,
    /// Link type inverses (link_type -> inverse)
    inverses: HashMap<String, String>,
}

#[allow(dead_code)]
impl Ontology {
    /// Create an ontology from configuration, resolving based on mode
    pub fn from_config(config: &OntologyConfig) -> Self {
        match config.mode {
            OntologyMode::Default => Self::default_ontology(),
            OntologyMode::Extended => Self::extended_ontology(config),
            OntologyMode::Replacement => Self::replacement_ontology(config),
        }
    }

    /// Create an ontology from both ontology config and graph config
    /// Merges custom link types from both sources (graph.types for backward compatibility)
    pub fn from_config_with_graph(
        ontology_config: &OntologyConfig,
        graph_config: &GraphConfig,
    ) -> Self {
        let mut ontology = Self::from_config(ontology_config);

        // Merge link types from graph.types for backward compatibility
        for (name, type_config) in &graph_config.types {
            ontology.link_types.insert(name.clone());
            if let Some(ref inverse) = type_config.inverse {
                ontology.inverses.insert(name.clone(), inverse.clone());
            }
        }

        ontology
    }

    /// Default ontology: standard types only
    fn default_ontology() -> Self {
        let note_types: HashSet<String> =
            STANDARD_NOTE_TYPES.iter().map(|s| s.to_string()).collect();

        let link_types: HashSet<String> = STANDARD_LINK_INVERSES
            .iter()
            .flat_map(|(k, _)| vec![k.to_string()])
            .collect();

        let inverses: HashMap<String, String> = STANDARD_LINK_INVERSES
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        Ontology {
            note_types,
            link_types,
            inverses,
        }
    }

    /// Extended ontology: standard + custom types (custom can override standard inverses)
    fn extended_ontology(config: &OntologyConfig) -> Self {
        let mut note_types: HashSet<String> =
            STANDARD_NOTE_TYPES.iter().map(|s| s.to_string()).collect();

        let mut link_types: HashSet<String> = STANDARD_LINK_INVERSES
            .iter()
            .flat_map(|(k, _)| vec![k.to_string()])
            .collect();

        let mut inverses: HashMap<String, String> = STANDARD_LINK_INVERSES
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        for name in config.note_types.keys() {
            note_types.insert(name.clone());
        }

        for (name, type_config) in &config.link_types {
            link_types.insert(name.clone());
            if let Some(ref inverse) = type_config.inverse {
                inverses.insert(name.clone(), inverse.clone());
            }
        }

        Ontology {
            note_types,
            link_types,
            inverses,
        }
    }

    /// Replacement ontology: custom types only
    fn replacement_ontology(config: &OntologyConfig) -> Self {
        let note_types: HashSet<String> = config.note_types.keys().cloned().collect();

        let link_types: HashSet<String> = config.link_types.keys().cloned().collect();

        let mut inverses: HashMap<String, String> = HashMap::new();

        for (name, type_config) in &config.link_types {
            if let Some(ref inverse) = type_config.inverse {
                inverses.insert(name.clone(), inverse.clone());
            }
        }

        Ontology {
            note_types,
            link_types,
            inverses,
        }
    }

    /// Check if a note type is valid
    pub fn is_valid_note_type(&self, note_type: &str) -> bool {
        self.note_types.contains(note_type)
    }

    /// Check if a link type is valid
    pub fn is_valid_link_type(&self, link_type: &str) -> bool {
        self.link_types.contains(link_type)
    }

    /// Get the inverse of a link type
    /// Returns the link type itself if it's its own inverse
    /// For unknown types, returns inverse-<type>
    pub fn get_inverse(&self, link_type: &str) -> String {
        let lt = link_type.to_lowercase();

        if let Some(inverse) = self.inverses.get(&lt) {
            inverse.clone()
        } else {
            LinkType::new(&lt).inverse().to_string()
        }
    }

    /// Get all valid note types
    pub fn note_types(&self) -> Vec<String> {
        let mut types: Vec<String> = self.note_types.iter().cloned().collect();
        types.sort();
        types
    }

    /// Get all valid link types
    pub fn link_types(&self) -> Vec<String> {
        let mut types: Vec<String> = self.link_types.iter().cloned().collect();
        types.sort();
        types
    }

    /// Validate a link type
    pub fn validate_link_type(&self, link_type: &str) -> Result<(), crate::lib::error::QipuError> {
        if !self.is_valid_link_type(link_type) {
            return Err(crate::lib::error::QipuError::UsageError(format!(
                "Invalid link type: '{}'",
                link_type
            )));
        }
        Ok(())
    }

    /// Validate a note type
    pub fn validate_note_type(&self, note_type: &str) -> Result<(), crate::lib::error::QipuError> {
        if !self.is_valid_note_type(note_type) {
            let valid_types = self.note_types().join(", ");
            return Err(crate::lib::error::QipuError::UsageError(format!(
                "Invalid note type: '{}'. Valid types: {}",
                note_type, valid_types
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::config::{LinkTypeConfig, NoteTypeConfig};

    #[test]
    fn test_default_ontology_standard_note_types() {
        let config = OntologyConfig::default();
        let ontology = Ontology::from_config(&config);

        assert!(ontology.is_valid_note_type("fleeting"));
        assert!(ontology.is_valid_note_type("literature"));
        assert!(ontology.is_valid_note_type("permanent"));
        assert!(ontology.is_valid_note_type("moc"));
        assert!(!ontology.is_valid_note_type("custom-type"));
    }

    #[test]
    fn test_default_ontology_standard_link_types() {
        let config = OntologyConfig::default();
        let ontology = Ontology::from_config(&config);

        assert!(ontology.is_valid_link_type("related"));
        assert!(ontology.is_valid_link_type("supports"));
        assert!(ontology.is_valid_link_type("contradicts"));
        assert!(ontology.is_valid_link_type("part-of"));
        assert!(ontology.is_valid_link_type("answers"));
        assert!(ontology.is_valid_link_type("refines"));
        assert!(ontology.is_valid_link_type("same-as"));
        assert!(ontology.is_valid_link_type("alias-of"));
        assert!(ontology.is_valid_link_type("follows"));
        assert!(!ontology.is_valid_link_type("custom-link"));
    }

    #[test]
    fn test_default_ontology_inverse() {
        let config = OntologyConfig::default();
        let ontology = Ontology::from_config(&config);

        assert_eq!(ontology.get_inverse("related"), "related");
        assert_eq!(ontology.get_inverse("supports"), "supported-by");
        assert_eq!(ontology.get_inverse("contradicts"), "contradicted-by");
        assert_eq!(ontology.get_inverse("part-of"), "has-part");
        assert_eq!(ontology.get_inverse("answers"), "answered-by");
        assert_eq!(ontology.get_inverse("refines"), "refined-by");
        assert_eq!(ontology.get_inverse("same-as"), "same-as");
        assert_eq!(ontology.get_inverse("alias-of"), "has-alias");
        assert_eq!(ontology.get_inverse("follows"), "precedes");
    }

    #[test]
    fn test_extended_ontology_merges_custom_note_types() {
        let config = OntologyConfig {
            mode: OntologyMode::Extended,
            note_types: [("custom-type".to_string(), NoteTypeConfig::default())]
                .into_iter()
                .collect(),
            ..Default::default()
        };

        let ontology = Ontology::from_config(&config);

        assert!(ontology.is_valid_note_type("fleeting"));
        assert!(ontology.is_valid_note_type("custom-type"));
    }

    #[test]
    fn test_extended_ontology_merges_custom_link_types() {
        let config = OntologyConfig {
            mode: OntologyMode::Extended,
            link_types: [("custom-link".to_string(), LinkTypeConfig::default())]
                .into_iter()
                .collect(),
            ..Default::default()
        };

        let ontology = Ontology::from_config(&config);

        assert!(ontology.is_valid_link_type("related"));
        assert!(ontology.is_valid_link_type("custom-link"));
    }

    #[test]
    fn test_extended_ontology_custom_inverse() {
        let link_config = LinkTypeConfig {
            inverse: Some("custom-inverse".to_string()),
            ..Default::default()
        };
        let config = OntologyConfig {
            mode: OntologyMode::Extended,
            link_types: [("custom-link".to_string(), link_config)]
                .into_iter()
                .collect(),
            ..Default::default()
        };

        let ontology = Ontology::from_config(&config);

        assert_eq!(ontology.get_inverse("custom-link"), "custom-inverse");
    }

    #[test]
    fn test_extended_ontology_custom_inverse_overrides_standard() {
        let link_config = LinkTypeConfig {
            inverse: Some("my-inverse".to_string()),
            ..Default::default()
        };
        let config = OntologyConfig {
            mode: OntologyMode::Extended,
            link_types: [("supports".to_string(), link_config)]
                .into_iter()
                .collect(),
            ..Default::default()
        };

        let ontology = Ontology::from_config(&config);

        assert_eq!(ontology.get_inverse("supports"), "my-inverse");
    }

    #[test]
    fn test_replacement_ontology_only_custom_types() {
        let config = OntologyConfig {
            mode: OntologyMode::Replacement,
            note_types: [("custom-type".to_string(), NoteTypeConfig::default())]
                .into_iter()
                .collect(),
            link_types: [("custom-link".to_string(), LinkTypeConfig::default())]
                .into_iter()
                .collect(),
        };

        let ontology = Ontology::from_config(&config);

        assert!(ontology.is_valid_note_type("custom-type"));
        assert!(!ontology.is_valid_note_type("fleeting"));
        assert!(!ontology.is_valid_note_type("literature"));
        assert!(ontology.is_valid_link_type("custom-link"));
        assert!(!ontology.is_valid_link_type("related"));
    }

    #[test]
    fn test_replacement_ontology_empty_config() {
        let config = OntologyConfig {
            mode: OntologyMode::Replacement,
            ..Default::default()
        };

        let ontology = Ontology::from_config(&config);

        assert!(!ontology.is_valid_note_type("fleeting"));
        assert!(!ontology.is_valid_link_type("related"));
        assert!(ontology.note_types().is_empty());
    }

    #[test]
    fn test_inverse_unknown_type() {
        let config = OntologyConfig::default();
        let ontology = Ontology::from_config(&config);

        assert_eq!(ontology.get_inverse("unknown"), "inverse-unknown");
    }

    #[test]
    fn test_note_types_sorted() {
        let config = OntologyConfig::default();
        let ontology = Ontology::from_config(&config);

        let types = ontology.note_types();
        assert_eq!(types, vec!["fleeting", "literature", "moc", "permanent"]);
    }

    #[test]
    fn test_link_types_sorted() {
        let config = OntologyConfig::default();
        let ontology = Ontology::from_config(&config);

        let types = ontology.link_types();
        let expected = vec![
            "alias-of",
            "answered-by",
            "answers",
            "contradicted-by",
            "contradicts",
            "derived-from",
            "derived-to",
            "follows",
            "has-alias",
            "has-part",
            "part-of",
            "precedes",
            "refined-by",
            "refines",
            "related",
            "same-as",
            "supported-by",
            "supports",
        ];
        assert_eq!(types, expected);
    }

    #[test]
    fn test_partial_customization_only_note_types() {
        let config = OntologyConfig {
            mode: OntologyMode::Extended,
            note_types: [
                ("custom-note-1".to_string(), NoteTypeConfig::default()),
                ("custom-note-2".to_string(), NoteTypeConfig::default()),
            ]
            .into_iter()
            .collect(),
            ..Default::default()
        };

        let ontology = Ontology::from_config(&config);

        assert!(ontology.is_valid_note_type("fleeting"));
        assert!(ontology.is_valid_note_type("custom-note-1"));
        assert!(ontology.is_valid_note_type("custom-note-2"));
        assert!(!ontology.is_valid_note_type("nonexistent"));

        assert!(ontology.is_valid_link_type("related"));
        assert!(!ontology.is_valid_link_type("custom-link"));
    }

    #[test]
    fn test_partial_customization_only_link_types() {
        let link_config = LinkTypeConfig {
            inverse: Some("custom-inverse".to_string()),
            ..Default::default()
        };
        let config = OntologyConfig {
            mode: OntologyMode::Extended,
            link_types: [("custom-link".to_string(), link_config)]
                .into_iter()
                .collect(),
            ..Default::default()
        };

        let ontology = Ontology::from_config(&config);

        assert!(ontology.is_valid_note_type("fleeting"));
        assert!(!ontology.is_valid_note_type("custom-note"));

        assert!(ontology.is_valid_link_type("related"));
        assert!(ontology.is_valid_link_type("custom-link"));
        assert_eq!(ontology.get_inverse("custom-link"), "custom-inverse");
    }

    #[test]
    fn test_invalid_extend_missing_inverse_type() {
        let link_config = LinkTypeConfig {
            inverse: Some("nonexistent-inverse".to_string()),
            ..Default::default()
        };
        let config = OntologyConfig {
            mode: OntologyMode::Extended,
            link_types: [("custom-link".to_string(), link_config)]
                .into_iter()
                .collect(),
            ..Default::default()
        };

        let ontology = Ontology::from_config(&config);

        assert_eq!(ontology.get_inverse("custom-link"), "nonexistent-inverse");
        assert!(!ontology.is_valid_link_type("nonexistent-inverse"));
    }

    #[test]
    fn test_invalid_extend_self_referencing_inverse() {
        let link_config = LinkTypeConfig {
            inverse: Some("self-link".to_string()),
            ..Default::default()
        };
        let config = OntologyConfig {
            mode: OntologyMode::Extended,
            link_types: [("self-link".to_string(), link_config)]
                .into_iter()
                .collect(),
            ..Default::default()
        };

        let ontology = Ontology::from_config(&config);

        assert_eq!(ontology.get_inverse("self-link"), "self-link");
    }

    #[test]
    fn test_invalid_extend_circular_inverses() {
        let link_config_a = LinkTypeConfig {
            inverse: Some("link-b".to_string()),
            ..Default::default()
        };
        let link_config_b = LinkTypeConfig {
            inverse: Some("link-a".to_string()),
            ..Default::default()
        };
        let config = OntologyConfig {
            mode: OntologyMode::Extended,
            link_types: [
                ("link-a".to_string(), link_config_a),
                ("link-b".to_string(), link_config_b),
            ]
            .into_iter()
            .collect(),
            ..Default::default()
        };

        let ontology = Ontology::from_config(&config);

        assert_eq!(ontology.get_inverse("link-a"), "link-b");
        assert_eq!(ontology.get_inverse("link-b"), "link-a");
    }

    #[test]
    fn test_extended_mode_with_empty_custom_types() {
        let config = OntologyConfig {
            mode: OntologyMode::Extended,
            ..Default::default()
        };

        let ontology = Ontology::from_config(&config);

        assert!(ontology.is_valid_note_type("fleeting"));
        assert!(ontology.is_valid_link_type("related"));
        assert_eq!(ontology.get_inverse("supports"), "supported-by");
    }

    #[test]
    fn test_replacement_mode_with_only_note_types() {
        let config = OntologyConfig {
            mode: OntologyMode::Replacement,
            note_types: [("custom-type".to_string(), NoteTypeConfig::default())]
                .into_iter()
                .collect(),
            ..Default::default()
        };

        let ontology = Ontology::from_config(&config);

        assert!(ontology.is_valid_note_type("custom-type"));
        assert!(!ontology.is_valid_note_type("fleeting"));
        assert!(ontology.link_types().is_empty());
    }

    #[test]
    fn test_replacement_mode_with_only_link_types() {
        let link_config = LinkTypeConfig {
            inverse: Some("my-inverse".to_string()),
            ..Default::default()
        };
        let config = OntologyConfig {
            mode: OntologyMode::Replacement,
            link_types: [("custom-link".to_string(), link_config)]
                .into_iter()
                .collect(),
            ..Default::default()
        };

        let ontology = Ontology::from_config(&config);

        assert!(ontology.is_valid_link_type("custom-link"));
        assert!(!ontology.is_valid_link_type("related"));
        assert!(ontology.note_types().is_empty());
    }
}
