//! Link tree command

use crate::cli::{Cli, OutputFormat};
use crate::commands::traversal;
use qipu_core::error::Result;
use qipu_core::store::Store;

use super::{
    human::output_tree_human, json::output_tree_json, records_tree::output_tree_records,
    LinkOutputContext, TreeOptions,
};

/// Execute the link tree command
pub fn execute(cli: &Cli, store: &Store, id_or_path: &str, opts: TreeOptions) -> Result<()> {
    use std::time::Instant;
    let start = Instant::now();

    let traversal_ctx = traversal::build_context(cli, store, id_or_path)?;

    if cli.verbose {
        tracing::debug!(elapsed = ?start.elapsed(), "load_indexes");
    }

    let result = traversal::run_tree(cli, store, &traversal_ctx, &opts)?;
    let note_map = traversal_ctx.note_map();

    match cli.format {
        OutputFormat::Json => {
            output_tree_json(
                cli,
                &result,
                traversal_ctx.compaction_ctx.as_ref(),
                note_map.as_ref(),
                &traversal_ctx.all_notes,
            )?;
        }
        OutputFormat::Human => {
            output_tree_human(
                cli,
                &result,
                &traversal_ctx.index,
                store,
                traversal_ctx.compaction_ctx.as_ref(),
                note_map.as_ref(),
                &traversal_ctx.all_notes,
            );
        }
        OutputFormat::Records => {
            let ctx = LinkOutputContext::new(
                store,
                &traversal_ctx.index,
                cli,
                traversal_ctx.compaction_ctx.as_ref(),
                note_map.as_ref(),
                opts.max_chars,
                &traversal_ctx.all_notes,
            );
            output_tree_records(&result, &ctx, &opts);
        }
    }

    Ok(())
}
