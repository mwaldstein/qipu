//! Data operations command argument structures

use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct ContextArgs {
    /// Walk the graph from a note and bundle traversed notes
    #[arg(long)]
    pub walk: Option<String>,

    /// Direction for graph walk (out, in, or both)
    #[arg(long, default_value = "both")]
    pub walk_direction: String,

    /// Maximum traversal depth for graph walk
    #[arg(long, default_value = "3")]
    pub walk_max_hops: u32,

    /// Include only these link types in graph walk (can be repeated, or use CSV)
    #[arg(long, short = 'T', action = clap::ArgAction::Append, value_delimiter = ',')]
    pub walk_type: Vec<String>,

    /// Exclude these link types in graph walk (can be repeated, or use CSV)
    #[arg(long, action = clap::ArgAction::Append, value_delimiter = ',')]
    pub walk_exclude_type: Vec<String>,

    /// Show only typed links in graph walk (from frontmatter)
    #[arg(long)]
    pub walk_typed_only: bool,

    /// Show only inline links in graph walk (from markdown body)
    #[arg(long)]
    pub walk_inline_only: bool,

    /// Maximum nodes to visit in graph walk
    #[arg(long)]
    pub walk_max_nodes: Option<usize>,

    /// Maximum edges to emit in graph walk
    #[arg(long)]
    pub walk_max_edges: Option<usize>,

    /// Maximum neighbors per node in graph walk
    #[arg(long)]
    pub walk_max_fanout: Option<usize>,

    /// Filter by minimum value in graph walk (0-100, default: 50)
    #[arg(long, value_parser = crate::cli::parse::parse_min_value)]
    pub walk_min_value: Option<u8>,

    /// Ignore note values during graph walk (unweighted BFS, weighted by default)
    #[arg(long, alias = "walk-unweighted", default_value = "false")]
    pub walk_ignore_value: bool,

    /// Select notes by ID (can be repeated)
    #[arg(long, short = 'n', action = clap::ArgAction::Append)]
    pub note: Vec<String>,

    /// Select notes by tag
    #[arg(long)]
    pub tag: Option<String>,

    /// Select notes linked from a MOC
    #[arg(long, short = 'm')]
    pub moc: Option<String>,

    /// Select notes by search query
    #[arg(long)]
    pub query: Option<String>,

    /// Maximum output characters (exact budget)
    #[arg(long)]
    pub max_chars: Option<usize>,

    /// Follow nested MOC links transitively
    #[arg(long)]
    pub transitive: bool,

    /// Include full note body content (default, use --summary-only to disable)
    #[arg(long, hide = true)]
    pub with_body: bool,

    /// Use summary instead of full body content
    #[arg(long)]
    pub summary_only: bool,

    /// Include safety banner for LLM prompt injection prevention
    #[arg(long)]
    pub safety_banner: bool,

    /// Add related notes using similarity expansion (threshold: 0.0-1.0, default: 0.3)
    ///
    /// Set to 0 to disable related-note expansion
    #[arg(long, default_value = "0.3")]
    pub related: f64,

    /// Include backlinks for selected notes
    #[arg(long)]
    pub backlinks: bool,

    /// Select notes by minimum value (0-100, can be used as standalone selector)
    #[arg(long, value_parser = crate::cli::parse::parse_min_value, value_name = "N")]
    pub min_value: Option<u8>,

    /// Select notes by custom metadata (format: key=value, key, !key, key>n, key>=n, key<n, key<=n, can be used as standalone selector, can be repeated)
    #[arg(long, action = clap::ArgAction::Append)]
    pub custom_filter: Vec<String>,

    /// Include custom metadata in output (opt-in)
    #[arg(long)]
    pub custom: bool,

    /// Include domain guidance and type descriptions (ontology) in output
    #[arg(long)]
    pub include_ontology: bool,
}

#[derive(Args, Debug)]
pub struct DumpArgs {
    /// Output pack file path
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,

    /// Select notes by ID (can be repeated)
    #[arg(long, short = 'n', action = clap::ArgAction::Append)]
    pub note: Vec<String>,

    /// Select notes by tag
    #[arg(long)]
    pub tag: Option<String>,

    /// Select notes linked from a MOC
    #[arg(long, short = 'm')]
    pub moc: Option<String>,

    /// Select notes by search query
    #[arg(long)]
    pub query: Option<String>,

    /// Traversal direction
    #[arg(long, default_value = "both")]
    pub direction: String,

    /// Maximum traversal depth
    #[arg(long, default_value = "3")]
    pub max_hops: u32,

    /// Include only these link types (can be repeated)
    #[arg(long, short = 'T', action = clap::ArgAction::Append)]
    pub r#type: Vec<String>,

    /// Show only typed links (from frontmatter)
    #[arg(long)]
    pub typed_only: bool,

    /// Show only inline links (from markdown body)
    #[arg(long)]
    pub inline_only: bool,

    /// Exclude attachments from pack
    #[arg(long)]
    pub no_attachments: bool,

    /// Output file path (default: stdout)
    #[arg(long, short = 'o')]
    pub output: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct ExportArgs {
    /// Select notes by ID (can be repeated)
    #[arg(long, short = 'n', action = clap::ArgAction::Append)]
    pub note: Vec<String>,

    /// Select notes by tag
    #[arg(long)]
    pub tag: Option<String>,

    /// Select notes linked from a MOC
    #[arg(long, short = 'm')]
    pub moc: Option<String>,

    /// Select notes by search query
    #[arg(long)]
    pub query: Option<String>,

    /// Output file path (default: stdout)
    #[arg(long, short = 'o')]
    pub output: Option<PathBuf>,

    /// Export mode: bundle, outline, bibliography
    #[arg(long, default_value = "bundle")]
    pub mode: String,

    /// Copy referenced attachments to the output directory
    #[arg(long)]
    pub with_attachments: bool,

    /// Link handling: preserve, markdown, anchors
    #[arg(long, default_value = "preserve")]
    pub link_mode: String,

    /// Bibliography format: markdown, bibtex, csl-json (only used with --mode bibliography)
    #[arg(long, default_value = "markdown")]
    pub bib_format: String,

    /// Expand selection by traversing links (0 = no expansion)
    #[arg(long, default_value = "0")]
    pub max_hops: u32,

    /// Convert output to PDF using pandoc (requires pandoc to be installed)
    #[arg(long)]
    pub pdf: bool,
}

#[derive(Args, Debug)]
pub struct LoadArgs {
    /// Pack file path
    pub pack_file: PathBuf,

    /// Conflict resolution strategy: skip, overwrite, merge-links
    #[arg(long, default_value = "skip")]
    pub strategy: String,

    /// Apply embedded config.toml from pack
    #[arg(long)]
    pub apply_config: bool,
}

#[derive(Args, Debug)]
pub struct PrimeArgs {
    /// Compact output (omit MOCs and recent notes)
    #[arg(long)]
    pub compact: bool,

    /// Minimal output (only ontology and commands)
    #[arg(long)]
    pub minimal: bool,

    /// Force full CLI output (ignore MCP detection)
    #[arg(long, conflicts_with = "mcp")]
    pub full: bool,

    /// Force minimal MCP output (~50 tokens)
    #[arg(long, conflicts_with = "full")]
    pub mcp: bool,
}
