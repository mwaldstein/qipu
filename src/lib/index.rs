//! Index infrastructure for qipu
//!
//! Per spec (specs/indexing-search.md):
//! - Derived indexes: metadata, tags, backlinks, graph
//! - Cache location: `.qipu/.cache/*.json`
//! - Incremental indexing with mtime/hash tracking
//! - Link extraction from wiki links, markdown links, and typed frontmatter links

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::lib::error::{QipuError, Result};
use crate::lib::note::{Note, NoteType};
use crate::lib::store::Store;

/// Ripgrep JSON output variants
#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
enum RipgrepMatch {
    Begin {
        path: String,
    },
    End {
        path: String,
    },
    Match {
        path: String,
        lines: String,
        line_number: u64,
        absolute_offset: u64,
    },
}

/// Link source - where the link was defined
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkSource {
    /// Link defined in frontmatter `links[]` array
    Typed,
    /// Link extracted from markdown body (wiki-style or markdown links)
    Inline,
}

impl std::fmt::Display for LinkSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkSource::Typed => write!(f, "typed"),
            LinkSource::Inline => write!(f, "inline"),
        }
    }
}

/// An edge in the note graph
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edge {
    /// Source note ID
    pub from: String,
    /// Target note ID
    pub to: String,
    /// Link type (related, derived-from, supports, contradicts, part-of)
    #[serde(rename = "type")]
    pub link_type: String,
    /// Where the link was defined
    pub source: LinkSource,
}

/// Metadata for a single note (stored in index)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteMetadata {
    /// Note ID
    pub id: String,
    /// Note title
    pub title: String,
    /// Note type
    #[serde(rename = "type")]
    pub note_type: NoteType,
    /// Tags
    pub tags: Vec<String>,
    /// File path relative to store
    pub path: String,
    /// Creation timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<DateTime<Utc>>,
    /// Last update timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<DateTime<Utc>>,
}

/// File metadata for incremental indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileEntry {
    /// File modification time
    mtime: u64,
    /// Note ID in this file
    note_id: String,
}

/// The complete index structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Index {
    /// Index format version
    pub version: u32,
    /// Metadata index: id -> note metadata
    pub metadata: HashMap<String, NoteMetadata>,
    /// Tag index: tag -> [note ids]
    pub tags: HashMap<String, Vec<String>>,
    /// Graph: all edges
    pub edges: Vec<Edge>,
    /// Unresolved links (links to non-existent IDs)
    pub unresolved: HashSet<String>,
    /// File tracking for incremental updates
    #[serde(default)]
    files: HashMap<PathBuf, FileEntry>,
}

/// Current index format version
pub const INDEX_VERSION: u32 = 1;

impl Index {
    /// Create a new empty index
    pub fn new() -> Self {
        Index {
            version: INDEX_VERSION,
            metadata: HashMap::new(),
            tags: HashMap::new(),
            edges: Vec::new(),
            unresolved: HashSet::new(),
            files: HashMap::new(),
        }
    }

    /// Load index from cache directory
    pub fn load(cache_dir: &Path) -> Result<Self> {
        let index_path = cache_dir.join("index.json");
        if !index_path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(&index_path)?;
        let index: Index = serde_json::from_str(&content)?;

        // Check version compatibility
        if index.version != INDEX_VERSION {
            // Version mismatch - return empty index to force rebuild
            return Ok(Self::new());
        }

        Ok(index)
    }

    /// Save index to cache directory
    ///
    /// Per specs/cli-tool.md: "Avoid writing derived caches unless command explicitly calls for it"
    /// This function only writes if the index content has actually changed.
    pub fn save(&self, cache_dir: &Path) -> Result<()> {
        fs::create_dir_all(cache_dir)?;
        let index_path = cache_dir.join("index.json");
        let new_content = serde_json::to_string_pretty(self)?;

        // Filesystem hygiene: only write if content actually changed
        // This avoids unnecessary cache file timestamp updates
        let should_write = if index_path.exists() {
            match fs::read_to_string(&index_path) {
                Ok(existing) => existing != new_content,
                Err(_) => true, // If we can't read, write anyway
            }
        } else {
            true // File doesn't exist, must write
        };

        if should_write {
            fs::write(index_path, new_content)?;
        }

        Ok(())
    }

    /// Get all note IDs
    #[allow(dead_code)]
    pub fn note_ids(&self) -> impl Iterator<Item = &str> {
        self.metadata.keys().map(|s| s.as_str())
    }

    /// Get metadata for a note by ID
    pub fn get_metadata(&self, id: &str) -> Option<&NoteMetadata> {
        self.metadata.get(id)
    }

    /// Get notes by tag
    #[allow(dead_code)]
    pub fn get_notes_by_tag(&self, tag: &str) -> Vec<&str> {
        self.tags
            .get(tag)
            .map(|ids| ids.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get outbound edges from a note
    pub fn get_outbound_edges(&self, id: &str) -> Vec<&Edge> {
        self.edges.iter().filter(|e| e.from == id).collect()
    }

    /// Get inbound edges (backlinks) to a note
    pub fn get_inbound_edges(&self, id: &str) -> Vec<&Edge> {
        self.edges.iter().filter(|e| e.to == id).collect()
    }

    /// Get all edges for a note (both directions)
    #[allow(dead_code)]
    pub fn get_all_edges(&self, id: &str) -> Vec<&Edge> {
        self.edges
            .iter()
            .filter(|e| e.from == id || e.to == id)
            .collect()
    }

    /// Check if a note ID exists in the index
    pub fn contains(&self, id: &str) -> bool {
        self.metadata.contains_key(id)
    }
}

/// Index builder - handles construction and updates
pub struct IndexBuilder<'a> {
    store: &'a Store,
    index: Index,
    rebuild: bool,
}

impl<'a> IndexBuilder<'a> {
    /// Create a new index builder
    pub fn new(store: &'a Store) -> Self {
        IndexBuilder {
            store,
            index: Index::new(),
            rebuild: false,
        }
    }

    /// Load existing index (for incremental updates)
    pub fn load_existing(mut self) -> Result<Self> {
        if !self.rebuild {
            self.index = Index::load(&self.store.root().join(".cache"))?;
        }
        Ok(self)
    }

    /// Force full rebuild (ignore existing index)
    pub fn rebuild(mut self) -> Self {
        self.rebuild = true;
        self.index = Index::new();
        self
    }

    /// Build or update the index
    pub fn build(mut self) -> Result<Index> {
        let notes = self.store.list_notes()?;
        let all_ids: HashSet<_> = notes.iter().map(|n| n.id().to_string()).collect();

        // Track which files we've seen this pass
        let mut seen_files: HashSet<PathBuf> = HashSet::new();

        // Clear edges for rebuild (we'll rebuild them all)
        self.index.edges.clear();
        self.index.unresolved.clear();

        // Process each note
        for note in &notes {
            let path = match &note.path {
                Some(p) => p.clone(),
                None => continue,
            };

            seen_files.insert(path.clone());

            // Check if we need to re-index this file
            let needs_reindex = self.rebuild || self.file_changed(&path);

            if needs_reindex {
                // Update metadata
                let meta = NoteMetadata {
                    id: note.id().to_string(),
                    title: note.title().to_string(),
                    note_type: note.note_type(),
                    tags: note.frontmatter.tags.clone(),
                    path: path.display().to_string(),
                    created: note.frontmatter.created,
                    updated: note.frontmatter.updated,
                };

                // Update tag index
                for tag in &meta.tags {
                    self.index
                        .tags
                        .entry(tag.clone())
                        .or_default()
                        .push(meta.id.clone());
                }

                self.index.metadata.insert(meta.id.clone(), meta);

                // Track file for incremental updates
                if let Ok(mtime) = get_mtime(&path) {
                    self.index.files.insert(
                        path.clone(),
                        FileEntry {
                            mtime,
                            note_id: note.id().to_string(),
                        },
                    );
                }
            }

            // Always extract links (edges need complete rebuild for consistency)
            let edges = extract_links(note, &all_ids, &mut self.index.unresolved);
            self.index.edges.extend(edges);
        }

        // Remove entries for deleted files
        let deleted: Vec<_> = self
            .index
            .files
            .keys()
            .filter(|p| !seen_files.contains(*p))
            .cloned()
            .collect();

        for path in deleted {
            if let Some(entry) = self.index.files.remove(&path) {
                self.index.metadata.remove(&entry.note_id);
            }
        }

        // Deduplicate tag lists
        for ids in self.index.tags.values_mut() {
            ids.sort();
            ids.dedup();
        }

        // Sort edges for determinism
        self.index.edges.sort_by(|a, b| {
            a.from
                .cmp(&b.from)
                .then_with(|| a.link_type.cmp(&b.link_type))
                .then_with(|| a.to.cmp(&b.to))
        });

        Ok(self.index)
    }

    /// Check if a file has changed since last index
    fn file_changed(&self, path: &Path) -> bool {
        let current_mtime = match get_mtime(path) {
            Ok(m) => m,
            Err(_) => return true,
        };

        match self.index.files.get(path) {
            Some(entry) => entry.mtime != current_mtime,
            None => true,
        }
    }
}

/// Get file modification time as unix timestamp
fn get_mtime(path: &Path) -> Result<u64> {
    let metadata = fs::metadata(path)?;
    let mtime = metadata
        .modified()
        .map_err(|e| QipuError::Other(format!("failed to get mtime: {}", e)))?;
    Ok(mtime
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs())
}

/// Extract all links from a note
fn extract_links(
    note: &Note,
    valid_ids: &HashSet<String>,
    unresolved: &mut HashSet<String>,
) -> Vec<Edge> {
    let mut edges = Vec::new();
    let from_id = note.id().to_string();

    // Extract typed links from frontmatter
    for link in &note.frontmatter.links {
        let to_id = link.id.clone();
        if !valid_ids.contains(&to_id) {
            unresolved.insert(to_id.clone());
        }
        edges.push(Edge {
            from: from_id.clone(),
            to: to_id,
            link_type: link.link_type.to_string(),
            source: LinkSource::Typed,
        });
    }

    // Extract wiki links from body: [[id]] or [[id|label]]
    let wiki_link_re = Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]").unwrap();
    for cap in wiki_link_re.captures_iter(&note.body) {
        let to_id = cap[1].trim().to_string();
        if to_id.is_empty() {
            continue;
        }
        if !valid_ids.contains(&to_id) {
            unresolved.insert(to_id.clone());
        }
        // Inline links default to "related" type
        edges.push(Edge {
            from: from_id.clone(),
            to: to_id,
            link_type: "related".to_string(),
            source: LinkSource::Inline,
        });
    }

    // Extract markdown links to qipu notes: [text](qp-xxxx) or [text](./qp-xxxx-slug.md)
    let md_link_re = Regex::new(r"\[([^\]]*)\]\(([^)]+)\)").unwrap();
    for cap in md_link_re.captures_iter(&note.body) {
        let target = cap[2].trim();

        // Check if this looks like a qipu note reference
        let to_id = if target.starts_with("qp-") {
            // Direct ID reference
            target.split('-').take(2).collect::<Vec<_>>().join("-")
        } else if target.contains("qp-") {
            // Path reference like ./qp-xxxx-slug.md
            if let Some(start) = target.find("qp-") {
                let rest = &target[start..];
                // Extract the ID portion (qp-xxxx)
                let end = rest
                    .find('-')
                    .and_then(|first| rest[first + 1..].find('-').map(|second| first + 1 + second));
                match end {
                    Some(end) => rest[..end].to_string(),
                    None => rest.trim_end_matches(".md").to_string(),
                }
            } else {
                continue;
            }
        } else {
            continue;
        };

        if to_id.is_empty() || !to_id.starts_with("qp-") {
            continue;
        }

        if !valid_ids.contains(&to_id) {
            unresolved.insert(to_id.clone());
        }

        edges.push(Edge {
            from: from_id.clone(),
            to: to_id,
            link_type: "related".to_string(),
            source: LinkSource::Inline,
        });
    }

    // Deduplicate edges (same from, to, type, source)
    edges.sort_by(|a, b| {
        a.to.cmp(&b.to)
            .then_with(|| a.link_type.cmp(&b.link_type))
            .then_with(|| format!("{:?}", a.source).cmp(&format!("{:?}", b.source)))
    });
    edges.dedup_by(|a, b| a.to == b.to && a.link_type == b.link_type && a.source == b.source);

    edges
}

/// Search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Note ID (may be canonical digest if compaction is resolved)
    pub id: String,
    /// Note title
    pub title: String,
    /// Note type
    #[serde(rename = "type")]
    pub note_type: NoteType,
    /// Tags
    pub tags: Vec<String>,
    /// File path
    pub path: String,
    /// Match context (snippet showing where the match occurred)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_context: Option<String>,
    /// Relevance score (higher is better)
    pub relevance: f64,
    /// Via field - indicates which compacted note triggered this result
    /// Per spec (specs/compaction.md line 122): when a digest appears because
    /// a compacted note matched, annotate with via=<matching-note-id>
    #[serde(skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
}

/// Check if ripgrep is available on the system
fn is_ripgrep_available() -> bool {
    Command::new("rg")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Search using ripgrep for faster file finding
///
/// This is an optimization that leverages ripgrep if available.
/// Falls back to embedded search if ripgrep is not found.
fn search_with_ripgrep(
    store: &Store,
    index: &Index,
    query: &str,
    type_filter: Option<NoteType>,
    tag_filter: Option<&str>,
) -> Result<Vec<SearchResult>> {
    let query_lower = query.to_lowercase();
    let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

    // Use ripgrep with JSON output to get both matches and context snippets
    let mut rg_cmd = Command::new("rg");
    rg_cmd
        .arg("--json")
        .arg("--case-insensitive")
        .arg("--no-heading")
        .arg("--with-filename")
        .arg("--context-before=1")
        .arg("--context-after=1")
        .arg("--max-columns=200");

    // Add search pattern (OR all terms together)
    let pattern = query_terms.join("|");
    rg_cmd.arg(&pattern);

    // Search in notes and mocs directories
    rg_cmd.arg(store.root().join("notes"));
    rg_cmd.arg(store.root().join("mocs"));

    let output = match rg_cmd.output() {
        Ok(output) => output,
        Err(_) => {
            // If ripgrep fails to run, fall back to embedded search
            return search_embedded(store, index, query, type_filter, tag_filter);
        }
    };

    // Parse ripgrep JSON output to get matches and contexts
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut matching_paths: HashSet<PathBuf> = HashSet::new();
    let mut path_contexts: HashMap<PathBuf, String> = HashMap::new();

    for line in stdout.lines() {
        if let Ok(rg_match) = serde_json::from_str::<RipgrepMatch>(line) {
            match rg_match {
                RipgrepMatch::Begin { path, .. } | RipgrepMatch::End { path, .. } => {
                    matching_paths.insert(PathBuf::from(path));
                }
                RipgrepMatch::Match { path, lines, .. } => {
                    let path_buf = PathBuf::from(&path);
                    matching_paths.insert(path_buf.clone());

                    // Store first context snippet for this file
                    if !path_contexts.contains_key(&path_buf) {
                        let context = lines.replace('\n', " ").trim().to_string();
                        if !context.is_empty() {
                            path_contexts.insert(path_buf, format!("...{}...", context));
                        }
                    }
                }
            }
        }
    }

    // If ripgrep found no matches, use embedded search as fallback
    // (ripgrep exit code 1 means no matches, not an error)
    if matching_paths.is_empty() {
        return search_embedded(store, index, query, type_filter, tag_filter);
    }

    // Build results from matching files using index metadata
    let mut results = Vec::new();

    // For performance, limit processing to first 500 matching files
    // Users rarely look beyond the first few pages of results
    let mut processed = 0;
    const MAX_FILES_TO_PROCESS: usize = 500;

    for meta in index.metadata.values() {
        // Skip if path doesn't match ripgrep results
        let path = PathBuf::from(&meta.path);
        if !matching_paths.contains(&path) {
            continue;
        }

        // Apply type filter
        if let Some(t) = type_filter {
            if meta.note_type != t {
                continue;
            }
        }

        // Apply tag filter
        if let Some(tag) = tag_filter {
            if !meta.tags.contains(&tag.to_string()) {
                continue;
            }
        }

        // Calculate relevance score (same logic as embedded search)
        let mut relevance = 0.0;
        let mut match_context = None;

        let title_lower = meta.title.to_lowercase();

        // Title matches (high weight)
        for term in &query_terms {
            if title_lower.contains(term) {
                relevance += 10.0;
                if title_lower == *term {
                    relevance += 5.0;
                }
            }
        }

        // Tag matches (medium weight)
        for tag in &meta.tags {
            let tag_lower = tag.to_lowercase();
            for term in &query_terms {
                if tag_lower == *term {
                    relevance += 7.0;
                } else if tag_lower.contains(term) {
                    relevance += 3.0;
                }
            }
        }

        // Use pre-fetched context and add relevance for body matches
        if let Some(context) = path_contexts.get(&path) {
            // We know this file matched in ripgrep, so add body relevance
            for term in &query_terms {
                if context.to_lowercase().contains(term) {
                    relevance += 2.0;
                }
            }
            match_context = Some(context.clone());
        }

        // Recency boost (prefer recently created notes)
        if let Some(created) = meta.created {
            let age_days = (Utc::now() - created).num_days();
            if age_days < 7 {
                relevance += 1.0;
            } else if age_days < 30 {
                relevance += 0.5;
            }
        }

        results.push(SearchResult {
            id: meta.id.clone(),
            title: meta.title.clone(),
            note_type: meta.note_type,
            tags: meta.tags.clone(),
            path: meta.path.clone(),
            match_context,
            relevance,
            via: None,
        });

        processed += 1;
        if processed >= MAX_FILES_TO_PROCESS {
            break;
        }
    }

    // Sort by relevance (descending)
    results.sort_by(|a, b| {
        b.relevance
            .partial_cmp(&a.relevance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Limit to top 100 results for better performance
    results.truncate(100);

    Ok(results)
}

/// Embedded text search (fallback when ripgrep not available)
fn search_embedded(
    store: &Store,
    index: &Index,
    query: &str,
    type_filter: Option<NoteType>,
    tag_filter: Option<&str>,
) -> Result<Vec<SearchResult>> {
    let query_lower = query.to_lowercase();
    let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

    if query_terms.is_empty() {
        return Ok(Vec::new());
    }

    let mut results = Vec::new();

    for meta in index.metadata.values() {
        // Apply type filter
        if let Some(t) = type_filter {
            if meta.note_type != t {
                continue;
            }
        }

        // Apply tag filter
        if let Some(tag) = tag_filter {
            if !meta.tags.contains(&tag.to_string()) {
                continue;
            }
        }

        // Calculate relevance score
        let mut relevance = 0.0;
        let mut matched = false;
        let mut match_context = None;

        let title_lower = meta.title.to_lowercase();

        // Title matches (high weight)
        for term in &query_terms {
            if title_lower.contains(term) {
                relevance += 10.0;
                matched = true;
                // Exact title match bonus
                if title_lower == *term {
                    relevance += 5.0;
                }
            }
        }

        // Tag matches (medium weight)
        for tag in &meta.tags {
            let tag_lower = tag.to_lowercase();
            for term in &query_terms {
                if tag_lower == *term {
                    relevance += 7.0;
                    matched = true;
                } else if tag_lower.contains(term) {
                    relevance += 3.0;
                    matched = true;
                }
            }
        }

        // Body search (lower weight, requires reading file)
        if !matched || relevance < 10.0 {
            // Read note content to search body
            if let Ok(note) = store.get_note(&meta.id) {
                let body_lower = note.body.to_lowercase();
                for term in &query_terms {
                    if body_lower.contains(term) {
                        relevance += 2.0;
                        matched = true;

                        // Extract context snippet
                        if match_context.is_none() {
                            if let Some(pos) = body_lower.find(term) {
                                let start = pos.saturating_sub(40);
                                let end = (pos + term.len() + 40).min(note.body.len());
                                let snippet = &note.body[start..end];
                                let snippet = snippet.replace('\n', " ");
                                match_context = Some(format!("...{}...", snippet.trim()));
                            }
                        }
                    }
                }
            }
        }

        // Recency boost (prefer recently updated notes)
        if matched {
            if let Some(created) = meta.created {
                let age_days = (Utc::now() - created).num_days();
                if age_days < 7 {
                    relevance += 1.0;
                } else if age_days < 30 {
                    relevance += 0.5;
                }
            }

            results.push(SearchResult {
                id: meta.id.clone(),
                title: meta.title.clone(),
                note_type: meta.note_type,
                tags: meta.tags.clone(),
                path: meta.path.clone(),
                match_context,
                relevance,
                via: None,
            });
        }
    }

    // Sort by relevance (descending)
    results.sort_by(|a, b| {
        b.relevance
            .partial_cmp(&a.relevance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(results)
}

/// Simple text search over the index
///
/// Uses ripgrep if available for faster file finding, otherwise falls back
/// to embedded matcher. Ranking: title matches > exact tag matches > body matches,
/// with recency boost.
pub fn search(
    store: &Store,
    index: &Index,
    query: &str,
    type_filter: Option<NoteType>,
    tag_filter: Option<&str>,
) -> Result<Vec<SearchResult>> {
    if is_ripgrep_available() {
        eprintln!("Using ripgrep search");
        search_with_ripgrep(store, index, query, type_filter, tag_filter)
    } else {
        eprintln!("Using embedded search");
        search_embedded(store, index, query, type_filter, tag_filter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::note::{Note, NoteFrontmatter};

    fn make_note(id: &str, title: &str, body: &str) -> Note {
        let fm = NoteFrontmatter::new(id.to_string(), title.to_string());
        Note::new(fm, body.to_string())
    }

    #[test]
    fn test_extract_wiki_links() {
        let mut note = make_note("qp-a1", "Test", "See [[qp-b2]] and [[qp-c3|some label]]");
        note.frontmatter.links = vec![];

        let valid_ids: HashSet<_> = ["qp-a1", "qp-b2", "qp-c3"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let mut unresolved = HashSet::new();

        let edges = extract_links(&note, &valid_ids, &mut unresolved);

        assert_eq!(edges.len(), 2);
        assert!(edges.iter().any(|e| e.to == "qp-b2"));
        assert!(edges.iter().any(|e| e.to == "qp-c3"));
        assert!(edges.iter().all(|e| e.source == LinkSource::Inline));
        assert!(edges.iter().all(|e| e.link_type == "related"));
    }

    #[test]
    fn test_extract_typed_links() {
        use crate::lib::note::{LinkType, TypedLink};

        let mut note = make_note("qp-a1", "Test", "Body text");
        note.frontmatter.links = vec![
            TypedLink {
                link_type: LinkType::DerivedFrom,
                id: "qp-b2".to_string(),
            },
            TypedLink {
                link_type: LinkType::Supports,
                id: "qp-c3".to_string(),
            },
        ];

        let valid_ids: HashSet<_> = ["qp-a1", "qp-b2", "qp-c3"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let mut unresolved = HashSet::new();

        let edges = extract_links(&note, &valid_ids, &mut unresolved);

        assert_eq!(edges.len(), 2);
        assert!(edges
            .iter()
            .any(|e| e.to == "qp-b2" && e.link_type == "derived-from"));
        assert!(edges
            .iter()
            .any(|e| e.to == "qp-c3" && e.link_type == "supports"));
        assert!(edges.iter().all(|e| e.source == LinkSource::Typed));
    }

    #[test]
    fn test_unresolved_links() {
        let note = make_note("qp-a1", "Test", "See [[qp-missing]]");

        let valid_ids: HashSet<_> = ["qp-a1"].iter().map(|s| s.to_string()).collect();
        let mut unresolved = HashSet::new();

        let edges = extract_links(&note, &valid_ids, &mut unresolved);

        assert_eq!(edges.len(), 1);
        assert!(unresolved.contains("qp-missing"));
    }

    #[test]
    fn test_index_new() {
        let index = Index::new();
        assert_eq!(index.version, INDEX_VERSION);
        assert!(index.metadata.is_empty());
        assert!(index.tags.is_empty());
        assert!(index.edges.is_empty());
    }
}
