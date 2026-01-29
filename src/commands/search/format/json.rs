//! JSON output formatting for search command

use crate::cli::Cli;
use qipu_core::compaction::CompactionContext;
use qipu_core::format::add_compaction_to_json;
use qipu_core::format::CompactionOutputOptions;
use qipu_core::store::Store;
use std::collections::HashMap;

/// Output in JSON format
pub fn output_json(
    cli: &Cli,
    _store: &Store,
    results: &[qipu_core::index::SearchResult],
    compaction_ctx: &Option<CompactionContext>,
    _notes_cache: &HashMap<String, qipu_core::note::Note>,
    compaction_note_map: &Option<std::collections::HashMap<&str, &qipu_core::note::Note>>,
) -> qipu_core::error::Result<()> {
    let opts = CompactionOutputOptions {
        with_compaction_ids: cli.with_compaction_ids,
        compaction_depth: cli.compaction_depth,
        compaction_max_nodes: cli.compaction_max_nodes,
    };

    let output: Vec<_> = results
        .iter()
        .map(|r| {
            let mut obj = serde_json::json!({
                "id": r.id,
                "title": r.title,
                "type": r.note_type.to_string(),
                "tags": r.tags,
                "path": r.path,

                "match_context": r.match_context,
                "relevance": r.relevance,
            });

            if let Some(ref created) = r.created {
                if let Some(obj_mut) = obj.as_object_mut() {
                    obj_mut.insert("created".to_string(), serde_json::json!(created));
                }
            }

            if let Some(ref updated) = r.updated {
                if let Some(obj_mut) = obj.as_object_mut() {
                    obj_mut.insert("updated".to_string(), serde_json::json!(updated));
                }
            }

            if let Some(via) = &r.via {
                if let Some(obj_mut) = obj.as_object_mut() {
                    obj_mut.insert("via".to_string(), serde_json::json!(via));
                }
            }

            if let Some(ref ctx) = compaction_ctx {
                if let Some(ref note_map) = compaction_note_map {
                    add_compaction_to_json(&opts, &r.id, &mut obj, ctx, note_map);
                }
            }

            obj
        })
        .collect();
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
