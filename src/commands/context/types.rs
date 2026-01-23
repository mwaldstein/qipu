use crate::lib::note::{LinkType, Note};

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
    pub max_tokens: Option<usize>,
    pub model: &'a str,
    pub transitive: bool,
    pub with_body: bool,
    pub safety_banner: bool,
    pub related_threshold: Option<f64>,
    pub backlinks: bool,
    pub min_value: Option<u8>,
    pub custom_filter: &'a [String],
    pub include_custom: bool,
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
