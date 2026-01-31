use super::{ExportContext, ExportMode};
use crate::cli::Cli;
use crate::commands::export::ExportOptions;
use qipu_core::compaction::CompactionContext;
use qipu_core::error::Result;
use qipu_core::note::Note;
use qipu_core::store::Store;

pub fn export_json(ctx: &ExportContext) -> Result<String> {
    let mode_str = match ctx.options.mode {
        ExportMode::Bundle => "bundle",
        ExportMode::Outline => "outline",
        ExportMode::Bibliography => "bibliography",
    };

    let output = match ctx.options.mode {
        ExportMode::Bibliography => export_bibliography_json(ctx.notes, ctx.store, mode_str),
        _ => export_notes_json(
            ctx.notes,
            ctx.store,
            ctx.options,
            ctx.cli,
            ctx.compaction_ctx,
            ctx.all_notes,
            mode_str,
        ),
    };

    Ok(serde_json::to_string_pretty(&output)?)
}

fn export_bibliography_json(notes: &[Note], store: &Store, mode_str: &str) -> serde_json::Value {
    let mut all_sources: Vec<_> = notes
        .iter()
        .flat_map(|note| {
            note.frontmatter.sources.iter().map(move |source| {
                crate::commands::json_builders::build_source_json(
                    source,
                    Some(note.id()),
                    Some(note.title()),
                )
            })
        })
        .collect();

    for note in notes {
        if let Some(source_url) = &note.frontmatter.source {
            let source = qipu_core::note::Source {
                url: source_url.clone(),
                title: None,
                accessed: None,
            };
            all_sources.push(crate::commands::json_builders::build_source_json(
                &source,
                Some(note.id()),
                Some(note.title()),
            ));
        }
    }

    all_sources.sort_by(|a, b| {
        let url_a = a["url"].as_str().unwrap_or("");
        let url_b = b["url"].as_str().unwrap_or("");
        url_a.cmp(url_b)
    });

    serde_json::json!({
        "store": store.root().display().to_string(),
        "mode": mode_str,
        "sources": all_sources,
    })
}

fn export_notes_json(
    notes: &[Note],
    store: &Store,
    _options: &ExportOptions,
    cli: &Cli,
    compaction_ctx: &CompactionContext,
    all_notes: &[Note],
    mode_str: &str,
) -> serde_json::Value {
    let note_map = CompactionContext::build_note_map(all_notes);

    serde_json::json!({
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

                    "created": note.frontmatter.created,
                    "updated": note.frontmatter.updated,
                    "content": note.body,
                    "source": note.frontmatter.source,
                    "sources": crate::commands::json_builders::build_sources_json(&note.frontmatter.sources),
                });

                crate::commands::json_builders::add_compaction_metadata_to_json(
                    &mut obj,
                    note,
                    cli,
                    compaction_ctx,
                    &note_map,
                );

                obj
            })
            .collect::<Vec<_>>(),
    })
}
