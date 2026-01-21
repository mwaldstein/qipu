use crate::lib::note::{LinkType, Note};

/// Options for the context command
pub struct ContextOptions<'a> {
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
