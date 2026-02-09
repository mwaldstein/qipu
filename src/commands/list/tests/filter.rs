//! Tests for list command filtering functionality

use crate::cli::OutputFormat;
use crate::commands::list;
use crate::commands::list::tests::{create_cli, create_test_store};
use chrono::{Duration, Utc};
use qipu_core::note::NoteType;

#[test]
fn test_list_filter_by_tag() {
    let (_temp_dir, store) = create_test_store();
    store
        .create_note("Note 1", None, &["matching".to_string()], None)
        .unwrap();
    store
        .create_note("Note 2", None, &["other".to_string()], None)
        .unwrap();
    store
        .create_note("Note 3", None, &["matching".to_string()], None)
        .unwrap();

    let cli = create_cli(OutputFormat::Human, false);
    let result = list::execute(
        &cli,
        &store,
        Some("matching"),
        None,
        None,
        None,
        None,
        false,
        false,
    );
    assert!(result.is_ok());
}

#[test]
fn test_list_filter_by_tag_none_matching() {
    let (_temp_dir, store) = create_test_store();
    store
        .create_note("Note 1", None, &["tag1".to_string()], None)
        .unwrap();

    let cli = create_cli(OutputFormat::Human, false);
    let result = list::execute(
        &cli,
        &store,
        Some("nonexistent"),
        None,
        None,
        None,
        None,
        false,
        false,
    );
    assert!(result.is_ok());
}

#[test]
fn test_list_filter_by_type() {
    let (_temp_dir, store) = create_test_store();
    let mut note1 = store.create_note("Fleeting Note", None, &[], None).unwrap();
    note1.frontmatter.note_type = Some(NoteType::from(NoteType::FLEETING));
    store.save_note(&mut note1).unwrap();

    let mut note2 = store
        .create_note("Permanent Note", None, &[], None)
        .unwrap();
    note2.frontmatter.note_type = Some(NoteType::from(NoteType::PERMANENT));
    store.save_note(&mut note2).unwrap();

    let cli = create_cli(OutputFormat::Human, false);
    let result = list::execute(
        &cli,
        &store,
        None,
        Some(NoteType::from(NoteType::PERMANENT)),
        None,
        None,
        None,
        false,
        false,
    );
    assert!(result.is_ok());
}

#[test]
fn test_list_filter_by_since() {
    let (_temp_dir, store) = create_test_store();

    let mut note1 = store.create_note("Old Note", None, &[], None).unwrap();
    note1.frontmatter.created = Some(Utc::now() - Duration::days(10));
    store.save_note(&mut note1).unwrap();

    let mut note2 = store.create_note("Recent Note", None, &[], None).unwrap();
    note2.frontmatter.created = Some(Utc::now() - Duration::days(1));
    store.save_note(&mut note2).unwrap();

    let cli = create_cli(OutputFormat::Human, false);
    let since = Utc::now() - Duration::days(5);
    let result = list::execute(
        &cli,
        &store,
        None,
        None,
        Some(since),
        None,
        None,
        false,
        false,
    );
    assert!(result.is_ok());
}

#[test]
fn test_list_filter_by_min_value_all_match() {
    let (_temp_dir, store) = create_test_store();

    let mut note1 = store
        .create_note("High Value Note", None, &["high".to_string()], None)
        .unwrap();
    note1.frontmatter.value = Some(90);
    store.save_note(&mut note1).unwrap();

    let mut note2 = store
        .create_note("Medium Value Note", None, &["medium".to_string()], None)
        .unwrap();
    note2.frontmatter.value = Some(70);
    store.save_note(&mut note2).unwrap();

    let cli = create_cli(OutputFormat::Human, false);
    let result = list::execute(&cli, &store, None, None, None, Some(50), None, false, false);
    assert!(result.is_ok());
}

#[test]
fn test_list_filter_by_min_value_some_match() {
    let (_temp_dir, store) = create_test_store();

    let mut note1 = store
        .create_note("High Value Note", None, &["high".to_string()], None)
        .unwrap();
    note1.frontmatter.value = Some(90);
    store.save_note(&mut note1).unwrap();

    let mut note2 = store
        .create_note("Low Value Note", None, &["low".to_string()], None)
        .unwrap();
    note2.frontmatter.value = Some(30);
    store.save_note(&mut note2).unwrap();

    let cli = create_cli(OutputFormat::Human, false);
    let result = list::execute(&cli, &store, None, None, None, Some(50), None, false, false);
    assert!(result.is_ok());
}

#[test]
fn test_list_filter_by_min_value_none_match() {
    let (_temp_dir, store) = create_test_store();

    let mut note1 = store
        .create_note("Low Value Note 1", None, &["low".to_string()], None)
        .unwrap();
    note1.frontmatter.value = Some(20);
    store.save_note(&mut note1).unwrap();

    let mut note2 = store
        .create_note("Low Value Note 2", None, &["low".to_string()], None)
        .unwrap();
    note2.frontmatter.value = Some(10);
    store.save_note(&mut note2).unwrap();

    let cli = create_cli(OutputFormat::Human, false);
    let result = list::execute(&cli, &store, None, None, None, Some(50), None, false, false);
    assert!(result.is_ok());
}

#[test]
fn test_list_filter_by_min_value_with_defaults() {
    let (_temp_dir, store) = create_test_store();

    let _note1 = store
        .create_note("Default Value Note", None, &["default".to_string()], None)
        .unwrap();

    let mut note2 = store
        .create_note("Low Value Note", None, &["low".to_string()], None)
        .unwrap();
    note2.frontmatter.value = Some(20);
    store.save_note(&mut note2).unwrap();

    let cli = create_cli(OutputFormat::Human, false);
    let result = list::execute(&cli, &store, None, None, None, Some(40), None, false, false);
    assert!(result.is_ok());
}

#[test]
fn test_list_filter_by_min_value_exact() {
    let (_temp_dir, store) = create_test_store();

    let mut note1 = store
        .create_note("Value 75 Note", None, &["exact".to_string()], None)
        .unwrap();
    note1.frontmatter.value = Some(75);
    store.save_note(&mut note1).unwrap();

    let mut note2 = store
        .create_note("Value 50 Note", None, &["exact".to_string()], None)
        .unwrap();
    note2.frontmatter.value = Some(50);
    store.save_note(&mut note2).unwrap();

    let cli = create_cli(OutputFormat::Human, false);
    let result = list::execute(&cli, &store, None, None, None, Some(50), None, false, false);
    assert!(result.is_ok());
}
