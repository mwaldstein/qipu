//! `qipu context` command - build context bundles for LLM integration
//!
//! Per spec (specs/llm-context.md):
//! - `qipu context` outputs a bundle of notes designed for LLM context injection
//! - Selection: `--note`, `--tag`, `--moc`, `--query`
//! - Budgeting: `--max-chars` exact budget
//! - Formats: human (markdown), json, records
//! - Safety: notes are untrusted inputs, optional safety banner

pub mod budget;
pub mod output;
pub mod select;
pub mod types;

use std::collections::{HashMap, HashSet};

use crate::cli::{Cli, OutputFormat};
use crate::lib::compaction::CompactionContext;
use crate::lib::error::{QipuError, Result};
use crate::lib::index::{search, IndexBuilder};
use crate::lib::note::Note;
use crate::lib::store::Store;

pub use types::ContextOptions;
use types::{RecordsOutputConfig, SelectedNote};

/// Execute the context command
pub fn execute(cli: &Cli, store: &Store, options: ContextOptions) -> Result<()> {
    // Build or load index for searching
    let index = IndexBuilder::new(store).load_existing()?.build()?;

    // Build compaction context for annotations
    let all_notes = store.list_notes()?;
    let compaction_ctx = CompactionContext::build(&all_notes)?;

    let note_map: HashMap<String, &Note> = all_notes
        .iter()
        .map(|note| (note.id().to_string(), note))
        .collect();

    // Collect notes based on selection criteria
    let mut selected_notes: Vec<SelectedNote> = Vec::new();
    let mut seen_ids: HashSet<String> = HashSet::new();
    let mut via_map: HashMap<String, String> = HashMap::new();

    let resolve_id = |id: &str| -> Result<String> {
        if cli.no_resolve_compaction {
            Ok(id.to_string())
        } else {
            compaction_ctx.canon(id)
        }
    };

    let mut insert_selected = |resolved_id: String, via_source: Option<String>| -> Result<()> {
        if let Some(via_id) = via_source {
            via_map.entry(resolved_id.clone()).or_insert(via_id);
        }

        if seen_ids.insert(resolved_id.clone()) {
            let note = note_map
                .get(&resolved_id)
                .ok_or_else(|| QipuError::NoteNotFound {
                    id: resolved_id.clone(),
                })?;
            selected_notes.push(SelectedNote {
                note: *note,
                via: None,
            });
        }

        Ok(())
    };

    // Selection by explicit note IDs
    for id in options.note_ids {
        let resolved_id = resolve_id(id)?;
        insert_selected(resolved_id, None)?;
    }

    // Selection by tag
    if let Some(tag_name) = options.tag {
        for note in &all_notes {
            if note.frontmatter.tags.contains(&tag_name.to_string()) {
                let resolved_id = resolve_id(note.id())?;
                insert_selected(resolved_id, None)?;
            }
        }
    }

    // Selection by MOC
    if let Some(moc) = options.moc_id {
        let linked_ids = select::get_moc_linked_ids(&index, moc, options.transitive);
        for id in linked_ids {
            let resolved_id = resolve_id(&id)?;
            insert_selected(resolved_id, None)?;
        }
    }

    // Selection by query
    if let Some(q) = options.query {
        let results = search(store, &index, q, None, None)?;
        for result in results {
            let resolved_id = resolve_id(&result.id)?;
            let via_source = if !cli.no_resolve_compaction && resolved_id != result.id {
                Some(result.id.clone())
            } else {
                None
            };
            insert_selected(resolved_id, via_source)?;
        }
    }

    for selected in &mut selected_notes {
        if let Some(via) = via_map.get(selected.note.id()) {
            selected.via = Some(via.clone());
        }
    }

    // If no selection criteria provided, return error
    if options.note_ids.is_empty()
        && options.tag.is_none()
        && options.moc_id.is_none()
        && options.query.is_none()
    {
        return Err(QipuError::Other(
            "no selection criteria provided. Use --note, --tag, --moc, or --query".to_string(),
        ));
    }

    // Sort notes deterministically (by created, then by id)
    selected_notes.sort_by(|a, b| {
        match (&a.note.frontmatter.created, &b.note.frontmatter.created) {
            (Some(a_created), Some(b_created)) => a_created.cmp(b_created),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
        .then_with(|| a.note.id().cmp(b.note.id()))
    });

    // Apply budgeting (records format handles its own exact budget)
    let (truncated, notes_to_output) = match cli.format {
        OutputFormat::Records => (false, selected_notes.iter().collect()),
        _ => budget::apply_budget(&selected_notes, options.max_chars, options.with_body),
    };

    // Output in requested format
    let store_path = store.root().display().to_string();

    match cli.format {
        OutputFormat::Json => {
            output::output_json(
                cli,
                &store_path,
                &notes_to_output,
                truncated,
                &compaction_ctx,
                &all_notes,
            )?;
        }
        OutputFormat::Human => {
            output::output_human(
                cli,
                &store_path,
                &notes_to_output,
                truncated,
                options.safety_banner,
                &compaction_ctx,
                &all_notes,
            );
        }
        OutputFormat::Records => {
            let config = RecordsOutputConfig {
                truncated,
                with_body: options.with_body,
                safety_banner: options.safety_banner,
                max_chars: options.max_chars,
            };
            output::output_records(
                cli,
                &store_path,
                &notes_to_output,
                &config,
                &compaction_ctx,
                &all_notes,
            );
        }
    }

    Ok(())
}
