use super::ExportMode;
use crate::cli::Cli;
use crate::commands::export::ExportOptions;
use crate::lib::compaction::CompactionContext;
use crate::lib::error::Result;
use crate::lib::note::Note;
use crate::lib::store::Store;

pub fn export_json(
    notes: &[Note],
    store: &Store,
    options: &ExportOptions,
    cli: &Cli,
    compaction_ctx: &CompactionContext,
    all_notes: &[Note],
) -> Result<String> {
    // Build note map for efficient lookups (avoid O(n²) when calculating compaction pct)
    let note_map = CompactionContext::build_note_map(all_notes);

    let mode_str = match options.mode {
        ExportMode::Bundle => "bundle",
        ExportMode::Outline => "outline",
        ExportMode::Bibliography => "bibliography",
    };

    let output = serde_json::json!({
        "store": store.root().display().to_string(),
        "mode": mode_str,
        "notes": notes
            .iter()
            .map(|note| {
                let mut obj = serde_json::json!({
                    "id": note.id(),
                    "title": note.title(),
                    "type": note.note_type().to_string(),
                    "tags": note.frontmatter.tags,
                    "path": note.path.as_ref().map(|p| p.display().to_string()),
                    "created": note.frontmatter.created,
                    "updated": note.frontmatter.updated,
                    "content": note.body,
                    "sources": note.frontmatter.sources.iter().map(|s| {
                        let mut obj = serde_json::json!({
                            "url": s.url,
                        });
                        if let Some(title) = &s.title {
                            obj["title"] = serde_json::json!(title);
                        }
                        if let Some(accessed) = &s.accessed {
                            obj["accessed"] = serde_json::json!(accessed);
                        }
                        obj
                    }).collect::<Vec<_>>(),
                });

                // Add compaction annotations for digest notes
                let compacts_count = compaction_ctx.get_compacts_count(&note.frontmatter.id);
                if compacts_count > 0 {
                    if let Some(obj_mut) = obj.as_object_mut() {
                        obj_mut.insert("compacts".to_string(), serde_json::json!(compacts_count));
                        if let Some(pct) = compaction_ctx.get_compaction_pct(note, &note_map) {
                            obj_mut.insert(
                                "compaction_pct".to_string(),
                                serde_json::json!(format!("{:.1}", pct)),
                            );
                        }

                        if cli.with_compaction_ids {
                            let depth = cli.compaction_depth.unwrap_or(1);
                            if let Some((ids, _truncated)) = compaction_ctx.get_compacted_ids(
                                &note.frontmatter.id,
                                depth,
                                cli.compaction_max_nodes,
                            ) {
                                obj_mut.insert(
                                    "compacted_ids".to_string(),
                                    serde_json::json!(ids),
                                );
                            }
                        }
                    }
                }

                obj
            })
            .collect::<Vec<_>>(),
    });

    Ok(serde_json::to_string_pretty(&output)?)
}
