use super::LinkEntry;
use crate::cli::Cli;
use crate::lib::compaction::CompactionContext;
use crate::lib::error::Result;
use crate::lib::graph::PathResult;

/// Output in JSON format
pub fn output_json(
    cli: &Cli,
    entries: &[LinkEntry],
    compaction_ctx: Option<&CompactionContext>,
    note_map: Option<&std::collections::HashMap<&str, &crate::lib::note::Note>>,
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
            if let Some(via) = &entry.via {
                if let Some(obj_mut) = json.as_object_mut() {
                    obj_mut.insert("via".to_string(), serde_json::json!(via));
                }
            }

            if let Some(ctx) = compaction_ctx {
                let compacts_count = ctx.get_compacts_count(&entry.id);
                if compacts_count > 0 {
                    if let Some(obj_mut) = json.as_object_mut() {
                        obj_mut.insert("compacts".to_string(), serde_json::json!(compacts_count));

                        if let Some(ref map) = note_map {
                            if let Some(note) = map.get(entry.id.as_str()) {
                                if let Some(pct) = ctx.get_compaction_pct(note, map) {
                                    obj_mut.insert(
                                        "compaction_pct".to_string(),
                                        serde_json::json!(format!("{:.1}", pct)),
                                    );
                                }
                            }
                        }
                    }

                    if cli.with_compaction_ids {
                        let depth = cli.compaction_depth.unwrap_or(1);
                        if let Some((ids, truncated)) =
                            ctx.get_compacted_ids(&entry.id, depth, cli.compaction_max_nodes)
                        {
                            if let Some(obj_mut) = json.as_object_mut() {
                                obj_mut.insert("compacted_ids".to_string(), serde_json::json!(ids));
                                if truncated {
                                    obj_mut.insert(
                                        "compacted_ids_truncated".to_string(),
                                        serde_json::json!(true),
                                    );
                                }
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

/// Output path in JSON format
pub fn output_path_json(
    cli: &Cli,
    result: &PathResult,
    compaction_ctx: Option<&CompactionContext>,
    note_map: Option<&std::collections::HashMap<&str, &crate::lib::note::Note>>,
) -> Result<()> {
    let mut json_result = serde_json::to_value(result)?;
    if let Some(ctx) = compaction_ctx {
        if let Some(notes) = json_result.get_mut("notes").and_then(|n| n.as_array_mut()) {
            for note in notes {
                let id = if let Some(id) = note.get("id").and_then(|i| i.as_str()) {
                    id.to_owned()
                } else {
                    continue;
                };

                let compacts_count = ctx.get_compacts_count(&id);
                if compacts_count > 0 {
                    if let Some(obj_mut) = note.as_object_mut() {
                        obj_mut.insert("compacts".to_string(), serde_json::json!(compacts_count));

                        if let Some(ref map) = note_map {
                            if let Some(note_ref) = map.get(id.as_str()) {
                                if let Some(pct) = ctx.get_compaction_pct(note_ref, map) {
                                    obj_mut.insert(
                                        "compaction_pct".to_string(),
                                        serde_json::json!(format!("{:.1}", pct)),
                                    );
                                }
                            }
                        }
                    }

                    if cli.with_compaction_ids {
                        let depth = cli.compaction_depth.unwrap_or(1);
                        if let Some((ids, truncated)) =
                            ctx.get_compacted_ids(&id, depth, cli.compaction_max_nodes)
                        {
                            if let Some(obj_mut) = note.as_object_mut() {
                                obj_mut.insert("compacted_ids".to_string(), serde_json::json!(ids));
                                if truncated {
                                    obj_mut.insert(
                                        "compacted_ids_truncated".to_string(),
                                        serde_json::json!(true),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    println!("{}", serde_json::to_string_pretty(&json_result)?);
    Ok(())
}
