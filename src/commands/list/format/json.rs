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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_json_empty() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let store = Store::init(temp_dir.path(), qipu_core::store::InitOptions::default()).unwrap();
        let cli = create_test_cli();

        let all_notes = store.list_notes().unwrap();
        let compaction_ctx = CompactionContext::build(&all_notes).unwrap();
        let note_map = CompactionContext::build_note_map(&all_notes);

        let result = output_json(&cli, &store, &[], &compaction_ctx, &note_map, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_json_single_note() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let store = Store::init(temp_dir.path(), qipu_core::store::InitOptions::default()).unwrap();
        let cli = create_test_cli();

        let note = store
            .create_note("Test Note", None, &["tag1".to_string()], None)
            .unwrap();

        let all_notes = store.list_notes().unwrap();
        let compaction_ctx = CompactionContext::build(&all_notes).unwrap();
        let note_map = CompactionContext::build_note_map(&all_notes);

        let result = output_json(&cli, &store, &[note], &compaction_ctx, &note_map, false);
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
