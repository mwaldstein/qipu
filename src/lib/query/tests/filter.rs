use crate::lib::compaction::CompactionContext;
use crate::lib::note::{Note, NoteFrontmatter, NoteType};
use crate::lib::query::NoteFilter;
use chrono::{DateTime, Duration, Utc};
use serde_yaml::Value;
use std::path::PathBuf;

fn create_test_note(
    id: &str,
    title: &str,
    tags: Vec<String>,
    note_type: Option<NoteType>,
    created: Option<DateTime<Utc>>,
    value: Option<u8>,
) -> Note {
    Note {
        frontmatter: NoteFrontmatter {
            id: id.to_string(),
            title: title.to_string(),
            tags,
            created,
            updated: None,
            note_type,
            compacts: vec![],
            sources: vec![],
            links: vec![],
            summary: None,
            source: None,
            author: None,
            generated_by: None,
            prompt_hash: None,
            verified: None,
            value,
            custom: std::collections::HashMap::new(),
        },
        body: String::new(),
        path: Some(PathBuf::from(format!("{}.md", id))),
    }
}

#[test]
fn test_filter_with_tag() {
    let note = create_test_note(
        "qp-abc",
        "Test Note",
        vec!["matching".to_string()],
        None,
        None,
        None,
    );

    let filter = NoteFilter::new().with_tag(Some("matching"));
    let compaction_ctx = CompactionContext::build(&[]).unwrap();

    assert!(filter.matches(&note, &compaction_ctx));
}

#[test]
fn test_filter_without_tag() {
    let note = create_test_note(
        "qp-abc",
        "Test Note",
        vec!["other".to_string()],
        None,
        None,
        None,
    );

    let filter = NoteFilter::new().with_tag(Some("matching"));
    let compaction_ctx = CompactionContext::build(&[]).unwrap();

    assert!(!filter.matches(&note, &compaction_ctx));
}

#[test]
fn test_filter_with_type() {
    let note = create_test_note(
        "qp-abc",
        "Test Note",
        vec![],
        Some(NoteType::from(NoteType::PERMANENT)),
        None,
        None,
    );

    let filter = NoteFilter::new().with_type(Some(NoteType::from(NoteType::PERMANENT)));
    let compaction_ctx = CompactionContext::build(&[]).unwrap();

    assert!(filter.matches(&note, &compaction_ctx));
}

#[test]
fn test_filter_without_type() {
    let note = create_test_note(
        "qp-abc",
        "Test Note",
        vec![],
        Some(NoteType::from(NoteType::FLEETING)),
        None,
        None,
    );

    let filter = NoteFilter::new().with_type(Some(NoteType::from(NoteType::PERMANENT)));
    let compaction_ctx = CompactionContext::build(&[]).unwrap();

    assert!(!filter.matches(&note, &compaction_ctx));
}

#[test]
fn test_filter_with_since() {
    let now = Utc::now();
    let note = create_test_note(
        "qp-abc",
        "Test Note",
        vec![],
        None,
        Some(now - Duration::days(1)),
        None,
    );

    let filter = NoteFilter::new().with_since(Some(now - Duration::days(5)));
    let compaction_ctx = CompactionContext::build(&[]).unwrap();

    assert!(filter.matches(&note, &compaction_ctx));
}

#[test]
fn test_filter_without_since() {
    let now = Utc::now();
    let note = create_test_note(
        "qp-abc",
        "Test Note",
        vec![],
        None,
        Some(now - Duration::days(10)),
        None,
    );

    let filter = NoteFilter::new().with_since(Some(now - Duration::days(5)));
    let compaction_ctx = CompactionContext::build(&[]).unwrap();

    assert!(!filter.matches(&note, &compaction_ctx));
}

#[test]
fn test_filter_with_min_value() {
    let note = create_test_note("qp-abc", "Test Note", vec![], None, None, Some(75));

    let filter = NoteFilter::new().with_min_value(Some(50));
    let compaction_ctx = CompactionContext::build(&[]).unwrap();

    assert!(filter.matches(&note, &compaction_ctx));
}

#[test]
fn test_filter_without_min_value() {
    let note = create_test_note("qp-abc", "Test Note", vec![], None, None, Some(30));

    let filter = NoteFilter::new().with_min_value(Some(50));
    let compaction_ctx = CompactionContext::build(&[]).unwrap();

    assert!(!filter.matches(&note, &compaction_ctx));
}

#[test]
fn test_filter_with_min_value_default() {
    let note = create_test_note("qp-abc", "Test Note", vec![], None, None, None);

    let filter = NoteFilter::new().with_min_value(Some(50));
    let compaction_ctx = CompactionContext::build(&[]).unwrap();

    assert!(filter.matches(&note, &compaction_ctx));
}

fn create_note_with_custom(custom: std::collections::HashMap<String, Value>) -> Note {
    Note {
        frontmatter: NoteFrontmatter {
            id: "qp-abc".to_string(),
            title: "Test Note".to_string(),
            tags: vec![],
            created: None,
            updated: None,
            note_type: None,
            compacts: vec![],
            sources: vec![],
            links: vec![],
            summary: None,
            source: None,
            author: None,
            generated_by: None,
            prompt_hash: None,
            verified: None,
            value: None,
            custom,
        },
        body: String::new(),
        path: Some(PathBuf::from("qp-abc.md")),
    }
}

#[test]
fn test_filter_with_custom_string() {
    let mut custom = std::collections::HashMap::new();
    custom.insert("key".to_string(), Value::String("value".to_string()));
    let note = create_note_with_custom(custom);

    let filter = NoteFilter::new().with_custom(Some("key=value"));
    let compaction_ctx = CompactionContext::build(&[]).unwrap();

    assert!(filter.matches(&note, &compaction_ctx));
}

#[test]
fn test_filter_with_custom_number() {
    let mut custom = std::collections::HashMap::new();
    custom.insert(
        "count".to_string(),
        Value::Number(serde_yaml::Number::from(42)),
    );
    let note = create_note_with_custom(custom);

    let filter = NoteFilter::new().with_custom(Some("count=42"));
    let compaction_ctx = CompactionContext::build(&[]).unwrap();

    assert!(filter.matches(&note, &compaction_ctx));
}

#[test]
fn test_filter_with_custom_bool() {
    let mut custom = std::collections::HashMap::new();
    custom.insert("flag".to_string(), Value::Bool(true));
    let note = create_note_with_custom(custom);

    let filter = NoteFilter::new().with_custom(Some("flag=true"));
    let compaction_ctx = CompactionContext::build(&[]).unwrap();

    assert!(filter.matches(&note, &compaction_ctx));
}

#[test]
fn test_filter_with_custom_mismatch() {
    let mut custom = std::collections::HashMap::new();
    custom.insert("key".to_string(), Value::String("other".to_string()));
    let note = create_note_with_custom(custom);

    let filter = NoteFilter::new().with_custom(Some("key=value"));
    let compaction_ctx = CompactionContext::build(&[]).unwrap();

    assert!(!filter.matches(&note, &compaction_ctx));
}

#[test]
fn test_filter_with_custom_exists() {
    let mut custom = std::collections::HashMap::new();
    custom.insert("key".to_string(), Value::String("value".to_string()));
    let note = create_note_with_custom(custom);

    let filter = NoteFilter::new().with_custom(Some("key"));
    let compaction_ctx = CompactionContext::build(&[]).unwrap();

    assert!(filter.matches(&note, &compaction_ctx));
}

#[test]
fn test_filter_with_custom_not_exists() {
    let mut custom = std::collections::HashMap::new();
    custom.insert("key".to_string(), Value::String("value".to_string()));
    let note = create_note_with_custom(custom);

    let filter = NoteFilter::new().with_custom(Some("!other"));
    let compaction_ctx = CompactionContext::build(&[]).unwrap();

    assert!(filter.matches(&note, &compaction_ctx));
}

#[test]
fn test_filter_with_custom_absent() {
    let mut custom = std::collections::HashMap::new();
    custom.insert("key".to_string(), Value::String("value".to_string()));
    let note = create_note_with_custom(custom);

    let filter = NoteFilter::new().with_custom(Some("!key"));
    let compaction_ctx = CompactionContext::build(&[]).unwrap();

    assert!(!filter.matches(&note, &compaction_ctx));
}

#[test]
fn test_filter_with_custom_numeric_greater_than() {
    let mut custom = std::collections::HashMap::new();
    custom.insert(
        "priority".to_string(),
        Value::Number(serde_yaml::Number::from(10)),
    );
    let note = create_note_with_custom(custom);

    let filter = NoteFilter::new().with_custom(Some("priority>5"));
    let compaction_ctx = CompactionContext::build(&[]).unwrap();

    assert!(filter.matches(&note, &compaction_ctx));
}

#[test]
fn test_filter_with_custom_numeric_less_than() {
    let mut custom = std::collections::HashMap::new();
    custom.insert(
        "priority".to_string(),
        Value::Number(serde_yaml::Number::from(3)),
    );
    let note = create_note_with_custom(custom);

    let filter = NoteFilter::new().with_custom(Some("priority<5"));
    let compaction_ctx = CompactionContext::build(&[]).unwrap();

    assert!(filter.matches(&note, &compaction_ctx));
}

#[test]
fn test_filter_with_custom_numeric_greater_equal() {
    let mut custom = std::collections::HashMap::new();
    custom.insert(
        "priority".to_string(),
        Value::Number(serde_yaml::Number::from(5)),
    );
    let note = create_note_with_custom(custom);

    let filter = NoteFilter::new().with_custom(Some("priority>=5"));
    let compaction_ctx = CompactionContext::build(&[]).unwrap();

    assert!(filter.matches(&note, &compaction_ctx));
}

#[test]
fn test_filter_with_custom_numeric_less_equal() {
    let mut custom = std::collections::HashMap::new();
    custom.insert(
        "priority".to_string(),
        Value::Number(serde_yaml::Number::from(5)),
    );
    let note = create_note_with_custom(custom);

    let filter = NoteFilter::new().with_custom(Some("priority<=5"));
    let compaction_ctx = CompactionContext::build(&[]).unwrap();

    assert!(filter.matches(&note, &compaction_ctx));
}

#[test]
fn test_filter_with_custom_numeric_fails() {
    let mut custom = std::collections::HashMap::new();
    custom.insert(
        "priority".to_string(),
        Value::Number(serde_yaml::Number::from(3)),
    );
    let note = create_note_with_custom(custom);

    let filter = NoteFilter::new().with_custom(Some("priority>5"));
    let compaction_ctx = CompactionContext::build(&[]).unwrap();

    assert!(!filter.matches(&note, &compaction_ctx));
}

#[test]
fn test_filter_with_custom_numeric_string_value() {
    let mut custom = std::collections::HashMap::new();
    custom.insert("priority".to_string(), Value::String("10".to_string()));
    let note = create_note_with_custom(custom);

    let filter = NoteFilter::new().with_custom(Some("priority>5"));
    let compaction_ctx = CompactionContext::build(&[]).unwrap();

    assert!(filter.matches(&note, &compaction_ctx));
}

#[test]
fn test_filter_with_custom_numeric_bool_true() {
    let mut custom = std::collections::HashMap::new();
    custom.insert("active".to_string(), Value::Bool(true));
    let note = create_note_with_custom(custom);

    let filter = NoteFilter::new().with_custom(Some("active>0"));
    let compaction_ctx = CompactionContext::build(&[]).unwrap();

    assert!(filter.matches(&note, &compaction_ctx));
}

#[test]
fn test_filter_with_custom_numeric_bool_false() {
    let mut custom = std::collections::HashMap::new();
    custom.insert("active".to_string(), Value::Bool(false));
    let note = create_note_with_custom(custom);

    let filter = NoteFilter::new().with_custom(Some("active<1"));
    let compaction_ctx = CompactionContext::build(&[]).unwrap();

    assert!(filter.matches(&note, &compaction_ctx));
}
