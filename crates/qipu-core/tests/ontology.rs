//! Unit tests for ontology resolution

use qipu_core::config::{LinkTypeConfig, NoteTypeConfig, OntologyConfig, OntologyMode};
use qipu_core::ontology::Ontology;

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
