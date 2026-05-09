use super::view::{ContextBundleView, ContextNoteView};
use qipu_core::error::Result;
use qipu_core::note::Note;
use std::time::Instant;
use tracing::debug;

/// Output in JSON format
pub fn output_json(view: &ContextBundleView) -> Result<()> {
    let start = Instant::now();

    if view.cli.verbose {
        debug!(
            notes_count = view.notes.len(),
            truncated = view.truncated,
            with_body = view.with_body,
            max_chars = view.max_chars,
            include_custom = view.include_custom,
            include_ontology = view.include_ontology,
            "output_json"
        );
    }

    let output = build_json_output(view);

    println!("{}", serde_json::to_string_pretty(&output)?);

    if view.cli.verbose {
        debug!(elapsed = ?start.elapsed(), "output_json_complete");
    }
    Ok(())
}

fn truncate_note_content(content: &str, note_size: usize, remaining: usize) -> (String, bool) {
    let marker = "…[truncated]";
    let marker_len = marker.len();
    let available_for_content = remaining.saturating_sub(note_size - content.len() + marker_len);

    if content.len() > marker.len() + 10 && available_for_content > marker.len() {
        let truncated_content_len = available_for_content.min(content.len() - marker.len());
        (
            format!("{} {}", &content[..truncated_content_len], marker),
            true,
        )
    } else {
        (marker.to_string(), true)
    }
}

fn build_json_output(view: &ContextBundleView) -> serde_json::Value {
    let mut json_notes: Vec<serde_json::Value> = Vec::new();
    let mut actual_truncated = false;
    let mut estimated_size = view.store_path.len() + 50;

    for note_view in &view.notes {
        let note_json_obj = build_note_json(note_view, &note_view.content, view.include_custom);

        let note_json_str = serde_json::to_string(&note_json_obj).unwrap_or_default();
        let note_size = note_json_str.len() + 10;

        if let Some(budget) = view.max_chars {
            let remaining = budget.saturating_sub(estimated_size);

            if view.truncated || note_size > remaining {
                actual_truncated = true;
                let (final_content, content_truncated) =
                    truncate_note_content(&note_view.content, note_size, remaining);

                let mut truncated_note_json =
                    build_note_json(note_view, &final_content, view.include_custom);

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
        "store": view.store_path,
        "truncated": view.truncated || actual_truncated,
        "notes": json_notes,
    });

    if view.include_ontology {
        if let Some(obj) = output.as_object_mut() {
            obj.insert(
                "ontology".to_string(),
                crate::commands::json_builders::build_ontology_json(view.store),
            );
        }
    }

    output
}

fn build_custom_json(
    custom: &std::collections::HashMap<String, serde_yaml::Value>,
) -> serde_json::Value {
    let custom_json: serde_json::Map<String, serde_json::Value> = custom
        .iter()
        .map(|(k, v)| {
            let json_value = serde_json::to_value(v).unwrap_or(serde_json::Value::Null);
            (k.clone(), json_value)
        })
        .collect();
    serde_json::Value::Object(custom_json)
}

fn build_compacted_note_json(note: &Note, include_custom: bool) -> serde_json::Value {
    let mut note_json = serde_json::json!({
        "id": note.id(),
        "title": note.title(),
        "type": note.note_type().to_string(),
        "tags": note.frontmatter.tags,
        "content": note.body,
        "sources": crate::commands::json_builders::build_sources_json(&note.frontmatter.sources),
    });

    if include_custom && !note.frontmatter.custom.is_empty() {
        if let Some(obj) = note_json.as_object_mut() {
            obj.insert(
                "custom".to_string(),
                build_custom_json(&note.frontmatter.custom),
            );
        }
    }

    note_json
}

fn build_compaction_info_json(
    note_view: &ContextNoteView,
    include_custom: bool,
) -> Option<serde_json::Value> {
    let compacts_count = note_view.compacts_count;
    if compacts_count == 0 {
        return None;
    }

    let mut obj = serde_json::json!({
        "compacts": compacts_count,
    });

    if let Some(pct) = note_view.compaction_pct {
        obj["compaction_pct"] = serde_json::json!(format!("{:.1}", pct));
    }

    if let Some(compacted_ids) = &note_view.compacted_ids {
        obj["compacted_ids"] = serde_json::json!(&compacted_ids.ids);
        if compacted_ids.truncated {
            obj["compacted_ids_truncated"] = serde_json::json!(true);
        }
    }

    if !note_view.compacted_notes.is_empty() {
        obj["compacted_notes"] = serde_json::json!(note_view
            .compacted_notes
            .iter()
            .map(|n| build_compacted_note_json(n, include_custom))
            .collect::<Vec<_>>());
        if note_view.compacted_notes_truncated {
            obj["compacted_notes_truncated"] = serde_json::json!(true);
        }
    }

    Some(obj)
}

fn build_note_json(
    note_view: &ContextNoteView,
    content: &str,
    include_custom: bool,
) -> serde_json::Value {
    let note = note_view.note;
    let mut json = serde_json::json!({
        "id": note.id(),
        "title": note.title(),
        "type": note.note_type().to_string(),
        "tags": note.frontmatter.tags,
        "content": content,
        "content_truncated": false,
        "sources": crate::commands::json_builders::build_sources_json(&note.frontmatter.sources),
        "source": note.frontmatter.source,
        "author": note.frontmatter.author,
        "generated_by": note.frontmatter.generated_by,
        "prompt_hash": note.frontmatter.prompt_hash,
        "verified": note.frontmatter.verified,
    });

    if include_custom && !note.frontmatter.custom.is_empty() {
        if let Some(obj) = json.as_object_mut() {
            obj.insert(
                "custom".to_string(),
                build_custom_json(&note.frontmatter.custom),
            );
        }
    }

    if let Some(via) = note_view.via {
        if let Some(obj) = json.as_object_mut() {
            obj.insert("via".to_string(), serde_json::json!(via));
        }
    }

    if let Some(compaction_info) = build_compaction_info_json(note_view, include_custom) {
        if let Some(obj) = json.as_object_mut() {
            for (key, value) in compaction_info.as_object().unwrap() {
                obj.insert(key.clone(), value.clone());
            }
        }
    }

    json
}
