//! Shared JSON building utilities for consistent JSON output formats

use crate::cli::Cli;
use qipu_core::compaction::CompactionContext;
use qipu_core::note::Note;
use qipu_core::ontology::Ontology;
use qipu_core::store::Store;

/// Build ontology JSON object
pub fn build_ontology_json(store: &Store) -> serde_json::Value {
    let config = store.config();
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

    serde_json::json!({
        "mode": config.ontology.mode,
        "note_types": note_type_objs,
        "link_types": link_type_objs,
    })
}

/// Build sources JSON array
pub fn build_sources_json(sources: &[qipu_core::note::Source]) -> Vec<serde_json::Value> {
    sources
        .iter()
        .map(|s| {
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
        })
        .collect()
}

/// Build source JSON object with optional note reference fields
pub fn build_source_json(
    source: &qipu_core::note::Source,
    note_id: Option<&str>,
    note_title: Option<&str>,
) -> serde_json::Value {
    let mut obj = serde_json::json!({
        "url": source.url,
    });
    if let Some(title) = &source.title {
        obj["title"] = serde_json::json!(title);
    }
    if let Some(accessed) = &source.accessed {
        obj["accessed"] = serde_json::json!(accessed);
    }
    if let Some(id) = note_id {
        obj["from_note_id"] = serde_json::json!(id);
    }
    if let Some(title) = note_title {
        obj["from_note_title"] = serde_json::json!(title);
    }
    obj
}

/// Add compaction metadata to JSON object
pub fn add_compaction_metadata_to_json(
    obj: &mut serde_json::Value,
    note: &Note,
    cli: &Cli,
    compaction_ctx: &CompactionContext,
    note_map: &std::collections::HashMap<&str, &Note>,
) {
    let compacts_count = compaction_ctx.get_compacts_count(&note.frontmatter.id);
    if compacts_count == 0 {
        return;
    }

    if let Some(obj_mut) = obj.as_object_mut() {
        obj_mut.insert("compacts".to_string(), serde_json::json!(compacts_count));

        if let Some(pct) = compaction_ctx.get_compaction_pct(note, note_map) {
            obj_mut.insert(
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
