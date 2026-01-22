//! `qipu list` command - list notes
//!
//! Per spec (specs/cli-interface.md):
//! - `--tag` filter
//! - `--type` filter
//! - `--since` filter
//! - Deterministic ordering (by created, then id)
//! - Compaction visibility (specs/compaction.md): hide compacted notes by default

use chrono::{DateTime, Utc};

use crate::cli::{Cli, OutputFormat};
use crate::lib::compaction::CompactionContext;
use crate::lib::error::Result;
use crate::lib::note::NoteType;
use crate::lib::records::escape_quotes;
use crate::lib::store::Store;

/// Execute the list command
pub fn execute(
    cli: &Cli,
    store: &Store,
    tag: Option<&str>,
    note_type: Option<NoteType>,
    since: Option<DateTime<Utc>>,
    min_value: Option<u8>,
    custom: Option<&str>,
) -> Result<()> {
    let all_notes = store.list_notes()?;
    let mut notes = all_notes.clone();

    // Build compaction context for both filtering and annotations
    // Per spec (specs/compaction.md line 101): hide notes with a compactor by default
    let compaction_ctx = CompactionContext::build(&all_notes)?;

    // Build note map for efficient lookups (avoid O(n²) when calculating compaction pct)
    let note_map = CompactionContext::build_note_map(&all_notes);

    if !cli.no_resolve_compaction {
        notes.retain(|n| !compaction_ctx.is_compacted(&n.frontmatter.id));
    }

    // Apply filters
    if let Some(tag) = tag {
        notes.retain(|n| n.frontmatter.tags.iter().any(|t| t == tag));
    }

    if let Some(nt) = note_type {
        notes.retain(|n| n.note_type() == nt);
    }

    if let Some(since) = since {
        notes.retain(|n| {
            n.frontmatter
                .created
                .is_some_and(|created| created >= since)
        });
    }

    if let Some(min_val) = min_value {
        notes.retain(|n| {
            let value = n.frontmatter.value.unwrap_or(50);
            value >= min_val
        });
    }

    // Apply custom metadata filter
    if let Some(custom_filter) = custom {
        if let Some((key, value)) = custom_filter.split_once('=') {
            notes.retain(|n| {
                n.frontmatter
                    .custom
                    .get(key)
                    .map(|v| {
                        // Compare as strings for simplicity
                        match v {
                            serde_yaml::Value::String(s) => s == value,
                            serde_yaml::Value::Number(num) => num.to_string() == value,
                            serde_yaml::Value::Bool(b) => b.to_string() == value,
                            _ => false,
                        }
                    })
                    .unwrap_or(false)
            });
        }
    }

    match cli.format {
        OutputFormat::Json => {
            let output: Vec<_> = notes
                .iter()
                .map(|n| {
                    let mut json = serde_json::json!({
                        "id": n.id(),
                        "title": n.title(),
                        "type": n.note_type().to_string(),
                        "tags": n.frontmatter.tags,
                        "path": n.path.as_ref().map(|p| p.display().to_string()),
                        "created": n.frontmatter.created,
                        "updated": n.frontmatter.updated,
                    });

                    // Add compaction annotations for digest notes
                    // Per spec (specs/compaction.md lines 116-119)
                    let compacts_count = compaction_ctx.get_compacts_count(&n.frontmatter.id);
                    if compacts_count > 0 {
                        if let Some(obj) = json.as_object_mut() {
                            obj.insert("compacts".to_string(), serde_json::json!(compacts_count));

                            if let Some(pct) = compaction_ctx.get_compaction_pct(n, &note_map) {
                                obj.insert(
                                    "compaction_pct".to_string(),
                                    serde_json::json!(format!("{:.1}", pct)),
                                );
                            }

                            // Add compacted IDs if --with-compaction-ids is set
                            // Per spec (specs/compaction.md line 131)
                            if cli.with_compaction_ids {
                                let depth = cli.compaction_depth.unwrap_or(1);
                                if let Some((ids, truncated)) = compaction_ctx.get_compacted_ids(
                                    &n.frontmatter.id,
                                    depth,
                                    cli.compaction_max_nodes,
                                ) {
                                    obj.insert("compacted_ids".to_string(), serde_json::json!(ids));
                                    // Per spec line 142: outputs must indicate truncation
                                    if truncated {
                                        obj.insert(
                                            "compacted_ids_truncated".to_string(),
                                            serde_json::json!(true),
                                        );
                                    }
                                }
                            }
                        }
                    }

                    json
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Human => {
            if notes.is_empty() {
                if !cli.quiet {
                    println!("No notes found");
                }
            } else {
                for note in &notes {
                    let type_indicator = match note.note_type() {
                        NoteType::Fleeting => "F",
                        NoteType::Literature => "L",
                        NoteType::Permanent => "P",
                        NoteType::Moc => "M",
                    };

                    // Build compaction annotations for digest notes
                    // Per spec (specs/compaction.md lines 116-119)
                    let mut annotations = String::new();
                    let compacts_count = compaction_ctx.get_compacts_count(&note.frontmatter.id);
                    if compacts_count > 0 {
                        annotations.push_str(&format!(" compacts={}", compacts_count));

                        if let Some(pct) = compaction_ctx.get_compaction_pct(note, &note_map) {
                            annotations.push_str(&format!(" compaction={:.0}%", pct));
                        }
                    }

                    println!(
                        "{} [{}] {}{}",
                        note.id(),
                        type_indicator,
                        note.title(),
                        annotations
                    );

                    // Show compacted IDs if --with-compaction-ids is set
                    // Per spec (specs/compaction.md line 131)
                    if cli.with_compaction_ids && compacts_count > 0 {
                        let depth = cli.compaction_depth.unwrap_or(1);
                        if let Some((ids, truncated)) = compaction_ctx.get_compacted_ids(
                            &note.frontmatter.id,
                            depth,
                            cli.compaction_max_nodes,
                        ) {
                            let ids_str = ids.join(", ");
                            let suffix = if truncated {
                                let max = cli.compaction_max_nodes.unwrap_or(ids.len());
                                format!(" (truncated, showing {} of {})", max, compacts_count)
                            } else {
                                String::new()
                            };
                            println!("  Compacted: {}{}", ids_str, suffix);
                        }
                    }
                }
            }
        }
        OutputFormat::Records => {
            // Header line per spec (specs/records-output.md)
            println!(
                "H qipu=1 records=1 store={} mode=list notes={}",
                store.root().display(),
                notes.len()
            );
            for note in &notes {
                let tags_csv = if note.frontmatter.tags.is_empty() {
                    "-".to_string()
                } else {
                    note.frontmatter.tags.join(",")
                };

                // Build compaction annotations for digest notes
                // Per spec (specs/compaction.md lines 116-119, 125)
                let mut annotations = String::new();
                let compacts_count = compaction_ctx.get_compacts_count(&note.frontmatter.id);
                if compacts_count > 0 {
                    annotations.push_str(&format!(" compacts={}", compacts_count));

                    if let Some(pct) = compaction_ctx.get_compaction_pct(note, &note_map) {
                        annotations.push_str(&format!(" compaction={:.0}%", pct));
                    }
                }

                println!(
                    "N {} {} \"{}\" tags={}{}",
                    note.id(),
                    note.note_type(),
                    escape_quotes(note.title()),
                    tags_csv,
                    annotations
                );

                // Show compacted IDs if --with-compaction-ids is set
                // Per spec (specs/compaction.md line 131)
                if cli.with_compaction_ids && compacts_count > 0 {
                    let depth = cli.compaction_depth.unwrap_or(1);
                    if let Some((ids, truncated)) = compaction_ctx.get_compacted_ids(
                        &note.frontmatter.id,
                        depth,
                        cli.compaction_max_nodes,
                    ) {
                        for id in &ids {
                            println!("D compacted {} from={}", id, note.id());
                        }
                        if truncated {
                            println!(
                                "D compacted_truncated max={} total={}",
                                cli.compaction_max_nodes.unwrap_or(ids.len()),
                                compacts_count
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::OutputFormat;
    use crate::lib::note::NoteType;
    use crate::lib::store::{InitOptions, Store};
    use chrono::{Duration, Utc};
    use tempfile::TempDir;

    fn create_cli(format: OutputFormat, quiet: bool) -> Cli {
        Cli {
            root: None,
            store: None,
            format,
            quiet,
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

    fn create_test_store() -> (TempDir, Store) {
        let temp_dir = TempDir::new().unwrap();
        let store = Store::init(temp_dir.path(), InitOptions::default()).unwrap();
        (temp_dir, store)
    }

    #[test]
    fn test_list_empty_store_human() {
        let (_temp_dir, store) = create_test_store();
        let cli = create_cli(OutputFormat::Human, false);

        let result = execute(&cli, &store, None, None, None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_empty_store_quiet() {
        let (_temp_dir, store) = create_test_store();
        let cli = create_cli(OutputFormat::Human, true);

        let result = execute(&cli, &store, None, None, None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_empty_store_json() {
        let (_temp_dir, store) = create_test_store();
        let cli = create_cli(OutputFormat::Json, false);

        let result = execute(&cli, &store, None, None, None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_empty_store_records() {
        let (_temp_dir, store) = create_test_store();
        let cli = create_cli(OutputFormat::Records, false);

        let result = execute(&cli, &store, None, None, None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_single_note_human() {
        let (_temp_dir, store) = create_test_store();
        store
            .create_note("Test Note", None, &["tag1".to_string()], None)
            .unwrap();

        let cli = create_cli(OutputFormat::Human, false);
        let result = execute(&cli, &store, None, None, None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_single_note_json() {
        let (_temp_dir, store) = create_test_store();
        store
            .create_note("Test Note", None, &["tag1".to_string()], None)
            .unwrap();

        let cli = create_cli(OutputFormat::Json, false);
        let result = execute(&cli, &store, None, None, None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_single_note_records() {
        let (_temp_dir, store) = create_test_store();
        store
            .create_note("Test Note", None, &["tag1".to_string()], None)
            .unwrap();

        let cli = create_cli(OutputFormat::Records, false);
        let result = execute(&cli, &store, None, None, None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_multiple_notes() {
        let (_temp_dir, store) = create_test_store();
        store
            .create_note("Note 1", None, &["tag1".to_string()], None)
            .unwrap();
        store
            .create_note("Note 2", None, &["tag2".to_string()], None)
            .unwrap();
        store
            .create_note("Note 3", None, &["tag3".to_string()], None)
            .unwrap();

        let cli = create_cli(OutputFormat::Human, false);
        let result = execute(&cli, &store, None, None, None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_filter_by_tag() {
        let (_temp_dir, store) = create_test_store();
        store
            .create_note("Note 1", None, &["matching".to_string()], None)
            .unwrap();
        store
            .create_note("Note 2", None, &["other".to_string()], None)
            .unwrap();
        store
            .create_note("Note 3", None, &["matching".to_string()], None)
            .unwrap();

        let cli = create_cli(OutputFormat::Human, false);
        let result = execute(&cli, &store, Some("matching"), None, None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_filter_by_tag_none_matching() {
        let (_temp_dir, store) = create_test_store();
        store
            .create_note("Note 1", None, &["tag1".to_string()], None)
            .unwrap();

        let cli = create_cli(OutputFormat::Human, false);
        let result = execute(&cli, &store, Some("nonexistent"), None, None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_filter_by_type() {
        let (_temp_dir, store) = create_test_store();
        let mut note1 = store.create_note("Fleeting Note", None, &[], None).unwrap();
        note1.frontmatter.note_type = Some(NoteType::Fleeting);
        store.save_note(&mut note1).unwrap();

        let mut note2 = store
            .create_note("Permanent Note", None, &[], None)
            .unwrap();
        note2.frontmatter.note_type = Some(NoteType::Permanent);
        store.save_note(&mut note2).unwrap();

        let cli = create_cli(OutputFormat::Human, false);
        let result = execute(
            &cli,
            &store,
            None,
            Some(NoteType::Permanent),
            None,
            None,
            None,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_filter_by_since() {
        let (_temp_dir, store) = create_test_store();

        let mut note1 = store.create_note("Old Note", None, &[], None).unwrap();
        note1.frontmatter.created = Some(Utc::now() - Duration::days(10));
        store.save_note(&mut note1).unwrap();

        let mut note2 = store.create_note("Recent Note", None, &[], None).unwrap();
        note2.frontmatter.created = Some(Utc::now() - Duration::days(1));
        store.save_note(&mut note2).unwrap();

        let cli = create_cli(OutputFormat::Human, false);
        let since = Utc::now() - Duration::days(5);
        let result = execute(&cli, &store, None, None, Some(since), None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_with_compaction_resolved() {
        let (_temp_dir, store) = create_test_store();

        let note1 = store
            .create_note("Original Note", None, &["original".to_string()], None)
            .unwrap();

        let note1_id = note1.id().to_string();

        let mut digest = store.create_note("Digest Note", None, &[], None).unwrap();
        digest.frontmatter.compacts = vec![note1_id.clone()];
        store.save_note(&mut digest).unwrap();

        let cli = create_cli(OutputFormat::Human, false);
        let result = execute(&cli, &store, None, None, None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_with_compaction_disabled() {
        let (_temp_dir, store) = create_test_store();

        let note1 = store
            .create_note("Original Note", None, &["original".to_string()], None)
            .unwrap();

        let note1_id = note1.id().to_string();

        let mut digest = store.create_note("Digest Note", None, &[], None).unwrap();
        digest.frontmatter.compacts = vec![note1_id.clone()];
        store.save_note(&mut digest).unwrap();

        let mut cli = create_cli(OutputFormat::Human, false);
        cli.no_resolve_compaction = true;

        let result = execute(&cli, &store, None, None, None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_with_compaction_ids() {
        let (_temp_dir, store) = create_test_store();

        let note1 = store
            .create_note("Original Note", None, &["original".to_string()], None)
            .unwrap();
        let note1_id = note1.id().to_string();

        let note2 = store
            .create_note("Another Original", None, &["original".to_string()], None)
            .unwrap();
        let note2_id = note2.id().to_string();

        let mut digest = store.create_note("Digest Note", None, &[], None).unwrap();
        digest.frontmatter.compacts = vec![note1_id, note2_id];
        store.save_note(&mut digest).unwrap();

        let mut cli = create_cli(OutputFormat::Human, false);
        cli.with_compaction_ids = true;

        let result = execute(&cli, &store, None, None, None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_with_compaction_ids_depth() {
        let (_temp_dir, store) = create_test_store();

        let note1 = store
            .create_note("Original Note", None, &["original".to_string()], None)
            .unwrap();
        let note1_id = note1.id().to_string();

        let mut digest = store.create_note("Digest Note", None, &[], None).unwrap();
        digest.frontmatter.compacts = vec![note1_id.clone()];
        store.save_note(&mut digest).unwrap();

        let mut cli = create_cli(OutputFormat::Human, false);
        cli.with_compaction_ids = true;
        cli.compaction_depth = Some(2);

        let result = execute(&cli, &store, None, None, None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_compaction_annotations_human() {
        let (_temp_dir, store) = create_test_store();

        let note1 = store
            .create_note_with_content(
                "Original Note",
                None,
                &["original".to_string()],
                "# Summary\nContent from original note 1.",
                None,
            )
            .unwrap();
        let note1_id = note1.id().to_string();

        let note2 = store
            .create_note_with_content(
                "Another Original",
                None,
                &["original".to_string()],
                "# Summary\nContent from original note 2.",
                None,
            )
            .unwrap();
        let note2_id = note2.id().to_string();

        let mut digest = store.create_note("Digest Note", None, &[], None).unwrap();
        digest.frontmatter.compacts = vec![note1_id, note2_id];
        store.save_note(&mut digest).unwrap();

        let cli = create_cli(OutputFormat::Human, false);
        let result = execute(&cli, &store, None, None, None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_compaction_annotations_json() {
        let (_temp_dir, store) = create_test_store();

        let note1 = store
            .create_note_with_content(
                "Original Note",
                None,
                &["original".to_string()],
                "# Summary\nContent from original note.",
                None,
            )
            .unwrap();
        let note1_id = note1.id().to_string();

        let mut digest = store.create_note("Digest Note", None, &[], None).unwrap();
        digest.frontmatter.compacts = vec![note1_id];
        store.save_note(&mut digest).unwrap();

        let cli = create_cli(OutputFormat::Json, false);
        let result = execute(&cli, &store, None, None, None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_compaction_annotations_records() {
        let (_temp_dir, store) = create_test_store();

        let note1 = store
            .create_note_with_content(
                "Original Note",
                None,
                &["original".to_string()],
                "# Summary\nContent from original note.",
                None,
            )
            .unwrap();
        let note1_id = note1.id().to_string();

        let mut digest = store.create_note("Digest Note", None, &[], None).unwrap();
        digest.frontmatter.compacts = vec![note1_id];
        store.save_note(&mut digest).unwrap();

        let cli = create_cli(OutputFormat::Records, false);
        let result = execute(&cli, &store, None, None, None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_all_formats_compaction_with_ids() {
        let (_temp_dir, store) = create_test_store();

        let note1 = store
            .create_note("Original Note", None, &["original".to_string()], None)
            .unwrap();
        let note1_id = note1.id().to_string();

        let mut digest = store.create_note("Digest Note", None, &[], None).unwrap();
        digest.frontmatter.compacts = vec![note1_id];
        store.save_note(&mut digest).unwrap();

        for format in [
            OutputFormat::Human,
            OutputFormat::Json,
            OutputFormat::Records,
        ] {
            let mut cli = create_cli(format, false);
            cli.with_compaction_ids = true;
            let result = execute(&cli, &store, None, None, None, None, None);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_list_filter_by_min_value_all_match() {
        let (_temp_dir, store) = create_test_store();

        let mut note1 = store
            .create_note("High Value Note", None, &["high".to_string()], None)
            .unwrap();
        note1.frontmatter.value = Some(90);
        store.save_note(&mut note1).unwrap();

        let mut note2 = store
            .create_note("Medium Value Note", None, &["medium".to_string()], None)
            .unwrap();
        note2.frontmatter.value = Some(70);
        store.save_note(&mut note2).unwrap();

        let cli = create_cli(OutputFormat::Human, false);
        let result = execute(&cli, &store, None, None, None, Some(50), None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_filter_by_min_value_some_match() {
        let (_temp_dir, store) = create_test_store();

        let mut note1 = store
            .create_note("High Value Note", None, &["high".to_string()], None)
            .unwrap();
        note1.frontmatter.value = Some(90);
        store.save_note(&mut note1).unwrap();

        let mut note2 = store
            .create_note("Low Value Note", None, &["low".to_string()], None)
            .unwrap();
        note2.frontmatter.value = Some(30);
        store.save_note(&mut note2).unwrap();

        let cli = create_cli(OutputFormat::Human, false);
        let result = execute(&cli, &store, None, None, None, Some(50), None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_filter_by_min_value_none_match() {
        let (_temp_dir, store) = create_test_store();

        let mut note1 = store
            .create_note("Low Value Note 1", None, &["low".to_string()], None)
            .unwrap();
        note1.frontmatter.value = Some(20);
        store.save_note(&mut note1).unwrap();

        let mut note2 = store
            .create_note("Low Value Note 2", None, &["low".to_string()], None)
            .unwrap();
        note2.frontmatter.value = Some(10);
        store.save_note(&mut note2).unwrap();

        let cli = create_cli(OutputFormat::Human, false);
        let result = execute(&cli, &store, None, None, None, Some(50), None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_filter_by_min_value_with_defaults() {
        let (_temp_dir, store) = create_test_store();

        let _note1 = store
            .create_note("Default Value Note", None, &["default".to_string()], None)
            .unwrap();

        let mut note2 = store
            .create_note("Low Value Note", None, &["low".to_string()], None)
            .unwrap();
        note2.frontmatter.value = Some(20);
        store.save_note(&mut note2).unwrap();

        let cli = create_cli(OutputFormat::Human, false);
        let result = execute(&cli, &store, None, None, None, Some(40), None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_filter_by_min_value_exact() {
        let (_temp_dir, store) = create_test_store();

        let mut note1 = store
            .create_note("Value 75 Note", None, &["exact".to_string()], None)
            .unwrap();
        note1.frontmatter.value = Some(75);
        store.save_note(&mut note1).unwrap();

        let mut note2 = store
            .create_note("Value 50 Note", None, &["exact".to_string()], None)
            .unwrap();
        note2.frontmatter.value = Some(50);
        store.save_note(&mut note2).unwrap();

        let cli = create_cli(OutputFormat::Human, false);
        let result = execute(&cli, &store, None, None, None, Some(50), None);
        assert!(result.is_ok());
    }
}
