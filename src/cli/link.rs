//! Link subcommands for managing relationships between notes

use super::parse::parse_link_type;
use clap::Subcommand;
use qipu_core::note::LinkType;

/// Link subcommands
#[derive(Subcommand, Debug)]
pub enum LinkCommands {
    /// List links for a note
    List {
        /// Note ID or file path
        id_or_path: String,

        /// Direction: out, in, or both
        #[arg(long, short, default_value = "both")]
        direction: String,

        /// Filter by link type (related, derived-from, supports, contradicts, part-of)
        #[arg(long, short = 'T')]
        r#type: Option<String>,

        /// Show only typed links (from frontmatter)
        #[arg(long)]
        typed_only: bool,

        /// Show only inline links (from markdown body)
        #[arg(long)]
        inline_only: bool,

        /// Maximum output characters (exact budget, records format only)
        #[arg(long)]
        max_chars: Option<usize>,
    },

    /// Add a typed link between notes
    Add {
        /// Source note ID
        from: String,

        /// Target note ID
        to: String,

        /// Link type (related, derived-from, supports, contradicts, part-of)
        #[arg(long, short = 'T', value_parser = parse_link_type, required = true)]
        r#type: LinkType,
    },

    /// Remove a typed link between notes
    Remove {
        /// Source note ID
        from: String,

        /// Target note ID
        to: String,

        /// Link type (related, derived-from, supports, contradicts, part-of)
        #[arg(long, short = 'T', value_parser = parse_link_type, required = true)]
        r#type: LinkType,
    },

    /// Show traversal tree from a note
    Tree {
        /// Note ID or file path
        id_or_path: String,

        /// Direction: out, in, or both
        #[arg(long, short, default_value = "both")]
        direction: String,

        /// Maximum traversal depth
        #[arg(long, default_value = "3")]
        max_hops: u32,

        /// Include only these link types (can be repeated, or use CSV with --types)
        #[arg(long, short = 'T', alias = "types", action = clap::ArgAction::Append, value_delimiter = ',')]
        r#type: Vec<String>,

        /// Exclude these link types (can be repeated, or use CSV with --exclude-types)
        #[arg(long, alias = "exclude-types", action = clap::ArgAction::Append, value_delimiter = ',')]
        exclude_type: Vec<String>,

        /// Show only typed links (from frontmatter)
        #[arg(long)]
        typed_only: bool,

        /// Show only inline links (from markdown body)
        #[arg(long)]
        inline_only: bool,

        /// Maximum nodes to visit
        #[arg(long)]
        max_nodes: Option<usize>,

        /// Maximum edges to emit
        #[arg(long)]
        max_edges: Option<usize>,

        /// Maximum neighbors per node
        #[arg(long)]
        max_fanout: Option<usize>,

        /// Maximum output characters (exact budget, records format only)
        #[arg(long)]
        max_chars: Option<usize>,

        /// Filter by minimum value (0-100, default: 50)
        #[arg(long, value_parser = crate::cli::parse::parse_min_value)]
        min_value: Option<u8>,

        /// Ignore note values during traversal (unweighted BFS, weighted by default)
        #[arg(long, alias = "unweighted", default_value = "false")]
        ignore_value: bool,
    },

    /// Find path between two notes
    Path {
        /// Starting note ID
        from: String,

        /// Target note ID
        to: String,

        /// Direction: out, in, or both
        #[arg(long, short, default_value = "both")]
        direction: String,

        /// Maximum path length
        #[arg(long, default_value = "10")]
        max_hops: u32,

        /// Include only these link types (can be repeated, or use CSV with --types)
        #[arg(long, short = 'T', alias = "types", action = clap::ArgAction::Append, value_delimiter = ',')]
        r#type: Vec<String>,

        /// Exclude these link types (can be repeated, or use CSV with --exclude-types)
        #[arg(long, alias = "exclude-types", action = clap::ArgAction::Append, value_delimiter = ',')]
        exclude_type: Vec<String>,

        /// Show only typed links (from frontmatter)
        #[arg(long)]
        typed_only: bool,

        /// Show only inline links (from markdown body)
        #[arg(long)]
        inline_only: bool,

        /// Maximum nodes to visit during path search
        #[arg(long)]
        max_nodes: Option<usize>,

        /// Maximum edges to consider during path search
        #[arg(long)]
        max_edges: Option<usize>,

        /// Maximum neighbors per node to consider
        #[arg(long)]
        max_fanout: Option<usize>,

        /// Maximum output characters (exact budget, records format only)
        #[arg(long)]
        max_chars: Option<usize>,

        /// Filter by minimum value (0-100, default: 50)
        #[arg(long, value_parser = crate::cli::parse::parse_min_value)]
        min_value: Option<u8>,

        /// Ignore note values during path finding (unweighted BFS, weighted by default)
        #[arg(long, alias = "unweighted", default_value = "false")]
        ignore_value: bool,
    },

    /// Materialize inline links into typed frontmatter links
    Materialize {
        /// Note ID or file path
        id_or_path: String,

        /// Link type for materialized links (default: related)
        #[arg(long, short = 'T', value_parser = parse_link_type, default_value = "related")]
        r#type: LinkType,

        /// Dry run - show what would be materialized without modifying
        #[arg(long)]
        dry_run: bool,

        /// Remove inline links from body after materializing
        #[arg(long)]
        remove_inline: bool,
    },
}
