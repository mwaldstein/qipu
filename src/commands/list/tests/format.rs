//! Tests for list command output formats

use crate::cli::OutputFormat;
use crate::commands::list;
use crate::commands::list::tests::{create_cli, create_test_store};

#[test]
fn test_list_empty_store_human() {
    let (_temp_dir, store) = create_test_store();
    let cli = create_cli(OutputFormat::Human, false);

    let result = list::execute(&cli, &store, None, None, None, None, None, false);
    assert!(result.is_ok());
}

#[test]
fn test_list_empty_store_quiet() {
    let (_temp_dir, store) = create_test_store();
    let cli = create_cli(OutputFormat::Human, true);

    let result = list::execute(&cli, &store, None, None, None, None, None, false);
    assert!(result.is_ok());
}

#[test]
fn test_list_empty_store_json() {
    let (_temp_dir, store) = create_test_store();
    let cli = create_cli(OutputFormat::Json, false);

    let result = list::execute(&cli, &store, None, None, None, None, None, false);
    assert!(result.is_ok());
}

#[test]
fn test_list_empty_store_records() {
    let (_temp_dir, store) = create_test_store();
    let cli = create_cli(OutputFormat::Records, false);

    let result = list::execute(&cli, &store, None, None, None, None, None, false);
    assert!(result.is_ok());
}

#[test]
fn test_list_single_note_human() {
    let (_temp_dir, store) = create_test_store();
    store
        .create_note("Test Note", None, &["tag1".to_string()], None)
        .unwrap();

    let cli = create_cli(OutputFormat::Human, false);
    let result = list::execute(&cli, &store, None, None, None, None, None, false);
    assert!(result.is_ok());
}

#[test]
fn test_list_single_note_json() {
    let (_temp_dir, store) = create_test_store();
    store
        .create_note("Test Note", None, &["tag1".to_string()], None)
        .unwrap();

    let cli = create_cli(OutputFormat::Json, false);
    let result = list::execute(&cli, &store, None, None, None, None, None, false);
    assert!(result.is_ok());
}

#[test]
fn test_list_single_note_records() {
    let (_temp_dir, store) = create_test_store();
    store
        .create_note("Test Note", None, &["tag1".to_string()], None)
        .unwrap();

    let cli = create_cli(OutputFormat::Records, false);
    let result = list::execute(&cli, &store, None, None, None, None, None, false);
    assert!(result.is_ok());
}

#[test]
fn test_list_multiple_notes() {
    let (_temp_dir, store) = create_test_store();
    store
        .create_note("Note 1", None, &["tag1".to_string()], None)
        .unwrap();
    store
        .create_note("Note 2", None, &["tag2".to_string()], None)
        .unwrap();
    store
        .create_note("Note 3", None, &["tag3".to_string()], None)
        .unwrap();

    let cli = create_cli(OutputFormat::Human, false);
    let result = list::execute(&cli, &store, None, None, None, None, None, false);
    assert!(result.is_ok());
}
