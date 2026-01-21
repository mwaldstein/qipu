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
    excluded_notes: &[&SelectedNote],
) -> Result<()> {
    let start = Instant::now();

    if cli.verbose {
        debug!(
            notes_count = notes.len(),
            truncated, with_body, max_chars, "output_json"
        );
    }

    let mut final_truncated = truncated;
    let mut note_count = notes.len();

    loop {
        let output = build_json_output(
            cli,
            store_path,
            &notes[..note_count],
            final_truncated,
            with_body,
            compaction_ctx,
            note_map,
            all_notes,
            excluded_notes,
        );

        let output_str = serde_json::to_string_pretty(&output)?;

        if max_chars.is_none() || output_str.len() <= max_chars.unwrap() {
            println!("{}", output_str);
            if cli.verbose {
                debug!(elapsed = ?start.elapsed(), "output_json_complete");
            }
            return Ok(());
        }

        if note_count > 0 {
            note_count -= 1;
            final_truncated = true;
        } else {
            let minimal = serde_json::json!({
                "store": store_path,
                "truncated": true,
                "notes": []
            });
            println!("{}", serde_json::to_string_pretty(&minimal)?);
            if cli.verbose {
                debug!(elapsed = ?start.elapsed(), "output_json_complete");
            }
            return Ok(());
        }
    }
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
    excluded_notes: &[&SelectedNote],
) -> serde_json::Value {
    let mut output = serde_json::json!({
        "store": store_path,
        "truncated": truncated,
        "notes": notes.iter().map(|selected| {
            let note = selected.note;
            let content = if with_body {
                note.body.clone()
            } else {
                note.summary()
            };
            let mut json = serde_json::json!({
                "id": note.id(),
                "title": note.title(),
                "type": note.note_type().to_string(),
                "tags": note.frontmatter.tags,
                "path": note.path.as_ref().map(|p| p.display().to_string()),
                "content": content,
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
                        obj.insert("compaction_pct".to_string(), serde_json::json!(format!("{:.1}", pct)));
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
                        if let Some((compacted_notes, truncated)) = compaction_ctx.get_compacted_notes_expanded(
                            &note.frontmatter.id,
                            depth,
                            cli.compaction_max_nodes,
                            all_notes,
                        ) {
                            obj.insert(
                                "compacted_notes".to_string(),
                                serde_json::json!(
                                    compacted_notes
                                        .iter()
                                        .map(|n: &&Note| serde_json::json!({
                                            "id": n.id(),
                                            "title": n.title(),
                                            "type": n.note_type().to_string(),
                                            "tags": n.frontmatter.tags,
                                            "path": n.path.as_ref().map(|p| p.display().to_string()),
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
                                        }))
                                        .collect::<Vec<_>>()
                                ),
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
        }).collect::<Vec<_>>(),
    });

    // Add excluded notes if any
    if !excluded_notes.is_empty() {
        if let Some(obj) = output.as_object_mut() {
            obj.insert(
                "excluded_notes".to_string(),
                serde_json::json!(excluded_notes
                    .iter()
                    .map(|selected| serde_json::json!({
                        "id": selected.note.id(),
                        "title": selected.note.title(),
                    }))
                    .collect::<Vec<_>>()),
            );
        }
    }

    output
}
