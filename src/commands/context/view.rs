use std::collections::HashMap;

use qipu_core::compaction::CompactionContext;
use qipu_core::note::Note;
use qipu_core::store::Store;

use super::types::SelectedNote;

pub struct ContextBundleInput<'a> {
    pub cli: &'a crate::cli::Cli,
    pub store: &'a Store,
    pub store_path: &'a str,
    pub notes: &'a [&'a SelectedNote<'a>],
    pub compaction_ctx: &'a CompactionContext,
    pub note_map: &'a HashMap<&'a str, &'a Note>,
    pub all_notes: &'a [Note],
    pub include_custom: bool,
    pub include_ontology: bool,
    pub truncated: bool,
    pub with_body: bool,
    pub safety_banner: bool,
    pub max_chars: Option<usize>,
}

pub struct ContextBundleView<'a> {
    pub cli: &'a crate::cli::Cli,
    pub store: &'a Store,
    pub store_path: &'a str,
    pub notes: Vec<ContextNoteView<'a>>,
    pub include_custom: bool,
    pub include_ontology: bool,
    pub truncated: bool,
    pub with_body: bool,
    pub safety_banner: bool,
    pub max_chars: Option<usize>,
}

pub struct ContextNoteView<'a> {
    pub selected: &'a SelectedNote<'a>,
    pub note: &'a Note,
    pub content: String,
    pub path: Option<String>,
    pub tags_csv: String,
    pub via: Option<&'a str>,
    pub compacts_count: usize,
    pub compaction_pct: Option<f32>,
    pub compacted_ids: Option<CompactedIds>,
    pub compacted_notes: Vec<&'a Note>,
    pub compacted_notes_truncated: bool,
}

pub struct CompactedIds {
    pub ids: Vec<String>,
    pub truncated: bool,
}

impl<'a> ContextBundleView<'a> {
    pub fn build(input: ContextBundleInput<'a>) -> Self {
        let notes = input
            .notes
            .iter()
            .map(|selected| build_note_view(&input, selected))
            .collect();

        Self {
            cli: input.cli,
            store: input.store,
            store_path: input.store_path,
            notes,
            include_custom: input.include_custom,
            include_ontology: input.include_ontology,
            truncated: input.truncated,
            with_body: input.with_body,
            safety_banner: input.safety_banner,
            max_chars: input.max_chars,
        }
    }
}

fn build_note_view<'a>(
    input: &ContextBundleInput<'a>,
    selected: &'a SelectedNote<'a>,
) -> ContextNoteView<'a> {
    let note = selected.note;
    let compacts_count = input
        .compaction_ctx
        .get_compacts_count(&note.frontmatter.id);
    let compaction_pct = input
        .compaction_ctx
        .get_compaction_pct(note, input.note_map);

    let compacted_ids = if input.cli.with_compaction_ids && compacts_count > 0 {
        let depth = input.cli.compaction_depth.unwrap_or(1);
        input
            .compaction_ctx
            .get_compacted_ids(&note.frontmatter.id, depth, input.cli.compaction_max_nodes)
            .map(|(ids, truncated)| CompactedIds { ids, truncated })
    } else {
        None
    };

    let (compacted_notes, compacted_notes_truncated) =
        if input.cli.expand_compaction && compacts_count > 0 {
            let depth = input.cli.compaction_depth.unwrap_or(1);
            input
                .compaction_ctx
                .get_compacted_notes_expanded(
                    &note.frontmatter.id,
                    depth,
                    input.cli.compaction_max_nodes,
                    input.all_notes,
                )
                .unwrap_or_default()
        } else {
            (Vec::new(), false)
        };

    ContextNoteView {
        selected,
        note,
        content: if input.with_body {
            note.body.clone()
        } else {
            note.summary()
        },
        path: note
            .path
            .as_ref()
            .map(|path| super::path_relative_to_cwd(path)),
        tags_csv: note.frontmatter.format_tags(),
        via: selected.via.as_deref(),
        compacts_count,
        compaction_pct,
        compacted_ids,
        compacted_notes,
        compacted_notes_truncated,
    }
}
