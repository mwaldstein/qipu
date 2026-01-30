use super::types::{BuildNoteJsonParams, ContextOutputParams};
use qipu_core::error::Result;
use qipu_core::note::Note;
use qipu_core::ontology::Ontology;
use std::time::Instant;
use tracing::debug;

/// Output in JSON format
pub fn output_json(params: ContextOutputParams) -> Result<()> {
    let start = Instant::now();

    if params.cli.verbose {
        debug!(
            notes_count = params.notes.len(),
            truncated = params.truncated,
            with_body = params.with_body,
            max_chars = params.max_chars,
            include_custom = params.include_custom,
            include_ontology = params.include_ontology,
            "output_json"
        );
    }

    let output = build_json_output(&params);

    println!("{}", serde_json::to_string_pretty(&output)?);

    if params.cli.verbose {
        debug!(elapsed = ?start.elapsed(), "output_json_complete");
    }
    Ok(())
}

fn build_json_output(params: &ContextOutputParams) -> serde_json::Value {
    let mut json_notes: Vec<serde_json::Value> = Vec::new();
    let mut actual_truncated = false;
    let mut estimated_size = params.store_path.len() + 50;

    for selected in params.notes.iter() {
        let note = selected.note;
        let content = if params.with_body {
            note.body.clone()
        } else {
            note.summary()
        };

        let note_json_obj = build_note_json(BuildNoteJsonParams {
            cli: params.cli,
            note,
            selected,
            compaction_ctx: params.compaction_ctx,
            note_map: params.note_map,
            all_notes: params.all_notes,
            include_custom: params.include_custom,
            content: &content,
        });

        let note_json_str = serde_json::to_string(&note_json_obj).unwrap_or_default();
        let note_size = note_json_str.len() + 10;

        if let Some(budget) = params.max_chars {
            let remaining = budget.saturating_sub(estimated_size);

            if params.truncated || note_size > remaining {
                actual_truncated = true;
                let marker = "…[truncated]";
                let marker_len = marker.len();

                let available_for_content =
                    remaining.saturating_sub(note_size - content.len() + marker_len);

                let (final_content, content_truncated) =
                    if content.len() > marker.len() + 10 && available_for_content > marker.len() {
                        let truncated_content_len =
                            available_for_content.min(content.len() - marker.len());
                        (
                            format!("{} {}", &content[..truncated_content_len], marker),
                            true,
                        )
                    } else {
                        (marker.to_string(), true)
                    };

                let mut truncated_note_json = build_note_json(BuildNoteJsonParams {
                    cli: params.cli,
                    note,
                    selected,
                    compaction_ctx: params.compaction_ctx,
                    note_map: params.note_map,
                    all_notes: params.all_notes,
                    include_custom: params.include_custom,
                    content: &final_content,
                });

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

    let mut output = serde_json::json!({
        "store": params.store_path,
        "truncated": params.truncated || actual_truncated,
        "notes": json_notes,
    });

    if params.include_ontology {
        let config = params.store.config();
        let ontology = Ontology::from_config_with_graph(&config.ontology, &config.graph);

        let note_types = ontology.note_types();
        let link_types = ontology.link_types();

        let note_type_objs: Vec<_> = note_types
            .iter()
            .map(|nt| {
                let type_config = config.ontology.note_types.get(nt);
                serde_json::json!({
                    "name": nt,
                    "description": type_config.and_then(|c| c.description.clone()),
                    "usage": type_config.and_then(|c| c.usage.clone()),
                })
            })
            .collect();

        let link_type_objs: Vec<_> = link_types
            .iter()
            .map(|lt| {
                let inverse = ontology.get_inverse(lt);
                let type_config = config.ontology.link_types.get(lt);
                serde_json::json!({
                    "name": lt,
                    "inverse": inverse,
                    "description": type_config.and_then(|c| c.description.clone()),
                    "usage": type_config.and_then(|c| c.usage.clone()),
                })
            })
            .collect();

        if let Some(obj) = output.as_object_mut() {
            obj.insert(
                "ontology".to_string(),
                serde_json::json!({
                    "mode": format_mode(params.store.config().ontology.mode),
                    "note_types": note_type_objs,
                    "link_types": link_type_objs,
                }),
            );
        }
    }

    output
}

fn format_mode(mode: qipu_core::config::OntologyMode) -> &'static str {
    match mode {
        qipu_core::config::OntologyMode::Default => "default",
        qipu_core::config::OntologyMode::Extended => "extended",
        qipu_core::config::OntologyMode::Replacement => "replacement",
    }
}

fn build_note_json(params: BuildNoteJsonParams) -> serde_json::Value {
    let mut json = serde_json::json!({
        "id": params.note.id(),
        "title": params.note.title(),
        "type": params.note.note_type().to_string(),
        "tags": params.note.frontmatter.tags,
        "content": params.content,
        "content_truncated": false,
        "sources": params.note.frontmatter.sources.iter().map(|s| {
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
        "source": params.note.frontmatter.source,
        "author": params.note.frontmatter.author,
        "generated_by": params.note.frontmatter.generated_by,
        "prompt_hash": params.note.frontmatter.prompt_hash,
        "verified": params.note.frontmatter.verified,
    });

    if params.include_custom && !params.note.frontmatter.custom.is_empty() {
        if let Some(obj) = json.as_object_mut() {
            let custom_json: serde_json::Map<String, serde_json::Value> = params
                .note
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

    if let Some(via) = &params.selected.via {
        if let Some(obj) = json.as_object_mut() {
            obj.insert("via".to_string(), serde_json::json!(via));
        }
    }

    let compacts_count = params
        .compaction_ctx
        .get_compacts_count(&params.note.frontmatter.id);
    if compacts_count > 0 {
        if let Some(obj) = json.as_object_mut() {
            obj.insert("compacts".to_string(), serde_json::json!(compacts_count));

            if let Some(pct) = params
                .compaction_ctx
                .get_compaction_pct(params.note, params.note_map)
            {
                obj.insert(
                    "compaction_pct".to_string(),
                    serde_json::json!(format!("{:.1}", pct)),
                );
            }

            if params.cli.with_compaction_ids {
                let depth = params.cli.compaction_depth.unwrap_or(1);
                if let Some((ids, truncated)) = params.compaction_ctx.get_compacted_ids(
                    &params.note.frontmatter.id,
                    depth,
                    params.cli.compaction_max_nodes,
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

            if params.cli.expand_compaction {
                let depth = params.cli.compaction_depth.unwrap_or(1);
                if let Some((compacted_notes, truncated)) =
                    params.compaction_ctx.get_compacted_notes_expanded(
                        &params.note.frontmatter.id,
                        depth,
                        params.cli.compaction_max_nodes,
                        params.all_notes,
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
                                if params.include_custom && !n.frontmatter.custom.is_empty() {
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
