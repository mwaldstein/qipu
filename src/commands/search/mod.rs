//! `qipu search` command - search notes
//!
//! Per spec (specs/cli-interface.md, specs/indexing-search.md):
//! - `qipu search <query>` - search titles + bodies
//! - `--type` filter
//! - `--tag` filter
//! - Result ranking: title > exact tag > body, recency boost
//! - Compaction resolution (specs/compaction.md): show canonical digests with via= annotations

pub mod format;

use std::time::Instant;

use tracing::debug;

use crate::cli::Cli;
use qipu_core::compaction::CompactionContext;
use qipu_core::error::Result;
use qipu_core::note::NoteType;
use qipu_core::search;
use qipu_core::store::Store;

use self::format::{output_human, output_json, output_records};

/// Execute the search command
#[allow(clippy::too_many_arguments)]
pub fn execute(
    cli: &Cli,
    store: &Store,
    query: &str,
    type_filter: Option<NoteType>,
    tag_filter: Option<&str>,
    exclude_mocs: bool,
    min_value: Option<u8>,
    sort: Option<&str>,
) -> Result<()> {
    let start = Instant::now();

    // Resolve tag aliases for filtering
    let equivalent_tags = tag_filter.map(|t| store.config().get_equivalent_tags(t));

    if cli.verbose {
        debug!(
            query,
            ?type_filter,
            ?tag_filter,
            ?equivalent_tags,
            exclude_mocs,
            ?min_value,
            ?sort,
            "search_params"
        );
    }

    let results = store.db().search(
        query,
        type_filter,
        tag_filter,
        min_value,
        equivalent_tags.as_deref(),
        200,
        &store.config().search,
    )?;

    if cli.verbose {
        debug!(result_count = results.len(), elapsed = ?start.elapsed(), "search");
    }

    let needs_compaction = !cli.no_resolve_compaction
        || cli.with_compaction_ids
        || cli.compaction_depth.is_some()
        || cli.compaction_max_nodes.is_some();

    let all_notes = if needs_compaction {
        store.list_notes()?
    } else {
        Vec::new()
    };

    let compaction_ctx = if needs_compaction {
        if cli.verbose {
            debug!(note_count = all_notes.len(), "build_compaction_context");
        }
        Some(CompactionContext::build(&all_notes)?)
    } else {
        None
    };

    let compaction_note_map = if needs_compaction {
        Some(CompactionContext::build_note_map(&all_notes))
    } else {
        None
    };

    let (results, notes_cache, _compacts_count) = search::process_search_results(
        results,
        !cli.no_resolve_compaction,
        store,
        &compaction_ctx,
        &compaction_note_map,
        exclude_mocs,
        sort,
    );

    match cli.format {
        crate::cli::OutputFormat::Json => {
            output_json(
                cli,
                store,
                &results,
                &compaction_ctx,
                &notes_cache,
                &compaction_note_map,
            )?;
        }
        crate::cli::OutputFormat::Human => {
            output_human(
                cli,
                store,
                &results,
                &compaction_ctx,
                &notes_cache,
                &compaction_note_map,
                query,
            );
        }
        crate::cli::OutputFormat::Records => {
            output_records(
                cli,
                store,
                &results,
                &compaction_ctx,
                &notes_cache,
                &compaction_note_map,
                query,
            );
        }
    }

    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "execute_command");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use qipu_core::note::NoteType;
    use qipu_core::store::InitOptions;
    use tempfile::tempdir;

    #[test]
    fn test_search_empty_query() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        let cli = Cli {
            root: None,
            store: None,
            format: crate::cli::OutputFormat::Human,
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
        };

        let result = execute(&cli, &store, "", None, None, false, None, None);
        assert!(result.is_ok(), "Empty query should not error");
    }

    fn make_default_cli() -> Cli {
        Cli {
            root: None,
            store: None,
            format: crate::cli::OutputFormat::Human,
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
    fn test_search_no_results() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        let cli = make_default_cli();

        let result = execute(&cli, &store, "nonexistent", None, None, false, None, None);
        assert!(result.is_ok(), "Search with no results should succeed");
    }

    #[test]
    fn test_search_with_type_filter() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        store
            .create_note(
                "Permanent Note",
                Some(NoteType::from(NoteType::PERMANENT)),
                &[],
                None,
            )
            .unwrap();
        store
            .create_note(
                "Fleeting Note",
                Some(NoteType::from(NoteType::FLEETING)),
                &[],
                None,
            )
            .unwrap();

        let cli = make_default_cli();

        let result = execute(
            &cli,
            &store,
            "note",
            Some(NoteType::from(NoteType::PERMANENT)),
            None,
            false,
            None,
            None,
        );
        assert!(result.is_ok(), "Search with type filter should succeed");
    }

    #[test]
    fn test_search_with_tag_filter() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        store
            .create_note("Tagged Note", None, &["rust".to_string()], None)
            .unwrap();
        store.create_note("Untagged Note", None, &[], None).unwrap();

        let cli = make_default_cli();

        let result = execute(&cli, &store, "note", None, Some("rust"), false, None, None);
        assert!(result.is_ok(), "Search with tag filter should succeed");
    }

    #[test]
    fn test_search_exclude_mocs() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        store
            .create_note("MOC Note", Some(NoteType::from(NoteType::MOC)), &[], None)
            .unwrap();
        store
            .create_note(
                "Regular Note",
                Some(NoteType::from(NoteType::FLEETING)),
                &[],
                None,
            )
            .unwrap();

        let cli = make_default_cli();

        let result = execute(&cli, &store, "note", None, None, true, None, None);
        assert!(result.is_ok(), "Search with MOC exclusion should succeed");
    }

    #[test]
    fn test_search_json_format() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        store
            .create_note("Test Note", None, &["test".to_string()], None)
            .unwrap();

        let mut cli = make_default_cli();
        cli.format = crate::cli::OutputFormat::Json;

        let result = execute(&cli, &store, "test", None, None, false, None, None);
        assert!(result.is_ok(), "Search with JSON format should succeed");
    }

    #[test]
    fn test_search_records_format() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        store
            .create_note("Test Note", None, &["test".to_string()], None)
            .unwrap();

        let mut cli = make_default_cli();
        cli.format = crate::cli::OutputFormat::Records;

        let result = execute(&cli, &store, "test", None, None, false, None, None);
        assert!(result.is_ok(), "Search with records format should succeed");
    }

    #[test]
    fn test_search_quiet_no_results() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        let mut cli = make_default_cli();
        cli.quiet = true;

        let result = execute(&cli, &store, "nonexistent", None, None, false, None, None);
        assert!(
            result.is_ok(),
            "Quiet search with no results should succeed"
        );
    }

    #[test]
    fn test_search_verbose_output() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        store.create_note("Test Note", None, &[], None).unwrap();

        let mut cli = make_default_cli();
        cli.verbose = true;

        let result = execute(&cli, &store, "test", None, None, false, None, None);
        assert!(result.is_ok(), "Verbose search should succeed");
    }

    #[test]
    fn test_search_compaction_resolution() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        let mut note1 = store.create_note("Digest Note", None, &[], None).unwrap();
        note1.body = "This is the digest content.\n\nCompacts from qp-abc, qp-def".to_string();
        store.save_note(&mut note1).unwrap();

        let mut note2 = store.create_note("Source Note", None, &[], None).unwrap();
        note2.body = "This will be compacted into qp-digest".to_string();
        store.save_note(&mut note2).unwrap();

        let cli = make_default_cli();

        let result = execute(&cli, &store, "digest", None, None, false, None, None);
        assert!(
            result.is_ok(),
            "Search with compaction resolution should succeed"
        );
    }

    #[test]
    fn test_search_no_resolve_compaction() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        store.create_note("Test Note", None, &[], None).unwrap();

        let mut cli = make_default_cli();
        cli.no_resolve_compaction = true;

        let result = execute(&cli, &store, "test", None, None, false, None, None);
        assert!(
            result.is_ok(),
            "Search without compaction resolution should succeed"
        );
    }

    #[test]
    fn test_search_with_compaction_ids() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        let mut note1 = store.create_note("Digest Note", None, &[], None).unwrap();
        note1.body = "Digest content\n\nCompacts from qp-source".to_string();
        store.save_note(&mut note1).unwrap();

        let mut cli = make_default_cli();
        cli.with_compaction_ids = true;
        cli.compaction_depth = Some(1);

        let result = execute(&cli, &store, "digest", None, None, false, None, None);
        assert!(result.is_ok(), "Search with compaction IDs should succeed");
    }

    #[test]
    fn test_search_multiple_results() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        for i in 0..5 {
            store
                .create_note(&format!("Note {}", i), None, &[], None)
                .unwrap();
        }

        let cli = make_default_cli();

        let result = execute(&cli, &store, "note", None, None, false, None, None);
        assert!(
            result.is_ok(),
            "Search with multiple results should succeed"
        );
    }

    #[test]
    fn test_search_with_min_value_filter() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        let mut note1 = store
            .create_note("High Value Note", None, &[], None)
            .unwrap();
        note1.frontmatter.value = Some(80);
        store.save_note(&mut note1).unwrap();

        let mut note2 = store
            .create_note("Low Value Note", None, &[], None)
            .unwrap();
        note2.frontmatter.value = Some(30);
        store.save_note(&mut note2).unwrap();

        let mut note3 = store
            .create_note("Default Value Note", None, &[], None)
            .unwrap();
        note3.frontmatter.value = None;
        store.save_note(&mut note3).unwrap();

        let cli = make_default_cli();

        let result = execute(&cli, &store, "note", None, None, false, Some(50), None);
        assert!(
            result.is_ok(),
            "Search with min-value filter should succeed"
        );
    }

    #[test]
    fn test_search_sort_by_value() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();

        let mut note1 = store
            .create_note("High Value Note", None, &[], None)
            .unwrap();
        note1.frontmatter.value = Some(90);
        store.save_note(&mut note1).unwrap();

        let mut note2 = store
            .create_note("Low Value Note", None, &[], None)
            .unwrap();
        note2.frontmatter.value = Some(20);
        store.save_note(&mut note2).unwrap();

        let mut note3 = store
            .create_note("Medium Value Note", None, &[], None)
            .unwrap();
        note3.frontmatter.value = Some(60);
        store.save_note(&mut note3).unwrap();

        let cli = make_default_cli();

        let result = execute(&cli, &store, "note", None, None, false, None, Some("value"));
        assert!(result.is_ok(), "Search with --sort value should succeed");
    }
}
