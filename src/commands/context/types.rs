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
