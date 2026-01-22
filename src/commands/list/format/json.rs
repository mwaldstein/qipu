//! JSON output formatting for list command

use crate::cli::Cli;
use crate::lib::compaction::CompactionContext;
use crate::lib::error::Result;
use crate::lib::store::Store;
use std::collections::HashMap;

/// Output in JSON format
pub fn output_json(
    cli: &Cli,
    _store: &Store,
    notes: &[crate::lib::note::Note],
    compaction_ctx: &CompactionContext,
    note_map: &HashMap<&str, &crate::lib::note::Note>,
) -> Result<()> {
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
            });

            add_compaction_annotations(cli, n, &mut json, compaction_ctx, note_map);

            json
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Add compaction annotations to JSON object
fn add_compaction_annotations(
    cli: &Cli,
    note: &crate::lib::note::Note,
    json: &mut serde_json::Value,
    compaction_ctx: &CompactionContext,
    note_map: &HashMap<&str, &crate::lib::note::Note>,
) {
    let compacts_count = compaction_ctx.get_compacts_count(&note.frontmatter.id);
    if compacts_count == 0 {
        return;
    }

    if let Some(obj) = json.as_object_mut() {
        obj.insert("compacts".to_string(), serde_json::json!(compacts_count));

        if let Some(pct) = compaction_ctx.get_compaction_pct(note, note_map) {
            obj.insert(
                "compaction_pct".to_string(),
                serde_json::json!(format!("{:.1}", pct)),
            );
        }

        add_compacted_ids(cli, note, obj, compaction_ctx);
    }
}

/// Add compacted IDs to JSON object
fn add_compacted_ids(
    cli: &Cli,
    note: &crate::lib::note::Note,
    obj: &mut serde_json::Map<String, serde_json::Value>,
    compaction_ctx: &CompactionContext,
) {
    if !cli.with_compaction_ids {
        return;
    }

    let depth = cli.compaction_depth.unwrap_or(1);
    if let Some((ids, truncated)) =
        compaction_ctx.get_compacted_ids(&note.frontmatter.id, depth, cli.compaction_max_nodes)
    {
        obj.insert("compacted_ids".to_string(), serde_json::json!(ids));

        if truncated {
            obj.insert(
                "compacted_ids_truncated".to_string(),
                serde_json::json!(true),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_json_empty() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let store =
            Store::init(temp_dir.path(), crate::lib::store::InitOptions::default()).unwrap();
        let cli = create_test_cli();

        let all_notes = store.list_notes().unwrap();
        let compaction_ctx = CompactionContext::build(&all_notes).unwrap();
        let note_map = CompactionContext::build_note_map(&all_notes);

        let result = output_json(&cli, &store, &[], &compaction_ctx, &note_map);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_json_single_note() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let store =
            Store::init(temp_dir.path(), crate::lib::store::InitOptions::default()).unwrap();
        let cli = create_test_cli();

        let note = store
            .create_note("Test Note", None, &["tag1".to_string()], None)
            .unwrap();

        let all_notes = store.list_notes().unwrap();
        let compaction_ctx = CompactionContext::build(&all_notes).unwrap();
        let note_map = CompactionContext::build_note_map(&all_notes);

        let result = output_json(&cli, &store, &[note], &compaction_ctx, &note_map);
        assert!(result.is_ok());
    }

    fn create_test_cli() -> Cli {
        Cli {
            root: None,
            store: None,
            format: crate::cli::OutputFormat::Json,
            quiet: false,
            verbose: false,
            log_level: None,
            log_json: false,
            no_resolve_compaction: false,
            with_compaction_ids: false,
            compaction_depth: None,
            compaction_max_nodes: None,
            expand_compaction: false,
            workspace: None,
            no_semantic_inversion: false,
            command: None,
        }
    }
}
