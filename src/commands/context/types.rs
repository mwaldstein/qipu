use qipu_core::note::{LinkType, Note};

/// Options for the context command
pub struct ContextOptions<'a> {
    pub walk_id: Option<&'a str>,
    pub walk_direction: &'a str,
    pub walk_max_hops: u32,
    pub walk_type: &'a [String],
    pub walk_exclude_type: &'a [String],
    pub walk_typed_only: bool,
    pub walk_inline_only: bool,
    pub walk_max_nodes: Option<usize>,
    pub walk_max_edges: Option<usize>,
    pub walk_max_fanout: Option<usize>,
    pub walk_min_value: Option<u8>,
    pub walk_ignore_value: bool,
    pub note_ids: &'a [String],
    pub tag: Option<&'a str>,
    pub moc_id: Option<&'a str>,
    pub query: Option<&'a str>,
    pub max_chars: Option<usize>,
    pub transitive: bool,
    pub with_body: bool,
    pub safety_banner: bool,
    pub related_threshold: Option<f64>,
    pub backlinks: bool,
    pub min_value: Option<u8>,
    pub custom_filter: &'a [String],
    pub include_custom: bool,
    pub include_ontology: bool,
}

pub struct SelectedNote<'a> {
    pub note: &'a Note,
    pub via: Option<String>,
    pub link_type: Option<LinkType>,
}

pub struct RecordsOutputConfig {
    pub truncated: bool,
    pub with_body: bool,
    pub safety_banner: bool,
    pub max_chars: Option<usize>,
}

pub struct ContextOutputParams<'a> {
    pub cli: &'a crate::cli::Cli,
    pub store: &'a qipu_core::store::Store,
    pub store_path: &'a str,
    pub notes: &'a [&'a SelectedNote<'a>],
    pub compaction_ctx: &'a qipu_core::compaction::CompactionContext,
    pub note_map: &'a std::collections::HashMap<&'a str, &'a qipu_core::note::Note>,
    pub all_notes: &'a [qipu_core::note::Note],
    pub include_custom: bool,
    pub include_ontology: bool,
    pub truncated: bool,
    pub with_body: bool,
    pub max_chars: Option<usize>,
}

pub struct HumanOutputParams<'a> {
    pub cli: &'a crate::cli::Cli,
    pub store: &'a qipu_core::store::Store,
    pub store_path: &'a str,
    pub notes: &'a [&'a SelectedNote<'a>],
    pub compaction_ctx: &'a qipu_core::compaction::CompactionContext,
    pub note_map: &'a std::collections::HashMap<&'a str, &'a qipu_core::note::Note>,
    pub all_notes: &'a [qipu_core::note::Note],
    pub include_custom: bool,
    pub include_ontology: bool,
    pub truncated: bool,
    pub with_body: bool,
    pub safety_banner: bool,
    pub max_chars: Option<usize>,
}

pub struct RecordsParams<'a> {
    pub cli: &'a crate::cli::Cli,
    pub store: &'a qipu_core::store::Store,
    pub store_path: &'a str,
    pub notes: &'a [&'a SelectedNote<'a>],
    pub config: &'a RecordsOutputConfig,
    pub compaction_ctx: &'a qipu_core::compaction::CompactionContext,
    pub note_map: &'a std::collections::HashMap<&'a str, &'a qipu_core::note::Note>,
    pub all_notes: &'a [qipu_core::note::Note],
    pub include_custom: bool,
    pub include_ontology: bool,
}

pub struct BuildNoteJsonParams<'a> {
    pub cli: &'a crate::cli::Cli,
    pub note: &'a qipu_core::note::Note,
    pub selected: &'a SelectedNote<'a>,
    pub compaction_ctx: &'a qipu_core::compaction::CompactionContext,
    pub note_map: &'a std::collections::HashMap<&'a str, &'a qipu_core::note::Note>,
    pub all_notes: &'a [qipu_core::note::Note],
    pub include_custom: bool,
    pub content: &'a str,
}
