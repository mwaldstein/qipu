use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pack file header
#[derive(Debug, Deserialize, Serialize)]
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
#[derive(Debug, Deserialize, Serialize, Clone)]
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
    #[serde(default)]
    pub custom: HashMap<String, serde_json::Value>,
}

/// Pack entry for a source reference
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PackSource {
    pub url: String,
    pub title: Option<String>,
    pub accessed: Option<String>,
}

/// Pack entry for a link
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PackLink {
    pub from: String,
    pub to: String,
    pub link_type: Option<String>,
    pub inline: bool,
}

/// Pack entry for an attachment
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PackAttachment {
    pub path: String,
    pub name: String,
    pub data: String, // Base64 encoded
    pub content_type: Option<String>,
}

/// Complete pack data structure
#[derive(Debug, Deserialize)]
pub struct PackData {
    pub header: PackHeader,
    pub notes: Vec<PackNote>,
    pub links: Vec<PackLink>,
    pub attachments: Vec<PackAttachment>,
}
