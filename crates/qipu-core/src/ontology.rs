//! Ontology resolution for custom note and link types
//!
//! Provides resolution logic for merging custom types with the standard ontology
//! based on the configured resolution mode (default, extended, or replacement).

use std::collections::{HashMap, HashSet};

use crate::config::{GraphConfig, OntologyConfig, OntologyMode};
use crate::note::LinkType;

/// Standard note types (built-in)
const STANDARD_NOTE_TYPES: &[&str] = &["fleeting", "literature", "permanent", "moc"];

/// Standard link type inverses (built-in)
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
pub struct Ontology {
    /// All valid note types
    note_types: HashSet<String>,
    /// All valid link types
    link_types: HashSet<String>,
    /// Link type inverses (link_type -> inverse)
    inverses: HashMap<String, String>,
}

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
    /// For unknown types, returns inverse-&lt;type&gt;
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
    pub fn validate_link_type(&self, link_type: &str) -> Result<(), crate::error::QipuError> {
        if !self.is_valid_link_type(link_type) {
            return Err(crate::error::QipuError::UsageError(format!(
                "Invalid link type: '{}'",
                link_type
            )));
        }
        Ok(())
    }

    /// Validate a note type
    pub fn validate_note_type(&self, note_type: &str) -> Result<(), crate::error::QipuError> {
        if !self.is_valid_note_type(note_type) {
            let valid_types = self.note_types().join(", ");
            return Err(crate::error::QipuError::UsageError(format!(
                "Invalid note type: '{}'. Valid types: {}",
                note_type, valid_types
            )));
        }
        Ok(())
    }
}
