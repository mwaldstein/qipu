//! Link management commands for qipu
//!
//! Per spec (specs/cli-interface.md, specs/graph-traversal.md):
//! - `qipu link list <id>` - list links for a note
//! - `qipu link add <from> <to> --type <t>` - add typed link
//! - `qipu link remove <from> <to> --type <t>` - remove typed link
//! - `qipu link tree <id>` - traversal tree from note
//! - `qipu link path <from> <to>` - find path between notes

use std::collections::{HashMap, HashSet, VecDeque};

use crate::cli::{Cli, OutputFormat};
use crate::lib::compaction::CompactionContext;
use crate::lib::error::Result;
use crate::lib::index::{Edge, Index, IndexBuilder, LinkSource};
use crate::lib::note::{LinkType, NoteType, TypedLink};
use crate::lib::store::Store;
use serde::Serialize;

/// Direction for link listing/traversal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    /// Outbound links only (links FROM this note)
    Out,
    /// Inbound links only (backlinks TO this note)
    In,
    #[default]
    /// Both directions
    Both,
}

impl std::str::FromStr for Direction {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "out" => Ok(Direction::Out),
            "in" => Ok(Direction::In),
            "both" => Ok(Direction::Both),
            other => Err(format!(
                "unknown direction '{}' (expected: out, in, both)",
                other
            )),
        }
    }
}

/// Link entry for output
#[derive(Debug, Clone, Serialize)]
pub struct LinkEntry {
    /// Direction relative to the queried note
    pub direction: String,
    /// The other note's ID
    pub id: String,
    /// The other note's title (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Link type (related, derived-from, supports, contradicts, part-of)
    #[serde(rename = "type")]
    pub link_type: String,
    /// Link source (typed or inline)
    pub source: String,
}

/// Execute the link list command
///
/// Lists all links for a note, with optional direction and type filters.
pub fn execute_list(
    cli: &Cli,
    store: &Store,
    id_or_path: &str,
    direction: Direction,
    type_filter: Option<&str>,
    typed_only: bool,
    inline_only: bool,
) -> Result<()> {
    // Resolve the note ID
    let note_id = resolve_note_id(store, id_or_path)?;

    // Load or build the index
    let index = IndexBuilder::new(store).load_existing()?.build()?;

    // Build compaction context if needed
    let compaction_ctx = if !cli.no_resolve_compaction {
        let notes = store.list_notes()?;
        Some(CompactionContext::build(&notes)?)
    } else {
        None
    };

    // Canonicalize the note ID to get which note's links we should show
    let canonical_id = if let Some(ref ctx) = compaction_ctx {
        ctx.canon(&note_id)?
    } else {
        note_id.clone()
    };

    // Verify canonical note exists
    if !index.contains(&canonical_id) {
        return Err(crate::lib::error::QipuError::NoteNotFound {
            id: canonical_id.clone(),
        });
    }

    // Collect all raw IDs that map to this canonical ID (for gathering edges)
    let mut source_ids = vec![canonical_id.clone()];
    if let Some(ref ctx) = compaction_ctx {
        // Find all notes that are compacted by this canonical ID
        if let Some(compacted_notes) = ctx.get_compacted_notes(&canonical_id) {
            source_ids.extend(compacted_notes.iter().cloned());
        }
    }

    // Collect links based on direction
    let mut entries = Vec::new();

    // Outbound edges (links FROM this note or any note it compacts)
    if direction == Direction::Out || direction == Direction::Both {
        for source_id in &source_ids {
            for edge in index.get_outbound_edges(source_id) {
                if let Some(mut entry) =
                    filter_and_convert(edge, "out", &index, type_filter, typed_only, inline_only)
                {
                    // Canonicalize the target ID if compaction is enabled
                    if let Some(ref ctx) = compaction_ctx {
                        entry.id = ctx.canon(&entry.id)?;
                        // Update title if it changed due to canonicalization
                        if let Some(meta) = index.get_metadata(&entry.id) {
                            entry.title = Some(meta.title.clone());
                        }
                    }
                    entries.push(entry);
                }
            }
        }
    }

    // Inbound edges (backlinks TO this note or any note it compacts)
    if direction == Direction::In || direction == Direction::Both {
        for source_id in &source_ids {
            for edge in index.get_inbound_edges(source_id) {
                if let Some(mut entry) =
                    filter_and_convert_inbound(edge, &index, type_filter, typed_only, inline_only)
                {
                    // Canonicalize the source ID if compaction is enabled
                    if let Some(ref ctx) = compaction_ctx {
                        entry.id = ctx.canon(&entry.id)?;
                        // Update title if it changed due to canonicalization
                        if let Some(meta) = index.get_metadata(&entry.id) {
                            entry.title = Some(meta.title.clone());
                        }
                    }
                    entries.push(entry);
                }
            }
        }
    }

    // Remove duplicates that may have been created by canonicalization
    entries.sort_by(|a, b| {
        a.direction
            .cmp(&b.direction)
            .then_with(|| a.link_type.cmp(&b.link_type))
            .then_with(|| a.id.cmp(&b.id))
    });
    entries
        .dedup_by(|a, b| a.direction == b.direction && a.link_type == b.link_type && a.id == b.id);

    // Output
    match cli.format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&entries)?);
        }
        OutputFormat::Human => {
            if entries.is_empty() {
                if !cli.quiet {
                    println!("No links found for {}", note_id);
                }
            } else {
                for entry in &entries {
                    let dir_arrow = match entry.direction.as_str() {
                        "out" => "->",
                        "in" => "<-",
                        _ => "--",
                    };
                    let title_part = entry
                        .title
                        .as_ref()
                        .map(|t| format!(" \"{}\"", t))
                        .unwrap_or_default();
                    println!(
                        "{} {} {} [{}] ({})",
                        dir_arrow, entry.id, title_part, entry.link_type, entry.source
                    );
                }
            }
        }
        OutputFormat::Records => {
            // Header line per spec (specs/records-output.md)
            println!(
                "H qipu=1 records=1 store={} mode=link.list id={} direction={}",
                store.root().display(),
                note_id,
                match direction {
                    Direction::Out => "out",
                    Direction::In => "in",
                    Direction::Both => "both",
                }
            );
            // Edge lines
            for entry in &entries {
                // E <from> <type> <to> <source>
                // For consistency, always show from -> to even for inbound
                let (from, to) = match entry.direction.as_str() {
                    "out" => (note_id.clone(), entry.id.clone()),
                    "in" => (entry.id.clone(), note_id.clone()),
                    _ => (note_id.clone(), entry.id.clone()),
                };
                println!("E {} {} {} {}", from, entry.link_type, to, entry.source);
            }
        }
    }

    Ok(())
}

/// Execute the link add command
///
/// Adds a typed link from one note to another.
pub fn execute_add(
    cli: &Cli,
    store: &Store,
    from_id: &str,
    to_id: &str,
    link_type: LinkType,
) -> Result<()> {
    // Resolve note IDs
    let from_resolved = resolve_note_id(store, from_id)?;
    let to_resolved = resolve_note_id(store, to_id)?;

    // Load and verify both notes exist
    let mut from_note = store.get_note(&from_resolved)?;
    let _to_note = store.get_note(&to_resolved)?;

    // Check if link already exists
    let link_exists = from_note
        .frontmatter
        .links
        .iter()
        .any(|l| l.id == to_resolved && l.link_type == link_type);

    if link_exists {
        if !cli.quiet {
            match cli.format {
                OutputFormat::Json => {
                    println!(
                        "{}",
                        serde_json::json!({
                            "status": "unchanged",
                            "from": from_resolved,
                            "to": to_resolved,
                            "type": link_type.to_string(),
                            "message": "link already exists"
                        })
                    );
                }
                OutputFormat::Human => {
                    println!(
                        "Link {} --[{}]--> {} already exists",
                        from_resolved, link_type, to_resolved
                    );
                }
                OutputFormat::Records => {
                    println!(
                        "H qipu=1 records=1 store={} mode=link.add status=unchanged",
                        store.root().display()
                    );
                    println!("E {} {} {} typed", from_resolved, link_type, to_resolved);
                }
            }
        }
        return Ok(());
    }

    // Add the link
    from_note.frontmatter.links.push(TypedLink {
        link_type,
        id: to_resolved.clone(),
    });

    // Save the note
    store.save_note(&mut from_note)?;

    // Output
    match cli.format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "status": "added",
                    "from": from_resolved,
                    "to": to_resolved,
                    "type": link_type.to_string()
                })
            );
        }
        OutputFormat::Human => {
            if !cli.quiet {
                println!(
                    "Added link: {} --[{}]--> {}",
                    from_resolved, link_type, to_resolved
                );
            }
        }
        OutputFormat::Records => {
            println!(
                "H qipu=1 records=1 store={} mode=link.add status=added",
                store.root().display()
            );
            println!("E {} {} {} typed", from_resolved, link_type, to_resolved);
        }
    }

    Ok(())
}

/// Execute the link remove command
///
/// Removes a typed link from one note to another.
pub fn execute_remove(
    cli: &Cli,
    store: &Store,
    from_id: &str,
    to_id: &str,
    link_type: LinkType,
) -> Result<()> {
    // Resolve note IDs
    let from_resolved = resolve_note_id(store, from_id)?;
    let to_resolved = resolve_note_id(store, to_id)?;

    // Load the source note
    let mut from_note = store.get_note(&from_resolved)?;

    // Find and remove the link
    let original_len = from_note.frontmatter.links.len();
    from_note
        .frontmatter
        .links
        .retain(|l| !(l.id == to_resolved && l.link_type == link_type));

    if from_note.frontmatter.links.len() == original_len {
        // Link didn't exist
        if !cli.quiet {
            match cli.format {
                OutputFormat::Json => {
                    println!(
                        "{}",
                        serde_json::json!({
                            "status": "not_found",
                            "from": from_resolved,
                            "to": to_resolved,
                            "type": link_type.to_string(),
                            "message": "link does not exist"
                        })
                    );
                }
                OutputFormat::Human => {
                    println!(
                        "Link {} --[{}]--> {} does not exist",
                        from_resolved, link_type, to_resolved
                    );
                }
                OutputFormat::Records => {
                    println!(
                        "H qipu=1 records=1 store={} mode=link.remove status=not_found",
                        store.root().display()
                    );
                }
            }
        }
        return Ok(());
    }

    // Save the note
    store.save_note(&mut from_note)?;

    // Output
    match cli.format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "status": "removed",
                    "from": from_resolved,
                    "to": to_resolved,
                    "type": link_type.to_string()
                })
            );
        }
        OutputFormat::Human => {
            if !cli.quiet {
                println!(
                    "Removed link: {} --[{}]--> {}",
                    from_resolved, link_type, to_resolved
                );
            }
        }
        OutputFormat::Records => {
            println!(
                "H qipu=1 records=1 store={} mode=link.remove status=removed",
                store.root().display()
            );
            println!("E {} {} {} typed", from_resolved, link_type, to_resolved);
        }
    }

    Ok(())
}

/// Resolve a note ID or path to a canonical note ID
fn resolve_note_id(store: &Store, id_or_path: &str) -> Result<String> {
    // If it looks like an ID (starts with qp-), try to use it directly
    if id_or_path.starts_with("qp-") {
        // Could be a full filename like qp-xxxx-slug.md or just qp-xxxx
        let id = id_or_path
            .trim_end_matches(".md")
            .split('-')
            .take(2)
            .collect::<Vec<_>>()
            .join("-");
        return Ok(id);
    }

    // Otherwise, try to find a note by path
    let notes = store.list_notes()?;
    for note in notes {
        if let Some(path) = &note.path {
            let path_str = path.display().to_string();
            if path_str.contains(id_or_path) || path_str.ends_with(id_or_path) {
                return Ok(note.id().to_string());
            }
        }
    }

    Err(crate::lib::error::QipuError::NoteNotFound {
        id: id_or_path.to_string(),
    })
}

/// Filter and convert an outbound edge to a LinkEntry
fn filter_and_convert(
    edge: &Edge,
    direction: &str,
    index: &Index,
    type_filter: Option<&str>,
    typed_only: bool,
    inline_only: bool,
) -> Option<LinkEntry> {
    // Apply source filters
    if typed_only && edge.source != LinkSource::Typed {
        return None;
    }
    if inline_only && edge.source != LinkSource::Inline {
        return None;
    }

    // Apply type filter
    if let Some(t) = type_filter {
        if edge.link_type != t {
            return None;
        }
    }

    // Get target note title if available
    let title = index.get_metadata(&edge.to).map(|m| m.title.clone());

    Some(LinkEntry {
        direction: direction.to_string(),
        id: edge.to.clone(),
        title,
        link_type: edge.link_type.clone(),
        source: edge.source.to_string(),
    })
}

/// Filter and convert an inbound edge to a LinkEntry
fn filter_and_convert_inbound(
    edge: &Edge,
    index: &Index,
    type_filter: Option<&str>,
    typed_only: bool,
    inline_only: bool,
) -> Option<LinkEntry> {
    // Apply source filters
    if typed_only && edge.source != LinkSource::Typed {
        return None;
    }
    if inline_only && edge.source != LinkSource::Inline {
        return None;
    }

    // Apply type filter
    if let Some(t) = type_filter {
        if edge.link_type != t {
            return None;
        }
    }

    // Get source note title if available
    let title = index.get_metadata(&edge.from).map(|m| m.title.clone());

    Some(LinkEntry {
        direction: "in".to_string(),
        id: edge.from.clone(),
        title,
        link_type: edge.link_type.clone(),
        source: edge.source.to_string(),
    })
}

/// Options for tree traversal
#[derive(Debug, Clone)]
pub struct TreeOptions {
    /// Direction for traversal
    pub direction: Direction,
    /// Maximum traversal depth
    pub max_hops: u32,
    /// Include only these link types (empty = all)
    pub type_include: Vec<String>,
    /// Exclude these link types
    pub type_exclude: Vec<String>,
    /// Show only typed links
    pub typed_only: bool,
    /// Show only inline links
    pub inline_only: bool,
    /// Maximum nodes to visit
    pub max_nodes: Option<usize>,
    /// Maximum edges to emit
    pub max_edges: Option<usize>,
    /// Maximum neighbors per node
    pub max_fanout: Option<usize>,
    /// Maximum output characters (records format only)
    pub max_chars: Option<usize>,
}

impl Default for TreeOptions {
    fn default() -> Self {
        TreeOptions {
            direction: Direction::Both,
            max_hops: 3,
            type_include: Vec::new(),
            type_exclude: Vec::new(),
            typed_only: false,
            inline_only: false,
            max_nodes: None,
            max_edges: None,
            max_fanout: None,
            max_chars: None,
        }
    }
}

/// Node in the traversal output
#[derive(Debug, Clone, Serialize)]
pub struct TreeNode {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub note_type: NoteType,
    pub tags: Vec<String>,
    pub path: String,
}

/// Edge in the traversal output
#[derive(Debug, Clone, Serialize)]
pub struct TreeEdge {
    pub from: String,
    pub to: String,
    #[serde(rename = "type")]
    pub link_type: String,
    pub source: String,
}

/// Spanning tree entry
#[derive(Debug, Clone, Serialize)]
pub struct SpanningTreeEntry {
    pub from: String,
    pub to: String,
    pub hop: u32,
}

/// Complete traversal result
#[derive(Debug, Clone, Serialize)]
pub struct TreeResult {
    pub root: String,
    pub direction: String,
    pub max_hops: u32,
    pub truncated: bool,
    pub truncation_reason: Option<String>,
    pub nodes: Vec<TreeNode>,
    pub edges: Vec<TreeEdge>,
    pub spanning_tree: Vec<SpanningTreeEntry>,
}

/// Execute the link tree command
pub fn execute_tree(cli: &Cli, store: &Store, id_or_path: &str, opts: TreeOptions) -> Result<()> {
    // Resolve the note ID
    let note_id = resolve_note_id(store, id_or_path)?;

    // Load or build the index
    let index = IndexBuilder::new(store).load_existing()?.build()?;

    // Build compaction context if needed
    let compaction_ctx = if !cli.no_resolve_compaction {
        let notes = store.list_notes()?;
        Some(CompactionContext::build(&notes)?)
    } else {
        None
    };

    // Canonicalize the root note ID
    let canonical_id = if let Some(ref ctx) = compaction_ctx {
        ctx.canon(&note_id)?
    } else {
        note_id.clone()
    };

    // Verify note exists (check canonical ID)
    if !index.contains(&canonical_id) {
        return Err(crate::lib::error::QipuError::NoteNotFound {
            id: canonical_id.clone(),
        });
    }

    // Perform BFS traversal with compaction context
    let result = bfs_traverse(&index, &canonical_id, &opts, compaction_ctx.as_ref())?;

    // Output
    match cli.format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        OutputFormat::Human => {
            output_tree_human(cli, &result, &index);
        }
        OutputFormat::Records => {
            output_tree_records(&result, store, &opts);
        }
    }

    Ok(())
}

/// Perform BFS traversal from a root node with optional compaction resolution
fn bfs_traverse(
    index: &Index,
    root: &str,
    opts: &TreeOptions,
    compaction_ctx: Option<&CompactionContext>,
) -> Result<TreeResult> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(String, u32)> = VecDeque::new();
    let mut nodes: Vec<TreeNode> = Vec::new();
    let mut edges: Vec<TreeEdge> = Vec::new();
    let mut spanning_tree: Vec<SpanningTreeEntry> = Vec::new();

    let mut truncated = false;
    let mut truncation_reason: Option<String> = None;

    // Initialize with root
    queue.push_back((root.to_string(), 0));
    visited.insert(root.to_string());

    // Add root node
    if let Some(meta) = index.get_metadata(root) {
        nodes.push(TreeNode {
            id: meta.id.clone(),
            title: meta.title.clone(),
            note_type: meta.note_type,
            tags: meta.tags.clone(),
            path: meta.path.clone(),
        });
    }

    while let Some((current_id, hop)) = queue.pop_front() {
        // Check max_nodes limit
        if let Some(max) = opts.max_nodes {
            if visited.len() >= max {
                truncated = true;
                truncation_reason = Some("max_nodes".to_string());
                break;
            }
        }

        // Check max_edges limit
        if let Some(max) = opts.max_edges {
            if edges.len() >= max {
                truncated = true;
                truncation_reason = Some("max_edges".to_string());
                break;
            }
        }

        // Don't expand beyond max_hops
        if hop >= opts.max_hops {
            continue;
        }

        // Get neighbors based on direction (gather edges from all compacted notes)
        let neighbors = get_filtered_neighbors(index, &current_id, opts, compaction_ctx);

        // Apply max_fanout
        let neighbors: Vec<_> = if let Some(max_fanout) = opts.max_fanout {
            if neighbors.len() > max_fanout {
                truncated = true;
                truncation_reason = Some("max_fanout".to_string());
            }
            neighbors.into_iter().take(max_fanout).collect()
        } else {
            neighbors
        };

        for (neighbor_id, edge) in neighbors {
            // Canonicalize edge endpoints if using compaction
            let canonical_from = if let Some(ctx) = compaction_ctx {
                ctx.canon(&edge.from)?
            } else {
                edge.from.clone()
            };
            let canonical_to = if let Some(ctx) = compaction_ctx {
                ctx.canon(&edge.to)?
            } else {
                edge.to.clone()
            };

            // Skip self-loops introduced by compaction contraction
            if canonical_from == canonical_to {
                continue;
            }

            // Canonicalize the neighbor ID
            let canonical_neighbor = if let Some(ctx) = compaction_ctx {
                ctx.canon(&neighbor_id)?
            } else {
                neighbor_id.clone()
            };

            // Check max_edges again before adding
            if let Some(max) = opts.max_edges {
                if edges.len() >= max {
                    truncated = true;
                    truncation_reason = Some("max_edges".to_string());
                    break;
                }
            }

            // Add edge with canonical IDs
            edges.push(TreeEdge {
                from: canonical_from,
                to: canonical_to,
                link_type: edge.link_type.clone(),
                source: edge.source.to_string(),
            });

            // Process neighbor if not visited (use canonical ID)
            if !visited.contains(&canonical_neighbor) {
                // Check max_nodes before adding
                if let Some(max) = opts.max_nodes {
                    if visited.len() >= max {
                        truncated = true;
                        truncation_reason = Some("max_nodes".to_string());
                        break;
                    }
                }

                visited.insert(canonical_neighbor.clone());

                // Add to spanning tree (first discovery, use canonical IDs)
                spanning_tree.push(SpanningTreeEntry {
                    from: current_id.clone(),
                    to: canonical_neighbor.clone(),
                    hop: hop + 1,
                });

                // Add node metadata (use canonical ID)
                if let Some(meta) = index.get_metadata(&canonical_neighbor) {
                    nodes.push(TreeNode {
                        id: meta.id.clone(),
                        title: meta.title.clone(),
                        note_type: meta.note_type,
                        tags: meta.tags.clone(),
                        path: meta.path.clone(),
                    });
                }

                // Queue for further expansion (use canonical ID)
                queue.push_back((canonical_neighbor, hop + 1));
            }
        }
    }

    // Sort for determinism
    nodes.sort_by(|a, b| a.id.cmp(&b.id));
    edges.sort_by(|a, b| {
        a.from
            .cmp(&b.from)
            .then_with(|| a.link_type.cmp(&b.link_type))
            .then_with(|| a.to.cmp(&b.to))
    });
    spanning_tree.sort_by(|a, b| a.hop.cmp(&b.hop).then_with(|| a.to.cmp(&b.to)));

    Ok(TreeResult {
        root: root.to_string(),
        direction: match opts.direction {
            Direction::Out => "out".to_string(),
            Direction::In => "in".to_string(),
            Direction::Both => "both".to_string(),
        },
        max_hops: opts.max_hops,
        truncated,
        truncation_reason,
        nodes,
        edges,
        spanning_tree,
    })
}

/// Get filtered neighbors for a node
fn get_filtered_neighbors<'a>(
    index: &'a Index,
    id: &str,
    opts: &TreeOptions,
    compaction_ctx: Option<&CompactionContext>,
) -> Vec<(String, &'a Edge)> {
    let mut neighbors: Vec<(String, &Edge)> = Vec::new();

    // Collect all source IDs that map to this ID (for gathering edges)
    // This includes the ID itself plus any notes compacted by this ID
    let mut source_ids = vec![id.to_string()];
    if let Some(ctx) = compaction_ctx {
        // Find all notes that are compacted by this ID
        if let Some(compacted_notes) = ctx.get_compacted_notes(id) {
            source_ids.extend(compacted_notes.iter().cloned());
        }
    }

    // Get outbound edges from ALL source IDs
    if opts.direction == Direction::Out || opts.direction == Direction::Both {
        for source_id in &source_ids {
            for edge in index.get_outbound_edges(source_id) {
                if filter_edge(edge, opts) {
                    neighbors.push((edge.to.clone(), edge));
                }
            }
        }
    }

    // Get inbound edges to ALL source IDs (backlinks)
    if opts.direction == Direction::In || opts.direction == Direction::Both {
        for source_id in &source_ids {
            for edge in index.get_inbound_edges(source_id) {
                if filter_edge(edge, opts) {
                    neighbors.push((edge.from.clone(), edge));
                }
            }
        }
    }

    // Sort for determinism: edge type, then target id
    neighbors.sort_by(|a, b| {
        a.1.link_type
            .cmp(&b.1.link_type)
            .then_with(|| a.0.cmp(&b.0))
    });

    neighbors
}

/// Check if an edge passes the filters
fn filter_edge(edge: &Edge, opts: &TreeOptions) -> bool {
    // Source filter
    if opts.typed_only && edge.source != LinkSource::Typed {
        return false;
    }
    if opts.inline_only && edge.source != LinkSource::Inline {
        return false;
    }

    // Type inclusion filter
    if !opts.type_include.is_empty() && !opts.type_include.contains(&edge.link_type) {
        return false;
    }

    // Type exclusion filter
    if opts.type_exclude.contains(&edge.link_type) {
        return false;
    }

    true
}

/// Output tree in human-readable format
fn output_tree_human(cli: &Cli, result: &TreeResult, index: &Index) {
    if result.nodes.is_empty() {
        if !cli.quiet {
            println!("No nodes found");
        }
        return;
    }

    // Build tree structure for pretty printing
    let mut children: HashMap<String, Vec<&SpanningTreeEntry>> = HashMap::new();
    for entry in &result.spanning_tree {
        children.entry(entry.from.clone()).or_default().push(entry);
    }

    // Print tree recursively
    fn print_tree(
        id: &str,
        children: &HashMap<String, Vec<&SpanningTreeEntry>>,
        index: &Index,
        visited: &HashSet<String>,
        prefix: &str,
        is_last: bool,
    ) {
        let title = index
            .get_metadata(id)
            .map(|m| m.title.as_str())
            .unwrap_or("(unknown)");

        let connector = if prefix.is_empty() {
            ""
        } else if is_last {
            "└── "
        } else {
            "├── "
        };

        println!("{}{}{} \"{}\"", prefix, connector, id, title);

        if let Some(kids) = children.get(id) {
            let new_prefix = if prefix.is_empty() {
                "".to_string()
            } else if is_last {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };

            for (i, entry) in kids.iter().enumerate() {
                let is_last_child = i == kids.len() - 1;
                if visited.contains(&entry.to) {
                    // Already seen - mark but don't recurse
                    let connector = if is_last_child {
                        "└── "
                    } else {
                        "├── "
                    };
                    let child_title = index
                        .get_metadata(&entry.to)
                        .map(|m| m.title.as_str())
                        .unwrap_or("(unknown)");
                    println!(
                        "{}{}{} \"{}\" (seen)",
                        new_prefix, connector, entry.to, child_title
                    );
                } else {
                    let mut new_visited = visited.clone();
                    new_visited.insert(entry.to.clone());
                    print_tree(
                        &entry.to,
                        children,
                        index,
                        &new_visited,
                        &new_prefix,
                        is_last_child,
                    );
                }
            }
        }
    }

    let mut initial_visited = HashSet::new();
    initial_visited.insert(result.root.clone());
    print_tree(&result.root, &children, index, &initial_visited, "", true);

    if result.truncated {
        println!();
        println!(
            "[truncated: {}]",
            result
                .truncation_reason
                .as_deref()
                .unwrap_or("limit reached")
        );
    }
}

/// Output tree in records format
fn output_tree_records(result: &TreeResult, store: &Store, opts: &TreeOptions) {
    let mut budget_truncated = false;
    let budget = opts.max_chars;

    // Collect output lines with budget tracking
    let mut node_lines = Vec::new();
    let mut edge_lines = Vec::new();
    let mut used_chars = 0;

    // Estimate header size (we'll generate it later with correct truncated flag)
    let header_estimate = 200; // Conservative estimate
    used_chars += header_estimate;

    // Generate node lines (including summaries)
    for node in &result.nodes {
        let tags_csv = if node.tags.is_empty() {
            "-".to_string()
        } else {
            node.tags.join(",")
        };
        let node_line = format!(
            "N {} {} \"{}\" tags={}",
            node.id, node.note_type, node.title, tags_csv
        );

        let mut summary_line = None;
        // Load note to get summary (per spec: prefer summaries over full bodies)
        if let Ok(note) = store.get_note(&node.id) {
            let summary = note.summary();
            if !summary.is_empty() {
                // Truncate summary to single line
                let summary_text = summary.lines().next().unwrap_or("").trim();
                if !summary_text.is_empty() {
                    summary_line = Some(format!("S {} {}", node.id, summary_text));
                }
            }
        }

        // Check budget before adding (10% safety buffer)
        let node_size = node_line.len() + 1 + summary_line.as_ref().map_or(0, |s| s.len() + 1);
        let node_size_with_buffer = node_size + (node_size / 10);

        if let Some(max) = budget {
            if used_chars + node_size_with_buffer > max {
                budget_truncated = true;
                break;
            }
        }

        node_lines.push((node_line, summary_line));
        used_chars += node_size;
    }

    // Generate edge lines
    if !budget_truncated {
        for edge in &result.edges {
            let edge_line = format!(
                "E {} {} {} {}",
                edge.from, edge.link_type, edge.to, edge.source
            );

            let edge_size = edge_line.len() + 1;
            let edge_size_with_buffer = edge_size + (edge_size / 10);

            if let Some(max) = budget {
                if used_chars + edge_size_with_buffer > max {
                    budget_truncated = true;
                    break;
                }
            }

            edge_lines.push(edge_line);
            used_chars += edge_size;
        }
    }

    // Now generate header with correct truncated flag
    let truncated_str = if result.truncated || budget_truncated {
        "true"
    } else {
        "false"
    };
    println!(
        "H qipu=1 records=1 store={} mode=link.tree root={} direction={} max_hops={} truncated={}",
        store.root().display(),
        result.root,
        result.direction,
        result.max_hops,
        truncated_str
    );

    // Output collected lines
    for (node_line, summary_line) in node_lines {
        println!("{}", node_line);
        if let Some(s) = summary_line {
            println!("{}", s);
        }
    }

    for edge_line in edge_lines {
        println!("{}", edge_line);
    }
}

/// Path result
#[derive(Debug, Clone, Serialize)]
pub struct PathResult {
    pub from: String,
    pub to: String,
    pub direction: String,
    pub found: bool,
    pub nodes: Vec<TreeNode>,
    pub edges: Vec<TreeEdge>,
    pub path_length: usize,
}

/// Execute the link path command
pub fn execute_path(
    cli: &Cli,
    store: &Store,
    from_id: &str,
    to_id: &str,
    opts: TreeOptions,
) -> Result<()> {
    // Resolve note IDs
    let from_resolved = resolve_note_id(store, from_id)?;
    let to_resolved = resolve_note_id(store, to_id)?;

    // Load or build the index
    let index = IndexBuilder::new(store).load_existing()?.build()?;

    // Build compaction context if needed
    let compaction_ctx = if !cli.no_resolve_compaction {
        let notes = store.list_notes()?;
        Some(CompactionContext::build(&notes)?)
    } else {
        None
    };

    // Canonicalize the note IDs
    let canonical_from = if let Some(ref ctx) = compaction_ctx {
        ctx.canon(&from_resolved)?
    } else {
        from_resolved.clone()
    };
    let canonical_to = if let Some(ref ctx) = compaction_ctx {
        ctx.canon(&to_resolved)?
    } else {
        to_resolved.clone()
    };

    // Verify both notes exist (check canonical IDs)
    if !index.contains(&canonical_from) {
        return Err(crate::lib::error::QipuError::NoteNotFound {
            id: canonical_from.clone(),
        });
    }
    if !index.contains(&canonical_to) {
        return Err(crate::lib::error::QipuError::NoteNotFound {
            id: canonical_to.clone(),
        });
    }

    // Find path using BFS with compaction context
    let result = bfs_find_path(
        &index,
        &canonical_from,
        &canonical_to,
        &opts,
        compaction_ctx.as_ref(),
    )?;

    // Output
    match cli.format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        OutputFormat::Human => {
            output_path_human(cli, &result);
        }
        OutputFormat::Records => {
            output_path_records(&result, store, &opts);
        }
    }

    Ok(())
}

/// Find path between two nodes using BFS with optional compaction resolution
fn bfs_find_path(
    index: &Index,
    from: &str,
    to: &str,
    opts: &TreeOptions,
    compaction_ctx: Option<&CompactionContext>,
) -> Result<PathResult> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(String, u32)> = VecDeque::new();
    let mut predecessors: HashMap<String, (String, Edge)> = HashMap::new();

    // Initialize
    queue.push_back((from.to_string(), 0));
    visited.insert(from.to_string());

    let mut found = false;

    while let Some((current_id, hop)) = queue.pop_front() {
        // Check if we found the target
        if current_id == to {
            found = true;
            break;
        }

        // Don't expand beyond max_hops
        if hop >= opts.max_hops {
            continue;
        }

        // Get neighbors (gather edges from all compacted notes)
        let neighbors = get_filtered_neighbors(index, &current_id, opts, compaction_ctx);

        for (neighbor_id, edge) in neighbors {
            // Canonicalize edge endpoints if using compaction
            let canonical_from = if let Some(ctx) = compaction_ctx {
                ctx.canon(&edge.from)?
            } else {
                edge.from.clone()
            };
            let canonical_to = if let Some(ctx) = compaction_ctx {
                ctx.canon(&edge.to)?
            } else {
                edge.to.clone()
            };

            // Skip self-loops introduced by compaction contraction
            if canonical_from == canonical_to {
                continue;
            }

            // Canonicalize the neighbor ID
            let canonical_neighbor = if let Some(ctx) = compaction_ctx {
                ctx.canon(&neighbor_id)?
            } else {
                neighbor_id.clone()
            };

            if !visited.contains(&canonical_neighbor) {
                visited.insert(canonical_neighbor.clone());
                // Store canonical edge
                let canonical_edge = Edge {
                    from: canonical_from,
                    to: canonical_to,
                    link_type: edge.link_type.clone(),
                    source: edge.source,
                };
                predecessors.insert(
                    canonical_neighbor.clone(),
                    (current_id.clone(), canonical_edge),
                );
                queue.push_back((canonical_neighbor, hop + 1));
            }
        }
    }

    // Reconstruct path if found
    let (nodes, edges) = if found {
        let mut path_nodes: Vec<String> = Vec::new();
        let mut path_edges: Vec<TreeEdge> = Vec::new();

        // Backtrack from target to source
        let mut current = to.to_string();
        path_nodes.push(current.clone());

        while current != from {
            if let Some((pred, edge)) = predecessors.get(&current) {
                path_edges.push(TreeEdge {
                    from: edge.from.clone(),
                    to: edge.to.clone(),
                    link_type: edge.link_type.clone(),
                    source: edge.source.to_string(),
                });
                current = pred.clone();
                path_nodes.push(current.clone());
            } else {
                break;
            }
        }

        path_nodes.reverse();
        path_edges.reverse();

        // Convert to TreeNodes
        let tree_nodes: Vec<TreeNode> = path_nodes
            .iter()
            .filter_map(|id| {
                index.get_metadata(id).map(|meta| TreeNode {
                    id: meta.id.clone(),
                    title: meta.title.clone(),
                    note_type: meta.note_type,
                    tags: meta.tags.clone(),
                    path: meta.path.clone(),
                })
            })
            .collect();

        (tree_nodes, path_edges)
    } else {
        (Vec::new(), Vec::new())
    };

    Ok(PathResult {
        from: from.to_string(),
        to: to.to_string(),
        direction: match opts.direction {
            Direction::Out => "out".to_string(),
            Direction::In => "in".to_string(),
            Direction::Both => "both".to_string(),
        },
        found,
        path_length: edges.len(),
        nodes,
        edges,
    })
}

/// Output path in human-readable format
fn output_path_human(cli: &Cli, result: &PathResult) {
    if !result.found {
        if !cli.quiet {
            println!("No path found from {} to {}", result.from, result.to);
        }
        return;
    }

    // Print path: node -> node -> node
    for (i, node) in result.nodes.iter().enumerate() {
        if i > 0 {
            // Print edge info
            if let Some(edge) = result.edges.get(i - 1) {
                println!("  |");
                println!("  | [{}] ({})", edge.link_type, edge.source);
                println!("  v");
            }
        }
        println!("{} \"{}\"", node.id, node.title);
    }

    println!();
    println!("Path length: {} hop(s)", result.path_length);
}

/// Output path in records format
fn output_path_records(result: &PathResult, store: &Store, opts: &TreeOptions) {
    let mut budget_truncated = false;
    let budget = opts.max_chars;

    // Collect output lines with budget tracking (only if path was found)
    let mut node_lines = Vec::new();
    let mut edge_lines = Vec::new();
    let mut used_chars = 0;

    // Estimate header size
    let header_estimate = 250; // Conservative estimate
    used_chars += header_estimate;

    if result.found {
        // Generate node lines (including summaries)
        for node in &result.nodes {
            let tags_csv = if node.tags.is_empty() {
                "-".to_string()
            } else {
                node.tags.join(",")
            };
            let node_line = format!(
                "N {} {} \"{}\" tags={}",
                node.id, node.note_type, node.title, tags_csv
            );

            let mut summary_line = None;
            // Load note to get summary (per spec: prefer summaries over full bodies)
            if let Ok(note) = store.get_note(&node.id) {
                let summary = note.summary();
                if !summary.is_empty() {
                    // Truncate summary to single line
                    let summary_text = summary.lines().next().unwrap_or("").trim();
                    if !summary_text.is_empty() {
                        summary_line = Some(format!("S {} {}", node.id, summary_text));
                    }
                }
            }

            // Check budget before adding (10% safety buffer)
            let node_size = node_line.len() + 1 + summary_line.as_ref().map_or(0, |s| s.len() + 1);
            let node_size_with_buffer = node_size + (node_size / 10);

            if let Some(max) = budget {
                if used_chars + node_size_with_buffer > max {
                    budget_truncated = true;
                    break;
                }
            }

            node_lines.push((node_line, summary_line));
            used_chars += node_size;
        }

        // Generate edge lines
        if !budget_truncated {
            for edge in &result.edges {
                let edge_line = format!(
                    "E {} {} {} {}",
                    edge.from, edge.link_type, edge.to, edge.source
                );

                let edge_size = edge_line.len() + 1;
                let edge_size_with_buffer = edge_size + (edge_size / 10);

                if let Some(max) = budget {
                    if used_chars + edge_size_with_buffer > max {
                        budget_truncated = true;
                        break;
                    }
                }

                edge_lines.push(edge_line);
                used_chars += edge_size;
            }
        }
    }

    // Generate header (budget truncation doesn't apply to path - paths are small)
    // But we include the logic for consistency
    let found_str = if result.found { "true" } else { "false" };
    let truncated_str = if budget_truncated { "true" } else { "false" };
    println!(
        "H qipu=1 records=1 store={} mode=link.path from={} to={} direction={} found={} length={} truncated={}",
        store.root().display(),
        result.from,
        result.to,
        result.direction,
        found_str,
        result.path_length,
        truncated_str
    );

    // Output collected lines
    for (node_line, summary_line) in node_lines {
        println!("{}", node_line);
        if let Some(s) = summary_line {
            println!("{}", s);
        }
    }

    for edge_line in edge_lines {
        println!("{}", edge_line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_parsing() {
        assert_eq!("out".parse::<Direction>().unwrap(), Direction::Out);
        assert_eq!("in".parse::<Direction>().unwrap(), Direction::In);
        assert_eq!("both".parse::<Direction>().unwrap(), Direction::Both);
        assert_eq!("OUT".parse::<Direction>().unwrap(), Direction::Out);
    }

    #[test]
    fn test_direction_parsing_invalid() {
        assert!("invalid".parse::<Direction>().is_err());
    }

    #[test]
    fn test_tree_options_default() {
        let opts = TreeOptions::default();
        assert_eq!(opts.direction, Direction::Both);
        assert_eq!(opts.max_hops, 3);
        assert!(opts.type_include.is_empty());
        assert!(opts.type_exclude.is_empty());
        assert!(!opts.typed_only);
        assert!(!opts.inline_only);
        assert!(opts.max_nodes.is_none());
        assert!(opts.max_edges.is_none());
        assert!(opts.max_fanout.is_none());
    }
}
