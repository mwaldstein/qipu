//! `qipu context` command - build context bundles for LLM integration
//!
//! Per spec (specs/llm-context.md):
//! - `qipu context` outputs a bundle of notes designed for LLM context injection
//! - Selection: `--note`, `--tag`, `--moc`, `--query`
//! - Budgeting: `--max-chars` exact budget
//! - Formats: human (markdown), json, records
//! - Safety: notes are untrusted inputs, optional safety banner

pub mod budget;
pub mod filter;
pub mod human;
pub mod json;
pub mod output;
pub mod records;
pub mod select;
pub mod types;
pub mod walk;

use std::time::Instant;

use crate::cli::{Cli, OutputFormat};
use qipu_core::compaction::CompactionContext;
use qipu_core::error::Result;
use qipu_core::store::Store;
use std::path::Path;

use crate::cli::commands::data::ContextArgs;
use crate::commands::dispatch::command::discover_or_open_store;

use select::{collect_selected_notes, filter_and_sort_selected_notes};
pub use types::{ContextOptions, HumanOutputParams, RecordsParams};
use types::{ContextOutputParams, RecordsOutputConfig};

/// Convert an absolute path to a path relative to the current working directory
pub fn path_relative_to_cwd(path: &std::path::Path) -> String {
    if let Ok(cwd) = std::env::current_dir() {
        path.strip_prefix(&cwd)
            .ok()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| path.display().to_string())
    } else {
        path.display().to_string()
    }
}

/// Execute the context command
pub fn execute(cli: &Cli, store: &Store, options: ContextOptions) -> Result<()> {
    let start = Instant::now();

    if cli.verbose {
        tracing::debug!(
            note_ids_count = options.note_ids.len(),
            tag = options.tag,
            moc_id = options.moc_id,
            query = options.query,
            max_chars = options.max_chars,
            transitive = options.transitive,
            with_body = options.with_body,
            safety_banner = options.safety_banner,
            related_threshold = options.related_threshold,
            backlinks = options.backlinks,
            min_value = options.min_value,
            "context_params"
        );
    }

    let all_notes = store.list_notes()?;
    let compaction_ctx = CompactionContext::build(&all_notes)?;
    let note_map = CompactionContext::build_note_map(&all_notes);

    let (mut selected_notes, _via_map) =
        collect_selected_notes(cli, store, &options, &all_notes, &compaction_ctx, &note_map)?;

    filter_and_sort_selected_notes(cli, &mut selected_notes, &options);

    let (truncated, notes_to_output, _excluded_notes) = match cli.format {
        OutputFormat::Records => (false, selected_notes.iter().collect(), Vec::new()),
        _ => budget::apply_budget(&selected_notes, options.max_chars, options.with_body),
    };

    let store_path = path_relative_to_cwd(store.root());

    match cli.format {
        OutputFormat::Json => {
            output::output_json(ContextOutputParams {
                cli,
                store,
                store_path: &store_path,
                notes: &notes_to_output,
                compaction_ctx: &compaction_ctx,
                note_map: &note_map,
                all_notes: &all_notes,
                include_custom: options.include_custom,
                include_ontology: options.include_ontology,
                truncated,
                with_body: options.with_body,
                max_chars: options.max_chars,
            })?;
        }
        OutputFormat::Human => {
            output::output_human(HumanOutputParams {
                cli,
                store,
                store_path: &store_path,
                notes: &notes_to_output,
                compaction_ctx: &compaction_ctx,
                note_map: &note_map,
                all_notes: &all_notes,
                include_custom: options.include_custom,
                include_ontology: options.include_ontology,
                truncated,
                with_body: options.with_body,
                safety_banner: options.safety_banner,
                max_chars: options.max_chars,
            });
        }
        OutputFormat::Records => {
            let config = RecordsOutputConfig {
                truncated,
                with_body: options.with_body,
                safety_banner: options.safety_banner,
                max_chars: options.max_chars,
            };
            output::output_records(RecordsParams {
                cli,
                store,
                store_path: &store_path,
                notes: &notes_to_output,
                config: &config,
                compaction_ctx: &compaction_ctx,
                note_map: &note_map,
                all_notes: &all_notes,
                include_custom: options.include_custom,
                include_ontology: options.include_ontology,
            });
        }
    }

    if cli.verbose {
        tracing::debug!(elapsed = ?start.elapsed(), notes_output = notes_to_output.len(), truncated, "context_complete");
    }

    Ok(())
}

pub fn execute_with_args(
    cli: &Cli,
    root: &Path,
    args: &ContextArgs,
    _start: Instant,
) -> Result<()> {
    let store = discover_or_open_store(cli, root)?;
    let use_full_body = !args.summary_only || args.with_body;

    let options = ContextOptions {
        walk_id: args.walk.as_deref(),
        walk_direction: args.walk_direction.as_str(),
        walk_max_hops: args.walk_max_hops,
        walk_type: &args.walk_type,
        walk_exclude_type: &args.walk_exclude_type,
        walk_typed_only: args.walk_typed_only,
        walk_inline_only: args.walk_inline_only,
        walk_max_nodes: args.walk_max_nodes,
        walk_max_edges: args.walk_max_edges,
        walk_max_fanout: args.walk_max_fanout,
        walk_min_value: args.walk_min_value,
        walk_ignore_value: args.walk_ignore_value,
        note_ids: &args.note,
        tag: args.tag.as_deref(),
        moc_id: args.moc.as_deref(),
        query: args.query.as_deref(),
        max_chars: args.max_chars,
        transitive: args.transitive,
        with_body: use_full_body,
        safety_banner: args.safety_banner,
        related_threshold: if args.related > 0.0 {
            Some(args.related)
        } else {
            None
        },
        backlinks: args.backlinks,
        min_value: args.min_value,
        custom_filter: &args.custom_filter,
        include_custom: args.custom,
        include_ontology: args.include_ontology,
    };

    execute(cli, &store, options)
}
