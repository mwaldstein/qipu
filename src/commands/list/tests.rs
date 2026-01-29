//! Tests for `qipu list` command

use crate::cli::{Cli, OutputFormat};
use crate::commands::list;
use qipu_core::note::NoteType;
use qipu_core::store::{InitOptions, Store};
use chrono::{Duration, Utc};
use tempfile::TempDir;

fn create_cli(format: OutputFormat, quiet: bool) -> Cli {
    Cli {
        root: None,
        store: None,
        format,
        quiet,
        verbose: false,
        log_level: None,
        log_json: false,
        no_resolve_compaction: false,
        with_compaction_ids: false,
        compaction_depth: None,
        compaction_max_nodes: None,
        expand_compaction: false,
        workspace: None,
        no_semantic_inversion: false,
        command: None,
    }
}

fn create_test_store() -> (TempDir, Store) {
    let temp_dir = TempDir::new().unwrap();
    let store = Store::init(temp_dir.path(), InitOptions::default()).unwrap();
    (temp_dir, store)
}

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
    let result = list::execute(&cli, &store, None, None, Some(since), None, None, false);
    assert!(result.is_ok());
}

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
    let result = list::execute(&cli, &store, None, None, None, None, None, false);
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

    let result = list::execute(&cli, &store, None, None, None, None, None, false);
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

    let result = list::execute(&cli, &store, None, None, None, None, None, false);
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

    let result = list::execute(&cli, &store, None, None, None, None, None, false);
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
    let result = list::execute(&cli, &store, None, None, None, None, None, false);
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
    let result = list::execute(&cli, &store, None, None, None, None, None, false);
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
    let result = list::execute(&cli, &store, None, None, None, None, None, false);
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
        let result = list::execute(&cli, &store, None, None, None, None, None, false);
        assert!(result.is_ok());
    }
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
    let result = list::execute(&cli, &store, None, None, None, Some(50), None, false);
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
    let result = list::execute(&cli, &store, None, None, None, Some(50), None, false);
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
    let result = list::execute(&cli, &store, None, None, None, Some(50), None, false);
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
    let result = list::execute(&cli, &store, None, None, None, Some(40), None, false);
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
    let result = list::execute(&cli, &store, None, None, None, Some(50), None, false);
    assert!(result.is_ok());
}
