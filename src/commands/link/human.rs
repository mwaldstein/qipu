use super::LinkEntry;
use crate::cli::Cli;
use crate::lib::compaction::CompactionContext;

/// Output in human-readable format
pub fn output_human(
    cli: &Cli,
    entries: &[LinkEntry],
    display_id: &str,
    compaction_ctx: Option<&CompactionContext>,
) {
    if entries.is_empty() {
        if !cli.quiet {
            println!("No links found for {}", display_id);
        }
    } else {
        for entry in entries {
            let dir_arrow = match entry.direction.as_str() {
                "out" => "->",
                "in" => "<-",
                _ => "--",
            };
            let title_part = entry
                .title
                .as_ref()
                .map(|t| format!(" \"{}\"", t))
                .unwrap_or_default();
            println!(
                "{} {} {} [{}] ({})",
                dir_arrow, entry.id, title_part, entry.link_type, entry.source
            );

            // Show compacted IDs if --with-compaction-ids is set
            if cli.with_compaction_ids {
                if let Some(ref ctx) = compaction_ctx {
                    let compacts_count = ctx.get_compacts_count(&entry.id);
                    if compacts_count > 0 {
                        let depth = cli.compaction_depth.unwrap_or(1);
                        if let Some((ids, truncated)) =
                            ctx.get_compacted_ids(&entry.id, depth, cli.compaction_max_nodes)
                        {
                            let ids_str = ids.join(", ");
                            let suffix = if truncated {
                                let max = cli.compaction_max_nodes.unwrap_or(ids.len());
                                format!(" (truncated, showing {} of {})", max, compacts_count)
                            } else {
                                String::new()
                            };
                            println!("  Compacted: {}{}", ids_str, suffix);
                        }
                    }
                }
            }
        }
    }
}
