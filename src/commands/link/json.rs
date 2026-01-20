use super::LinkEntry;
use crate::cli::Cli;
use crate::lib::compaction::CompactionContext;
use crate::lib::error::Result;

/// Output in JSON format
pub fn output_json(
    cli: &Cli,
    entries: &[LinkEntry],
    compaction_ctx: Option<&CompactionContext>,
) -> Result<()> {
    let json_output: Vec<serde_json::Value> = entries
        .iter()
        .map(|entry| {
            let mut json = serde_json::json!({
                "direction": entry.direction,
                "id": entry.id,
                "type": entry.link_type,
                "source": entry.source,
            });
            if let Some(title) = &entry.title {
                if let Some(obj_mut) = json.as_object_mut() {
                    obj_mut.insert("title".to_string(), serde_json::json!(title));
                }
            }

            // Add compacted IDs if --with-compaction-ids is set
            if cli.with_compaction_ids {
                if let Some(ref ctx) = compaction_ctx {
                    let compacts_count = ctx.get_compacts_count(&entry.id);
                    if compacts_count > 0 {
                        let depth = cli.compaction_depth.unwrap_or(1);
                        if let Some((ids, _truncated)) =
                            ctx.get_compacted_ids(&entry.id, depth, cli.compaction_max_nodes)
                        {
                            if let Some(obj_mut) = json.as_object_mut() {
                                obj_mut.insert("compacted_ids".to_string(), serde_json::json!(ids));
                            }
                        }
                    }
                }
            }

            json
        })
        .collect();
    println!("{}", serde_json::to_string_pretty(&json_output)?);
    Ok(())
}
