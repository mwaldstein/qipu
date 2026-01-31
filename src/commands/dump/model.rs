use qipu_core::graph::Direction;
use serde::Serialize;

/// Options for dump command
pub struct DumpOptions<'a> {
    pub note_ids: &'a [String],
    pub tag: Option<&'a str>,
    pub moc_id: Option<&'a str>,
    pub query: Option<&'a str>,
    pub direction: Direction,
    pub max_hops: u32,
    pub type_include: &'a [String],
    pub typed_only: bool,
    pub inline_only: bool,
    pub include_attachments: bool,
    pub output: Option<&'a std::path::Path>,
}

/// Pack entry for a link
#[derive(Debug, Clone, Serialize)]
pub struct PackLink {
    pub from: String,
    pub to: String,
    pub link_type: Option<String>,
    pub inline: bool,
}

/// Pack entry for an attachment
#[derive(Debug, Clone, Serialize)]
pub struct PackAttachment {
    pub path: String,
    pub name: String,
    pub data: Vec<u8>,
    pub content_type: Option<String>,
}
