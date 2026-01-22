use super::types::SelectedNote;
use crate::cli::Cli;
use crate::lib::compaction::CompactionContext;
use crate::lib::error::Result;
use crate::lib::note::Note;
use std::collections::HashMap;
use std::time::Instant;
use tracing::debug;

/// Output in JSON format
#[allow(clippy::too_many_arguments)]
pub fn output_json(
    cli: &Cli,
    store_path: &str,
    notes: &[&SelectedNote],
    truncated: bool,
    with_body: bool,
    compaction_ctx: &CompactionContext,
    note_map: &HashMap<&str, &Note>,
    all_notes: &[Note],
    max_chars: Option<usize>,
    _excluded_notes: &[&SelectedNote],
    include_custom: bool,
) -> Result<()> {
    let start = Instant::now();

    if cli.verbose {
        debug!(
            notes_count = notes.len(),
            truncated, with_body, max_chars, include_custom, "output_json"
        );
    }

    let output = build_json_output(
        cli,
        store_path,
        notes,
        truncated,
        with_body,
        compaction_ctx,
        note_map,
        all_notes,
        max_chars,
        include_custom,
    );

    println!("{}", serde_json::to_string_pretty(&output)?);

    if cli.verbose {
        debug!(elapsed = ?start.elapsed(), "output_json_complete");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn build_json_output(
    cli: &Cli,
    store_path: &str,
    notes: &[&SelectedNote],
    truncated: bool,
    with_body: bool,
    compaction_ctx: &CompactionContext,
    note_map: &HashMap<&str, &Note>,
    all_notes: &[Note],
    max_chars: Option<usize>,
    include_custom: bool,
) -> serde_json::Value {
    let mut json_notes: Vec<serde_json::Value> = Vec::new();
    let mut actual_truncated = false;
    let mut estimated_size = store_path.len() + 50;

    for (_idx, selected) in notes.iter().enumerate() {
        let note = selected.note;
        let content = if with_body {
            note.body.clone()
        } else {
            note.summary()
        };

        let note_json_obj = build_note_json(
            cli,
            note,
            selected,
            compaction_ctx,
            note_map,
            all_notes,
            include_custom,
            &content,
        );

        let note_json_str = serde_json::to_string(&note_json_obj).unwrap_or_default();
        let note_size = note_json_str.len() + 10;

        if let Some(budget) = max_chars {
            let remaining = budget.saturating_sub(estimated_size);

            if truncated || note_size > remaining {
                actual_truncated = true;
                let marker = "…[truncated]";
                let marker_len = marker.len();

                let available_for_content =
                    remaining.saturating_sub(note_size - content.len() + marker_len);

                let (final_content, content_truncated) =
                    if content.len() > marker.len() + 10 && available_for_content > marker.len() {
                        let truncated_content_len =
                            available_for_content.min(content.len() - marker_len);
                        (
                            format!("{} {}", &content[..truncated_content_len], marker),
                            true,
                        )
                    } else {
                        (marker.to_string(), true)
                    };

                let mut truncated_note_json = build_note_json(
                    cli,
                    note,
                    selected,
                    compaction_ctx,
                    note_map,
                    all_notes,
                    include_custom,
                    &final_content,
                );

                if let Some(obj) = truncated_note_json.as_object_mut() {
                    obj.insert(
                        "content_truncated".to_string(),
                        serde_json::json!(content_truncated),
                    );
                }

                let final_json_str =
                    serde_json::to_string(&truncated_note_json).unwrap_or_default();
                if estimated_size + final_json_str.len() <= budget {
                    json_notes.push(truncated_note_json);
                }
                break;
            }

            if estimated_size + note_size > budget {
                actual_truncated = true;
                break;
            }
        }

        estimated_size += note_size;
        json_notes.push(note_json_obj);
    }

    serde_json::json!({
        "store": store_path,
        "truncated": truncated || actual_truncated,
        "notes": json_notes,
    })
}

#[allow(clippy::too_many_arguments)]
fn build_note_json(
    cli: &Cli,
    note: &Note,
    selected: &SelectedNote,
    compaction_ctx: &CompactionContext,
    note_map: &HashMap<&str, &Note>,
    all_notes: &[Note],
    include_custom: bool,
    content: &str,
) -> serde_json::Value {
    let mut json = serde_json::json!({
        "id": note.id(),
        "title": note.title(),
        "type": note.note_type().to_string(),
        "tags": note.frontmatter.tags,
        "content": content,
        "content_truncated": false,
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
        "source": note.frontmatter.source,
        "author": note.frontmatter.author,
        "generated_by": note.frontmatter.generated_by,
        "prompt_hash": note.frontmatter.prompt_hash,
        "verified": note.frontmatter.verified,
    });

    if include_custom && !note.frontmatter.custom.is_empty() {
        if let Some(obj) = json.as_object_mut() {
            let custom_json: serde_json::Map<String, serde_json::Value> = note
                .frontmatter
                .custom
                .iter()
                .map(|(k, v)| {
                    let json_value = serde_json::to_value(v).unwrap_or(serde_json::Value::Null);
                    (k.clone(), json_value)
                })
                .collect();
            obj.insert("custom".to_string(), serde_json::Value::Object(custom_json));
        }
    }

    if let Some(via) = &selected.via {
        if let Some(obj) = json.as_object_mut() {
            obj.insert("via".to_string(), serde_json::json!(via));
        }
    }

    let compacts_count = compaction_ctx.get_compacts_count(&note.frontmatter.id);
    if compacts_count > 0 {
        if let Some(obj) = json.as_object_mut() {
            obj.insert("compacts".to_string(), serde_json::json!(compacts_count));

            if let Some(pct) = compaction_ctx.get_compaction_pct(note, note_map) {
                obj.insert(
                    "compaction_pct".to_string(),
                    serde_json::json!(format!("{:.1}", pct)),
                );
            }

            if cli.with_compaction_ids {
                let depth = cli.compaction_depth.unwrap_or(1);
                if let Some((ids, truncated)) = compaction_ctx.get_compacted_ids(
                    &note.frontmatter.id,
                    depth,
                    cli.compaction_max_nodes,
                ) {
                    obj.insert("compacted_ids".to_string(), serde_json::json!(ids));
                    if truncated {
                        obj.insert(
                            "compacted_ids_truncated".to_string(),
                            serde_json::json!(true),
                        );
                    }
                }
            }

            if cli.expand_compaction {
                let depth = cli.compaction_depth.unwrap_or(1);
                if let Some((compacted_notes, truncated)) = compaction_ctx
                    .get_compacted_notes_expanded(
                        &note.frontmatter.id,
                        depth,
                        cli.compaction_max_nodes,
                        all_notes,
                    )
                {
                    obj.insert(
                        "compacted_notes".to_string(),
                        serde_json::json!(compacted_notes
                            .iter()
                            .map(|n: &&Note| {
                                let mut note_json = serde_json::json!({
                                    "id": n.id(),
                                    "title": n.title(),
                                    "type": n.note_type().to_string(),
                                    "tags": n.frontmatter.tags,

                                    "content": n.body,
                                    "sources": n.frontmatter.sources.iter().map(|s| {
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
                                if include_custom && !n.frontmatter.custom.is_empty() {
                                    if let Some(obj) = note_json.as_object_mut() {
                                        let custom_json: serde_json::Map<
                                            String,
                                            serde_json::Value,
                                        > = n
                                            .frontmatter
                                            .custom
                                            .iter()
                                            .map(|(k, v)| {
                                                let json_value = serde_json::to_value(v)
                                                    .unwrap_or(serde_json::Value::Null);
                                                (k.clone(), json_value)
                                            })
                                            .collect();
                                        obj.insert(
                                            "custom".to_string(),
                                            serde_json::Value::Object(custom_json),
                                        );
                                    }
                                }
                                note_json
                            })
                            .collect::<Vec<_>>()),
                    );
                    if truncated {
                        obj.insert(
                            "compacted_notes_truncated".to_string(),
                            serde_json::json!(true),
                        );
                    }
                }
            }
        }
    }

    json
}
