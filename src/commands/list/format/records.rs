//! Records output formatting for list command

use crate::cli::Cli;
use crate::commands::format::status::format_custom_value;
use qipu_core::compaction::CompactionContext;
use qipu_core::format::build_compaction_annotations;
use qipu_core::records::escape_quotes;
use qipu_core::store::Store;
use std::collections::HashMap;

/// Output in records format
pub fn output_records(
    cli: &Cli,
    store: &Store,
    notes: &[qipu_core::note::Note],
    compaction_ctx: &CompactionContext,
    note_map: &HashMap<&str, &qipu_core::note::Note>,
    show_custom: bool,
) {
    println!(
        "H qipu=1 records=1 store={} mode=list notes={}",
        store.root().display(),
        notes.len()
    );

    for note in notes {
        output_note_record(cli, note, compaction_ctx, note_map, show_custom);
    }
}

/// Output a single note record
fn output_note_record(
    cli: &Cli,
    note: &qipu_core::note::Note,
    compaction_ctx: &CompactionContext,
    note_map: &HashMap<&str, &qipu_core::note::Note>,
    show_custom: bool,
) {
    let tags_csv = if note.frontmatter.tags.is_empty() {
        "-".to_string()
    } else {
        note.frontmatter.tags.join(",")
    };

    let annotations = build_compaction_annotations(note.id(), compaction_ctx, note_map);

    println!(
        "N {} {} \"{}\" tags={}{}",
        note.id(),
        note.note_type(),
        escape_quotes(note.title()),
        tags_csv,
        annotations
    );

    if show_custom && !note.frontmatter.custom.is_empty() {
        for (key, value) in &note.frontmatter.custom {
            println!("C {} {}={}", note.id(), key, format_custom_value(value));
        }
    }

    let compacts_count = compaction_ctx.get_compacts_count(&note.frontmatter.id);
    if cli.with_compaction_ids && compacts_count > 0 {
        let depth = cli.compaction_depth.unwrap_or(1);
        if let Some((ids, truncated)) =
            compaction_ctx.get_compacted_ids(&note.frontmatter.id, depth, cli.compaction_max_nodes)
        {
            for id in &ids {
                println!("D compacted {} from={}", id, note.id());
            }

            if truncated {
                println!(
                    "D compacted_truncated max={} total={}",
                    cli.compaction_max_nodes.unwrap_or(ids.len()),
                    compacts_count
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_records_empty() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let store =
            Store::init(temp_dir.path(), qipu_core::store::InitOptions::default()).unwrap();
        let cli = create_test_cli();

        let all_notes = store.list_notes().unwrap();
        let compaction_ctx = CompactionContext::build(&all_notes).unwrap();
        let note_map = CompactionContext::build_note_map(&all_notes);

        output_records(&cli, &store, &[], &compaction_ctx, &note_map, false);
    }

    #[test]
    fn test_output_records_single_note() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let store =
            Store::init(temp_dir.path(), qipu_core::store::InitOptions::default()).unwrap();
        let cli = create_test_cli();

        let note = store
            .create_note("Test Note", None, &["tag1".to_string()], None)
            .unwrap();

        let all_notes = store.list_notes().unwrap();
        let compaction_ctx = CompactionContext::build(&all_notes).unwrap();
        let note_map = CompactionContext::build_note_map(&all_notes);

        output_records(&cli, &store, &[note], &compaction_ctx, &note_map, false);
    }

    fn create_test_cli() -> Cli {
        Cli {
            root: None,
            store: None,
            format: crate::cli::OutputFormat::Records,
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
}
