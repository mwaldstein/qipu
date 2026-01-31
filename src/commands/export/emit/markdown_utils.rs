//! Shared Markdown export utilities

use crate::cli::Cli;
use qipu_core::compaction::CompactionContext;
use qipu_core::note::Note;

/// Add compaction metadata to markdown output
pub fn add_compaction_metadata(
    output: &mut String,
    note: &Note,
    cli: &Cli,
    compaction_ctx: &CompactionContext,
    note_map: &std::collections::HashMap<&str, &Note>,
) {
    let compacts_count = compaction_ctx.get_compacts_count(&note.frontmatter.id);
    if compacts_count > 0 {
        output.push_str(&format!("**Compaction:** compacts={}", compacts_count));

        if let Some(pct) = compaction_ctx.get_compaction_pct(note, note_map) {
            output.push_str(&format!(" compaction={:.0}%", pct));
        }
        output.push_str("\n\n");

        if cli.with_compaction_ids {
            let depth = cli.compaction_depth.unwrap_or(1);
            if let Some((ids, truncated)) = compaction_ctx.get_compacted_ids(
                &note.frontmatter.id,
                depth,
                cli.compaction_max_nodes,
            ) {
                let ids_str = ids.join(", ");
                let suffix = if truncated {
                    let max = cli.compaction_max_nodes.unwrap_or(ids.len());
                    format!(" (truncated, showing {} of {})", max, compacts_count)
                } else {
                    String::new()
                };
                output.push_str(&format!("**Compacted IDs:** {}{}\n\n", ids_str, suffix));
            }
        }
    }
}
