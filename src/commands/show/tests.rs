//! Tests for `qipu show` command

use crate::cli::{Cli, OutputFormat};
use crate::commands::show;
use crate::lib::note::NoteType;
use crate::lib::store::InitOptions;
use crate::lib::store::Store;
use std::fs;
use std::io::Write;
use tempfile::tempdir;

fn make_default_cli() -> Cli {
    Cli {
        root: None,
        store: None,
        format: OutputFormat::Human,
        quiet: false,
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

#[test]
fn test_show_by_id() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), InitOptions::default()).unwrap();

    let note = store
        .create_note("Test Note", None, &["test".to_string()], None)
        .unwrap();
    let id = note.id();

    let cli = make_default_cli();
    let result = show::execute(&cli, &store, id, false, false);
    assert!(result.is_ok(), "Show by ID should succeed");
}

#[test]
fn test_show_by_file_path() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.md");

    let mut file = fs::File::create(&file_path).unwrap();
    writeln!(
        file,
        "---\nid: qp-external\ntitle: External Note\n---\n\nThis is an external note."
    )
    .unwrap();

    let store = Store::init(dir.path(), InitOptions::default()).unwrap();
    let cli = make_default_cli();

    let result = show::execute(&cli, &store, file_path.to_str().unwrap(), false, false);
    match result {
        Ok(_) => {}
        Err(e) => panic!("Show by file path failed: {}", e),
    }
}

#[test]
fn test_show_nonexistent_id() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), InitOptions::default()).unwrap();

    let cli = make_default_cli();
    let result = show::execute(&cli, &store, "qp-nonexistent", false, false);
    assert!(result.is_err(), "Show nonexistent ID should fail");
}

#[test]
fn test_show_json_format() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), InitOptions::default()).unwrap();

    let note = store
        .create_note(
            "JSON Note",
            Some(NoteType::from(NoteType::PERMANENT)),
            &["json".to_string()],
            None,
        )
        .unwrap();
    let id = note.id();

    let mut cli = make_default_cli();
    cli.format = OutputFormat::Json;

    let result = show::execute(&cli, &store, id, false, false);
    assert!(result.is_ok(), "Show with JSON format should succeed");
}

#[test]
fn test_show_records_format() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), InitOptions::default()).unwrap();

    let mut note = store
        .create_note(
            "Records Note",
            Some(NoteType::from(NoteType::FLEETING)),
            &[],
            None,
        )
        .unwrap();
    note.body = "This is the body content.\nWith multiple lines.".to_string();
    store.save_note(&mut note).unwrap();
    let id = note.id();

    let mut cli = make_default_cli();
    cli.format = OutputFormat::Records;

    let result = show::execute(&cli, &store, id, false, false);
    assert!(result.is_ok(), "Show with records format should succeed");
}

#[test]
fn test_show_with_compaction_resolution() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), InitOptions::default()).unwrap();

    let mut digest_note = store
        .create_note("Digest Note", None, &["digest".to_string()], None)
        .unwrap();
    digest_note.body = "Compacts from qp-source".to_string();
    store.save_note(&mut digest_note).unwrap();

    let source_note = store
        .create_note("Source Note", None, &["source".to_string()], None)
        .unwrap();

    let cli = make_default_cli();
    let result = show::execute(&cli, &store, source_note.id(), false, false);
    assert!(
        result.is_ok(),
        "Show with compaction resolution should succeed"
    );
}

#[test]
fn test_show_no_resolve_compaction() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), InitOptions::default()).unwrap();

    let mut digest_note = store
        .create_note("Digest Note", None, &["digest".to_string()], None)
        .unwrap();
    digest_note.body = "Compacts from qp-source".to_string();
    store.save_note(&mut digest_note).unwrap();

    let source_note = store
        .create_note("Source Note", None, &["source".to_string()], None)
        .unwrap();

    let mut cli = make_default_cli();
    cli.no_resolve_compaction = true;

    let result = show::execute(&cli, &store, source_note.id(), false, false);
    assert!(
        result.is_ok(),
        "Show with no resolve compaction should succeed"
    );
}

#[test]
fn test_show_links_mode() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), InitOptions::default()).unwrap();

    let note = store
        .create_note("Linked Note", None, &["test".to_string()], None)
        .unwrap();
    let id = note.id();

    let cli = make_default_cli();
    let result = show::execute(&cli, &store, id, true, false);
    assert!(result.is_ok(), "Show links mode should succeed");
}

#[test]
fn test_show_links_json_format() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), InitOptions::default()).unwrap();

    let note = store
        .create_note("JSON Links Note", None, &["test".to_string()], None)
        .unwrap();
    let id = note.id();

    let mut cli = make_default_cli();
    cli.format = OutputFormat::Json;

    let result = show::execute(&cli, &store, id, true, false);
    assert!(result.is_ok(), "Show links with JSON format should succeed");
}

#[test]
fn test_show_links_records_format() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), InitOptions::default()).unwrap();

    let note = store
        .create_note("Records Links Note", None, &["test".to_string()], None)
        .unwrap();
    let id = note.id();

    let mut cli = make_default_cli();
    cli.format = OutputFormat::Records;

    let result = show::execute(&cli, &store, id, true, false);
    assert!(
        result.is_ok(),
        "Show links with records format should succeed"
    );
}

#[test]
fn test_show_with_compaction_ids() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), InitOptions::default()).unwrap();

    let mut digest_note = store
        .create_note("Digest with IDs", None, &["digest".to_string()], None)
        .unwrap();
    digest_note.body = "Compacts from qp-source1, qp-source2".to_string();
    store.save_note(&mut digest_note).unwrap();
    let id = digest_note.id();

    let mut cli = make_default_cli();
    cli.format = OutputFormat::Json;
    cli.with_compaction_ids = true;

    let result = show::execute(&cli, &store, id, false, false);
    assert!(result.is_ok(), "Show with compaction IDs should succeed");
}

#[test]
fn test_show_verbose() {
    let dir = tempdir().unwrap();
    let store = Store::init(dir.path(), InitOptions::default()).unwrap();

    let note = store
        .create_note("Verbose Note", None, &["test".to_string()], None)
        .unwrap();
    let id = note.id();

    let mut cli = make_default_cli();
    cli.verbose = true;

    let result = show::execute(&cli, &store, id, false, false);
    assert!(result.is_ok(), "Show with verbose should succeed");
}
