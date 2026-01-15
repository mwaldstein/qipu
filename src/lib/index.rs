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
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::lib::error::{QipuError, Result};
use crate::lib::logging;
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
        #[allow(dead_code)]
        line_number: u64,
        #[allow(dead_code)]
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
    /// Reverse mapping: note_id -> file_path (for fast lookup)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    id_to_path: HashMap<String, PathBuf>,
}

/// Current index format version
pub const INDEX_VERSION: u32 = 1;

const INDEX_META_FILE: &str = "index_meta.json";
const INDEX_METADATA_FILE: &str = "metadata.json";
const INDEX_TAGS_FILE: &str = "tags.json";
const INDEX_EDGES_FILE: &str = "edges.json";
const INDEX_UNRESOLVED_FILE: &str = "unresolved.json";
const INDEX_FILES_FILE: &str = "files.json";
const INDEX_ID_TO_PATH_FILE: &str = "id_to_path.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndexMeta {
    version: u32,
}

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
            id_to_path: HashMap::new(),
        }
    }

    /// Load index from cache directory
    pub fn load(cache_dir: &Path) -> Result<Self> {
        let meta_path = cache_dir.join(INDEX_META_FILE);
        if meta_path.exists() {
            let content = fs::read_to_string(&meta_path)?;
            let meta: IndexMeta = serde_json::from_str(&content)?;
            if meta.version != INDEX_VERSION {
                return Ok(Self::new());
            }

            let metadata = load_cache_file(&cache_dir.join(INDEX_METADATA_FILE))?;
            let tags = load_cache_file(&cache_dir.join(INDEX_TAGS_FILE))?;
            let edges = load_cache_file(&cache_dir.join(INDEX_EDGES_FILE))?;
            let unresolved = load_cache_file(&cache_dir.join(INDEX_UNRESOLVED_FILE))?;
            let files = load_cache_file(&cache_dir.join(INDEX_FILES_FILE))?;
            let id_to_path = load_cache_file(&cache_dir.join(INDEX_ID_TO_PATH_FILE))?;

            return Ok(Index {
                version: meta.version,
                metadata,
                tags,
                edges,
                unresolved,
                files,
                id_to_path,
            });
        }

        let index_path = cache_dir.join("index.json");
        if !index_path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(&index_path)?;
        let index: Index = serde_json::from_str(&content)?;

        if index.version != INDEX_VERSION {
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

        let meta = IndexMeta {
            version: INDEX_VERSION,
        };

        write_cache_file(&cache_dir.join(INDEX_META_FILE), &meta)?;
        write_cache_file(&cache_dir.join(INDEX_METADATA_FILE), &self.metadata)?;
        write_cache_file(&cache_dir.join(INDEX_TAGS_FILE), &self.tags)?;
        write_cache_file(&cache_dir.join(INDEX_EDGES_FILE), &self.edges)?;
        write_cache_file(&cache_dir.join(INDEX_UNRESOLVED_FILE), &self.unresolved)?;
        write_cache_file(&cache_dir.join(INDEX_FILES_FILE), &self.files)?;
        write_cache_file(&cache_dir.join(INDEX_ID_TO_PATH_FILE), &self.id_to_path)?;

        let legacy_path = cache_dir.join("index.json");
        if legacy_path.exists() {
            fs::remove_file(legacy_path)?;
        }

        Ok(())
    }

    /// Get all note IDs
    #[allow(dead_code)]
    pub fn note_ids(&self) -> impl Iterator<Item = &str> {
        self.metadata.keys().map(|s| s.as_str())
    }

    /// Get file path for a note ID (for fast lookup)
    pub fn get_note_path(&self, note_id: &str) -> Option<&PathBuf> {
        self.id_to_path.get(note_id)
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

fn load_cache_file<T: DeserializeOwned + Default>(path: &Path) -> Result<T> {
    if !path.exists() {
        return Ok(T::default());
    }

    let content = fs::read_to_string(path)?;
    let value = serde_json::from_str(&content)?;
    Ok(value)
}

fn write_cache_file<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let new_content = serde_json::to_string_pretty(value)?;
    write_if_changed(path, &new_content)
}

fn write_if_changed(path: &Path, new_content: &str) -> Result<()> {
    let should_write = if path.exists() {
        match fs::read_to_string(path) {
            Ok(existing) => existing != new_content,
            Err(_) => true,
        }
    } else {
        true
    };

    if should_write {
        fs::write(path, new_content)?;
    }

    Ok(())
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
                self.prune_note_indexes(&path, note.id());

                // Update metadata
                let meta = NoteMetadata {
                    id: note.id().to_string(),
                    title: note.title().to_string(),
                    note_type: note.note_type(),
                    tags: note.frontmatter.tags.clone(),
                    path: self.relative_path(&path),
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

                    // Add reverse mapping for fast lookup
                    self.index.id_to_path.insert(note.id().to_string(), path);
                }
            } else {
                let relative_path = self.relative_path(&path);
                if let Some(meta) = self.index.metadata.get_mut(note.id()) {
                    if meta.path != relative_path {
                        meta.path = relative_path;
                    }
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
                let should_remove = self
                    .index
                    .id_to_path
                    .get(&entry.note_id)
                    .map(|current| current == &path)
                    .unwrap_or(true);

                if should_remove {
                    self.remove_note_from_tags(&entry.note_id);
                    self.index.metadata.remove(&entry.note_id);
                    self.index.id_to_path.remove(&entry.note_id);
                }
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

    fn relative_path(&self, path: &Path) -> String {
        path.strip_prefix(self.store.root())
            .unwrap_or(path)
            .to_string_lossy()
            .to_string()
    }

    fn prune_note_indexes(&mut self, path: &Path, note_id: &str) {
        if let Some(entry) = self.index.files.get(path) {
            let existing_id = entry.note_id.clone();
            self.remove_note_from_tags(&existing_id);

            if existing_id != note_id {
                self.index.metadata.remove(&existing_id);
                self.index.id_to_path.remove(&existing_id);
            }
        } else if self.index.metadata.contains_key(note_id) {
            self.remove_note_from_tags(note_id);
        }
    }

    fn remove_note_from_tags(&mut self, note_id: &str) {
        let mut empty_tags = Vec::new();
        for (tag, ids) in self.index.tags.iter_mut() {
            ids.retain(|id| id != note_id);
            if ids.is_empty() {
                empty_tags.push(tag.clone());
            }
        }

        for tag in empty_tags {
            self.index.tags.remove(&tag);
        }
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
    let wiki_link_re = match Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]") {
        Ok(re) => re,
        Err(e) => {
            if logging::verbose_enabled() {
                eprintln!("Warning: Failed to compile wiki link regex: {}", e);
            }
            return edges; // Return empty edges if regex fails
        }
    };
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
    let md_link_re = match Regex::new(r"\[([^\]]*)\]\(([^)]+)\)") {
        Ok(re) => re,
        Err(e) => {
            if logging::verbose_enabled() {
                eprintln!("Warning: Failed to compile markdown link regex: {}", e);
            }
            return edges; // Return empty edges if regex fails
        }
    };
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

fn absolute_meta_path(store: &Store, meta_path: &str) -> PathBuf {
    let path = Path::new(meta_path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        store.root().join(path)
    }
}

fn normalize_meta_path(store: &Store, meta_path: &str) -> String {
    let path = Path::new(meta_path);
    if path.is_absolute() {
        path.strip_prefix(store.root())
            .unwrap_or(path)
            .to_string_lossy()
            .to_string()
    } else {
        meta_path.to_string()
    }
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
                    if let std::collections::hash_map::Entry::Vacant(e) =
                        path_contexts.entry(path_buf)
                    {
                        let context = lines.replace('\n', " ").trim().to_string();
                        if !context.is_empty() {
                            e.insert(format!("...{}...", context));
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

    // For performance, limit processing to first 200 matching files
    // Users rarely look beyond the first few pages of results
    let mut processed = 0;
    const MAX_FILES_TO_PROCESS: usize = 200;

    // Create a faster lookup for matching paths
    let matching_path_set: HashSet<&PathBuf> = matching_paths.iter().collect();

    // Iterate over metadata in deterministic order (sorted by note ID)
    let mut note_ids: Vec<&String> = index.metadata.keys().collect();
    note_ids.sort();
    for note_id in note_ids {
        let meta = &index.metadata[note_id];
        // Skip if path doesn't match ripgrep results
        let path = absolute_meta_path(store, &meta.path);
        if !matching_path_set.contains(&path) {
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

        // Calculate relevance score (same logic as embedded search) - optimized
        let mut relevance = 0.0;
        let mut match_context = None;

        let title_lower = meta.title.to_lowercase();

        // Title matches (high weight) - optimized early exit
        for term in &query_terms {
            if title_lower.contains(term) {
                relevance += 10.0;
                if title_lower == *term {
                    relevance += 5.0;
                }
                // Early exit for strong title matches
                if relevance >= 15.0 {
                    break;
                }
            }
        }

        // Tag matches (medium weight) - only check if needed
        if relevance < 15.0 {
            for tag in &meta.tags {
                let tag_lower = tag.to_lowercase();
                for term in &query_terms {
                    if tag_lower == *term {
                        relevance += 7.0;
                    } else if tag_lower.contains(term) {
                        relevance += 3.0;
                    }
                }
                if relevance >= 10.0 {
                    break;
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

        // Only include results with some relevance
        if relevance > 0.0 {
            results.push(SearchResult {
                id: meta.id.clone(),
                title: meta.title.clone(),
                note_type: meta.note_type,
                tags: meta.tags.clone(),
                path: normalize_meta_path(store, &meta.path),
                match_context,
                relevance,
                via: None,
            });
        }

        processed += 1;
        if processed >= MAX_FILES_TO_PROCESS {
            break;
        }

        // Early exit if we have enough strong results
        if results.len() >= 50 {
            let strong_count = results.iter().filter(|r| r.relevance >= 10.0).count();
            if strong_count >= 20 {
                break;
            }
        }
    }

    // Sort by relevance (descending), then by note ID (ascending) for deterministic tie-breaking
    results.sort_by(|a, b| {
        b.relevance
            .partial_cmp(&a.relevance)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.id.cmp(&b.id))
    });

    // Limit results to improve performance for large stores
    results.truncate(200);

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
    let tag_string = tag_filter.map(|t| t.to_string());

    // Early exit: if we have strong title matches in metadata, limit body reads
    let mut strong_title_matches = 0;
    const MAX_STRONG_MATCHES: usize = 50;

    // Iterate over metadata in deterministic order (sorted by note ID)
    let mut note_ids: Vec<&String> = index.metadata.keys().collect();
    note_ids.sort();
    for note_id in note_ids {
        let meta = &index.metadata[note_id];
        // Apply type filter
        if let Some(t) = type_filter {
            if meta.note_type != t {
                continue;
            }
        }

        // Apply tag filter
        if let Some(ref tag) = tag_string {
            if !meta.tags.contains(tag) {
                continue;
            }
        }

        // Calculate relevance score
        let mut relevance = 0.0;
        let mut matched = false;
        let mut match_context = None;

        let title_lower = meta.title.to_lowercase();

        // Title matches (high weight) - optimized to avoid repeated contains() calls
        for term in &query_terms {
            if title_lower.contains(term) {
                relevance += 10.0;
                matched = true;
                // Exact title match bonus
                if title_lower == *term {
                    relevance += 5.0;
                }
                // Break early if we have a strong title match to avoid unnecessary body reads
                if relevance >= 15.0 {
                    strong_title_matches += 1;
                    break;
                }
            }
        }

        // Skip body search if we already have enough strong matches
        if strong_title_matches >= MAX_STRONG_MATCHES && relevance >= 15.0 {
            matched = true; // Ensure we include this result
        } else {
            // Tag matches (medium weight) - optimized to avoid repeated to_lowercase()
            if !matched || relevance < 15.0 {
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
                    if relevance >= 10.0 {
                        break;
                    }
                }
            }

            // Body search (lower weight, requires reading file) - only if needed and under limit
            if !matched || relevance < 10.0 {
                // Read note content to search body (use index for fast path lookup)
                if let Ok(note) = store.get_note_with_index(&meta.id, index) {
                    let body_lower = note.body.to_lowercase();
                    for term in &query_terms {
                        if body_lower.contains(term) {
                            relevance += 2.0;
                            matched = true;

                            // Extract context snippet - only for first match
                            if match_context.is_none() {
                                if let Some(pos) = body_lower.find(term) {
                                    let start = pos.saturating_sub(40);
                                    let end = (pos + term.len() + 40).min(note.body.len());
                                    let snippet = &note.body[start..end];
                                    let snippet = snippet.replace('\n', " ");
                                    match_context = Some(format!("...{}...", snippet.trim()));
                                    break; // Only get context for first term match
                                }
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
                path: normalize_meta_path(store, &meta.path),
                match_context,
                relevance,
                via: None,
            });
        }
    }

    // Sort by relevance (descending), then by note ID (ascending) for deterministic tie-breaking
    results.sort_by(|a, b| {
        b.relevance
            .partial_cmp(&a.relevance)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.id.cmp(&b.id))
    });

    // Limit results to improve performance for large stores
    results.truncate(200);

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
    // Always try ripgrep first - it's much faster than embedded search
    if is_ripgrep_available() {
        if logging::verbose_enabled() {
            eprintln!("Using ripgrep search");
        }
        search_with_ripgrep(store, index, query, type_filter, tag_filter)
    } else {
        if logging::verbose_enabled() {
            eprintln!("Using embedded search");
        }
        search_embedded(store, index, query, type_filter, tag_filter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::note::{Note, NoteFrontmatter};
    use crate::lib::store::{InitOptions, Store};
    use std::path::PathBuf;
    use std::thread::sleep;
    use std::time::Duration;
    use tempfile::tempdir;

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
    fn test_incremental_index_updates_tags() {
        let dir = tempdir().unwrap();
        let store = Store::init(dir.path(), InitOptions::default()).unwrap();
        let initial_tags = vec!["alpha".to_string()];

        let mut note = store
            .create_note("Tagged Note", None, &initial_tags)
            .unwrap();

        let index = IndexBuilder::new(&store).build().unwrap();
        index.save(&store.root().join(".cache")).unwrap();

        sleep(Duration::from_secs(1));
        note.frontmatter.tags = vec!["beta".to_string()];
        store.save_note(&mut note).unwrap();

        let index = IndexBuilder::new(&store)
            .load_existing()
            .unwrap()
            .build()
            .unwrap();

        assert!(index.get_notes_by_tag("alpha").is_empty());
        assert!(index.get_notes_by_tag("beta").contains(&note.id()));
    }

    #[test]
    fn test_index_cache_roundtrip() {
        let dir = tempdir().unwrap();
        let cache_dir = dir.path().join(".cache");

        let mut index = Index::new();
        index.metadata.insert(
            "qp-a1".to_string(),
            NoteMetadata {
                id: "qp-a1".to_string(),
                title: "Cached Note".to_string(),
                note_type: NoteType::Fleeting,
                tags: vec!["alpha".to_string()],
                path: "notes/qp-a1.md".to_string(),
                created: None,
                updated: None,
            },
        );
        index
            .tags
            .insert("alpha".to_string(), vec!["qp-a1".to_string()]);
        index.edges.push(Edge {
            from: "qp-a1".to_string(),
            to: "qp-b2".to_string(),
            link_type: "related".to_string(),
            source: LinkSource::Inline,
        });
        index.unresolved.insert("qp-missing".to_string());
        index.files.insert(
            PathBuf::from("notes/qp-a1.md"),
            FileEntry {
                mtime: 123,
                note_id: "qp-a1".to_string(),
            },
        );
        index
            .id_to_path
            .insert("qp-a1".to_string(), PathBuf::from("notes/qp-a1.md"));

        index.save(&cache_dir).unwrap();

        let loaded = Index::load(&cache_dir).unwrap();
        let loaded_meta = loaded.metadata.get("qp-a1").unwrap();

        assert_eq!(loaded.version, INDEX_VERSION);
        assert_eq!(loaded.metadata.len(), 1);
        assert_eq!(loaded_meta.title, "Cached Note");
        assert_eq!(loaded_meta.tags, vec!["alpha".to_string()]);
        assert_eq!(
            loaded.tags.get("alpha").unwrap(),
            &vec!["qp-a1".to_string()]
        );
        assert_eq!(loaded.edges.len(), 1);
        assert!(loaded.unresolved.contains("qp-missing"));
        assert_eq!(loaded.files.len(), 1);
        assert_eq!(
            loaded.id_to_path.get("qp-a1").unwrap(),
            &PathBuf::from("notes/qp-a1.md")
        );
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
