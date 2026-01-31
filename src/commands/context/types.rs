use qipu_core::note::{LinkType, Note};

pub struct ContextOptions<'a> {
    pub walk: WalkOptions<'a>,
    pub selection: SelectionOptions<'a>,
    pub expansion: ExpansionOptions,
    pub output: OutputOptions,
}

pub struct WalkOptions<'a> {
    pub id: Option<&'a str>,
    pub direction: &'a str,
    pub max_hops: u32,
    pub type_include: &'a [String],
    pub type_exclude: &'a [String],
    pub typed_only: bool,
    pub inline_only: bool,
    pub max_nodes: Option<usize>,
    pub max_edges: Option<usize>,
    pub max_fanout: Option<usize>,
    pub min_value: Option<u8>,
    pub ignore_value: bool,
}

pub struct SelectionOptions<'a> {
    pub note_ids: &'a [String],
    pub tag: Option<&'a str>,
    pub moc_id: Option<&'a str>,
    pub query: Option<&'a str>,
    pub min_value: Option<u8>,
    pub custom_filter: &'a [String],
}

pub struct ExpansionOptions {
    pub transitive: bool,
    pub backlinks: bool,
    pub related_threshold: Option<f64>,
}

pub struct OutputOptions {
    pub max_chars: Option<usize>,
    pub with_body: bool,
    pub safety_banner: bool,
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
