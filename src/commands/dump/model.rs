use crate::lib::graph::Direction;
use serde::Serialize;
use std::collections::HashMap;

/// Options for the dump command
pub struct DumpOptions<'a> {
    pub note_ids: &'a [String],
    pub tag: Option<&'a str>,
    pub moc_id: Option<&'a str>,
    pub query: Option<&'a str>,
    pub direction: Direction,
    pub max_hops: u32,
    pub type_include: Vec<String>,
    pub typed_only: bool,
    pub inline_only: bool,
    pub include_attachments: bool,
    pub output: Option<&'a std::path::Path>,
}

/// Pack file header
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct PackHeader {
    pub version: String,
    pub store_version: u32,
    pub created: chrono::DateTime<chrono::Utc>,
    pub store_path: String,
    pub notes_count: usize,
    pub attachments_count: usize,
    pub links_count: usize,
}

/// Pack entry for a note
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct PackNote {
    pub id: String,
    pub title: String,
    pub note_type: String,
    pub tags: Vec<String>,
    pub created: Option<chrono::DateTime<chrono::Utc>>,
    pub updated: Option<chrono::DateTime<chrono::Utc>>,
    pub path: Option<String>,
    pub content: String,
    pub sources: Vec<PackSource>,
    pub summary: Option<String>,
    pub compacts: Vec<String>,
    pub source: Option<String>,
    pub author: Option<String>,
    pub generated_by: Option<String>,
    pub prompt_hash: Option<String>,
    pub verified: Option<bool>,
    pub value: Option<u8>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub custom: HashMap<String, serde_json::Value>,
}

/// Pack entry for a source reference
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct PackSource {
    pub url: String,
    pub title: Option<String>,
    pub accessed: Option<String>,
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
