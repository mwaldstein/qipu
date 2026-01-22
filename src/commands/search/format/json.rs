//! JSON output formatting for search command

use crate::cli::Cli;
use crate::lib::compaction::CompactionContext;
use crate::lib::store::Store;
use std::collections::HashMap;

/// Output in JSON format
pub fn output_json(
    cli: &Cli,
    _store: &Store,
    results: &[crate::lib::index::SearchResult],
    compaction_ctx: &Option<CompactionContext>,
    notes_cache: &HashMap<String, crate::lib::note::Note>,
    compaction_note_map: &Option<std::collections::HashMap<&str, &crate::lib::note::Note>>,
) -> crate::lib::error::Result<()> {
    let output: Vec<_> = results
        .iter()
        .map(|r| {
            let mut obj = serde_json::json!({
                "id": r.id,
                "title": r.title,
                "type": r.note_type.to_string(),
                "tags": r.tags,
                
                "match_context": r.match_context,
                "relevance": r.relevance,
            });

            if let Some(via) = &r.via {
                if let Some(obj_mut) = obj.as_object_mut() {
                    obj_mut.insert("via".to_string(), serde_json::json!(via));
                }
            }

            if let Some(ref ctx) = compaction_ctx {
                let compacts_count = ctx.get_compacts_count(&r.id);
                if compacts_count > 0 {
                    if let Some(obj_mut) = obj.as_object_mut() {
                        obj_mut.insert("compacts".to_string(), serde_json::json!(compacts_count));

                        if let Some(note) = notes_cache.get(&r.id) {
                            if let Some(ref note_map) = compaction_note_map {
                                if let Some(pct) = ctx.get_compaction_pct(note, note_map) {
                                    obj_mut.insert(
                                        "compaction_pct".to_string(),
                                        serde_json::json!(format!("{:.1}", pct)),
                                    );
                                }
                            }
                        }

                        if cli.with_compaction_ids {
                            let depth = cli.compaction_depth.unwrap_or(1);
                            if let Some((ids, truncated)) =
                                ctx.get_compacted_ids(&r.id, depth, cli.compaction_max_nodes)
                            {
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

            obj
        })
        .collect();
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
