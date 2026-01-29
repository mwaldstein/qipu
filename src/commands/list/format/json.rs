//! JSON output formatting for list command

use crate::cli::Cli;
use qipu_core::compaction::CompactionContext;
use qipu_core::error::Result;
use qipu_core::format::add_compaction_to_json;
use qipu_core::format::CompactionOutputOptions;
use qipu_core::store::Store;
use std::collections::HashMap;

/// Output in JSON format
pub fn output_json(
    cli: &Cli,
    _store: &Store,
    notes: &[qipu_core::note::Note],
    compaction_ctx: &CompactionContext,
    note_map: &HashMap<&str, &qipu_core::note::Note>,
    show_custom: bool,
) -> Result<()> {
    let opts = CompactionOutputOptions {
        with_compaction_ids: cli.with_compaction_ids,
        compaction_depth: cli.compaction_depth,
        compaction_max_nodes: cli.compaction_max_nodes,
    };

    let output: Vec<_> = notes
        .iter()
        .map(|n| {
            let mut json = serde_json::json!({
                "id": n.id(),
                "title": n.title(),
                "type": n.note_type().to_string(),
                "tags": n.frontmatter.tags,

                "created": n.frontmatter.created,
                "updated": n.frontmatter.updated,
                "path": n.path,
            });

            add_compaction_to_json(&opts, n.id(), &mut json, compaction_ctx, note_map);

            if show_custom && !n.frontmatter.custom.is_empty() {
                if let Some(obj) = json.as_object_mut() {
                    obj.insert(
                        "custom".to_string(),
                        serde_json::to_value(&n.frontmatter.custom)
                            .unwrap_or(serde_json::json!({})),
                    );
                }
            }

            json
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
