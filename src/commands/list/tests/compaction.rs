//! Tests for list command compaction functionality

use crate::cli::OutputFormat;
use crate::commands::list;
use crate::commands::list::tests::{create_cli, create_test_store};

#[test]
fn test_list_with_compaction_resolved() {
    let (_temp_dir, store) = create_test_store();

    let note1 = store
        .create_note("Original Note", None, &["original".to_string()], None)
        .unwrap();

    let note1_id = note1.id().to_string();

    let mut digest = store.create_note("Digest Note", None, &[], None).unwrap();
    digest.frontmatter.compacts = vec![note1_id.clone()];
    store.save_note(&mut digest).unwrap();

    let cli = create_cli(OutputFormat::Human, false);
    let result = list::execute(&cli, &store, None, None, None, None, None, false, false);
    assert!(result.is_ok());
}

#[test]
fn test_list_with_compaction_disabled() {
    let (_temp_dir, store) = create_test_store();

    let note1 = store
        .create_note("Original Note", None, &["original".to_string()], None)
        .unwrap();

    let note1_id = note1.id().to_string();

    let mut digest = store.create_note("Digest Note", None, &[], None).unwrap();
    digest.frontmatter.compacts = vec![note1_id.clone()];
    store.save_note(&mut digest).unwrap();

    let mut cli = create_cli(OutputFormat::Human, false);
    cli.no_resolve_compaction = true;

    let result = list::execute(&cli, &store, None, None, None, None, None, false, false);
    assert!(result.is_ok());
}

#[test]
fn test_list_with_compaction_ids() {
    let (_temp_dir, store) = create_test_store();

    let note1 = store
        .create_note("Original Note", None, &["original".to_string()], None)
        .unwrap();
    let note1_id = note1.id().to_string();

    let note2 = store
        .create_note("Another Original", None, &["original".to_string()], None)
        .unwrap();
    let note2_id = note2.id().to_string();

    let mut digest = store.create_note("Digest Note", None, &[], None).unwrap();
    digest.frontmatter.compacts = vec![note1_id, note2_id];
    store.save_note(&mut digest).unwrap();

    let mut cli = create_cli(OutputFormat::Human, false);
    cli.with_compaction_ids = true;

    let result = list::execute(&cli, &store, None, None, None, None, None, false, false);
    assert!(result.is_ok());
}

#[test]
fn test_list_with_compaction_ids_depth() {
    let (_temp_dir, store) = create_test_store();

    let note1 = store
        .create_note("Original Note", None, &["original".to_string()], None)
        .unwrap();
    let note1_id = note1.id().to_string();

    let mut digest = store.create_note("Digest Note", None, &[], None).unwrap();
    digest.frontmatter.compacts = vec![note1_id.clone()];
    store.save_note(&mut digest).unwrap();

    let mut cli = create_cli(OutputFormat::Human, false);
    cli.with_compaction_ids = true;
    cli.compaction_depth = Some(2);

    let result = list::execute(&cli, &store, None, None, None, None, None, false, false);
    assert!(result.is_ok());
}

#[test]
fn test_list_compaction_annotations_human() {
    let (_temp_dir, store) = create_test_store();

    let note1 = store
        .create_note_with_content(
            "Original Note",
            None,
            &["original".to_string()],
            "# Summary\nContent from original note 1.",
            None,
        )
        .unwrap();
    let note1_id = note1.id().to_string();

    let note2 = store
        .create_note_with_content(
            "Another Original",
            None,
            &["original".to_string()],
            "# Summary\nContent from original note 2.",
            None,
        )
        .unwrap();
    let note2_id = note2.id().to_string();

    let mut digest = store.create_note("Digest Note", None, &[], None).unwrap();
    digest.frontmatter.compacts = vec![note1_id, note2_id];
    store.save_note(&mut digest).unwrap();

    let cli = create_cli(OutputFormat::Human, false);
    let result = list::execute(&cli, &store, None, None, None, None, None, false, false);
    assert!(result.is_ok());
}

#[test]
fn test_list_compaction_annotations_json() {
    let (_temp_dir, store) = create_test_store();

    let note1 = store
        .create_note_with_content(
            "Original Note",
            None,
            &["original".to_string()],
            "# Summary\nContent from original note.",
            None,
        )
        .unwrap();
    let note1_id = note1.id().to_string();

    let mut digest = store.create_note("Digest Note", None, &[], None).unwrap();
    digest.frontmatter.compacts = vec![note1_id];
    store.save_note(&mut digest).unwrap();

    let cli = create_cli(OutputFormat::Json, false);
    let result = list::execute(&cli, &store, None, None, None, None, None, false, false);
    assert!(result.is_ok());
}

#[test]
fn test_list_compaction_annotations_records() {
    let (_temp_dir, store) = create_test_store();

    let note1 = store
        .create_note_with_content(
            "Original Note",
            None,
            &["original".to_string()],
            "# Summary\nContent from original note.",
            None,
        )
        .unwrap();
    let note1_id = note1.id().to_string();

    let mut digest = store.create_note("Digest Note", None, &[], None).unwrap();
    digest.frontmatter.compacts = vec![note1_id];
    store.save_note(&mut digest).unwrap();

    let cli = create_cli(OutputFormat::Records, false);
    let result = list::execute(&cli, &store, None, None, None, None, None, false, false);
    assert!(result.is_ok());
}

#[test]
fn test_list_all_formats_compaction_with_ids() {
    let (_temp_dir, store) = create_test_store();

    let note1 = store
        .create_note("Original Note", None, &["original".to_string()], None)
        .unwrap();
    let note1_id = note1.id().to_string();

    let mut digest = store.create_note("Digest Note", None, &[], None).unwrap();
    digest.frontmatter.compacts = vec![note1_id];
    store.save_note(&mut digest).unwrap();

    for format in [
        OutputFormat::Human,
        OutputFormat::Json,
        OutputFormat::Records,
    ] {
        let mut cli = create_cli(format, false);
        cli.with_compaction_ids = true;
        let result = list::execute(&cli, &store, None, None, None, None, None, false, false);
        assert!(result.is_ok());
    }
}
