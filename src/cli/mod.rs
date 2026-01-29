//! CLI argument parsing for qipu
//!
//! Uses clap for argument parsing per spec requirements.
//! Supports global flags: --root, --store, --format, --quiet, --verbose

pub mod args;
pub mod commands;
pub mod compact;
pub mod custom;
pub mod link;
pub mod ontology;
pub mod output;
pub mod parse;
pub mod store;
pub mod tags;
pub mod value;
pub mod workspace;

use clap::Parser;
use std::path::PathBuf;

/// Parse and validate log level argument
fn parse_log_level(s: &str) -> Result<String, String> {
    match s.to_lowercase().as_str() {
        "error" | "warn" | "info" | "debug" | "trace" => Ok(s.to_lowercase()),
        _ => Err(format!(
            "invalid log level '{}': expected one of: error, warn, info, debug, trace",
            s
        )),
    }
}

pub use args::CreateArgs;
pub use commands::Commands;
pub use compact::CompactCommands;
pub use custom::CustomCommands;
pub use link::LinkCommands;
pub use ontology::OntologyCommands;
pub use output::OutputFormat;
pub use store::StoreCommands;
pub use tags::TagsCommands;
pub use value::ValueCommands;
pub use workspace::WorkspaceCommands;

/// Qipu - Zettelkasten-inspired knowledge management CLI
#[derive(Parser, Debug)]
#[command(name = "qipu")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Base directory for resolving the store
    #[arg(long, global = true)]
    pub root: Option<PathBuf>,

    /// Explicit store root path
    #[arg(long, global = true, env = "QIPU_STORE")]
    pub store: Option<PathBuf>,

    /// Output format
    #[arg(long, global = true, value_enum, default_value = "human")]
    pub format: OutputFormat,

    /// Suppress non-essential output
    #[arg(long, short, global = true)]
    pub quiet: bool,

    /// Report timing for major phases
    #[arg(long, short, global = true)]
    pub verbose: bool,

    /// Set log level (error, warn, info, debug, trace)
    #[arg(long, global = true, value_name = "LEVEL", value_parser = parse_log_level)]
    pub log_level: Option<String>,

    /// Output logs in JSON format
    #[arg(long, global = true)]
    pub log_json: bool,

    /// Disable compaction resolution (show raw compacted notes)
    #[arg(long, global = true)]
    pub no_resolve_compaction: bool,

    /// Include compacted note IDs in output
    #[arg(long, global = true)]
    pub with_compaction_ids: bool,

    /// Compaction traversal depth (requires --with-compaction-ids)
    #[arg(long, global = true)]
    pub compaction_depth: Option<u32>,

    /// Maximum compacted notes to include in output
    #[arg(long, global = true)]
    pub compaction_max_nodes: Option<usize>,

    /// Expand compacted notes to include full content (context command only)
    #[arg(long, global = true)]
    pub expand_compaction: bool,

    /// Target workspace name
    #[arg(long, global = true)]
    pub workspace: Option<String>,

    /// Disable semantic inversion for link listing/traversal
    #[arg(long, global = true)]
    pub no_semantic_inversion: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::note::NoteType;

    #[test]
    fn test_parse_cli_help() {
        // Should not panic
        let result = Cli::try_parse_from(["qipu", "--help"]);
        assert!(result.is_err()); // --help exits
    }

    #[test]
    fn test_parse_cli_version() {
        // Should not panic
        let result = Cli::try_parse_from(["qipu", "--version"]);
        assert!(result.is_err()); // --version exits
    }

    #[test]
    fn test_parse_init() {
        let cli = Cli::try_parse_from(["qipu", "init"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Init { .. })));
    }

    #[test]
    fn test_parse_create() {
        let cli = Cli::try_parse_from(["qipu", "create", "My Note"]).unwrap();
        if let Some(Commands::Create(args)) = cli.command {
            assert_eq!(args.title, "My Note");
        } else {
            panic!("Expected Create command");
        }
    }

    #[test]
    fn test_parse_create_with_options() {
        let cli = Cli::try_parse_from([
            "qipu",
            "create",
            "My Note",
            "--type",
            "permanent",
            "--tag",
            "test",
            "--tag",
            "demo",
        ])
        .unwrap();
        if let Some(Commands::Create(args)) = cli.command {
            assert_eq!(args.title, "My Note");
            assert_eq!(args.r#type, Some(NoteType::from(NoteType::PERMANENT)));
            assert_eq!(args.tag, vec!["test", "demo"]);
        } else {
            panic!("Expected Create command");
        }
    }

    #[test]
    fn test_parse_list() {
        let cli = Cli::try_parse_from(["qipu", "list"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::List { .. })));
    }

    #[test]
    fn test_parse_list_with_filters() {
        let cli =
            Cli::try_parse_from(["qipu", "list", "--tag", "test", "--type", "fleeting"]).unwrap();
        if let Some(Commands::List(args)) = &cli.command {
            assert_eq!(args.tag, Some("test".to_string()));
            assert_eq!(args.r#type, Some(NoteType::from(NoteType::FLEETING)));
            assert_eq!(args.since, None);
            assert_eq!(args.custom, None);
            assert_eq!(args.min_value, None);
            assert!(!args.show_custom);
        } else {
            panic!("Expected List command");
        }
    }

    #[test]
    fn test_parse_list_with_min_value() {
        let cli = Cli::try_parse_from(["qipu", "list", "--min-value", "75"]).unwrap();
        if let Some(Commands::List(args)) = &cli.command {
            assert_eq!(args.min_value, Some(75));
        } else {
            panic!("Expected List command");
        }
    }

    #[test]
    fn test_parse_format() {
        let cli = Cli::try_parse_from(["qipu", "--format", "json", "list"]).unwrap();
        assert_eq!(cli.format, OutputFormat::Json);
    }

    #[test]
    fn test_parse_valid_log_levels() {
        // Test all valid log levels
        for level in ["error", "warn", "info", "debug", "trace"] {
            let cli = Cli::try_parse_from(["qipu", "--log-level", level, "list"]).unwrap();
            assert_eq!(cli.log_level, Some(level.to_string()));
        }
    }

    #[test]
    fn test_parse_log_level_case_insensitive() {
        let cli = Cli::try_parse_from(["qipu", "--log-level", "DEBUG", "list"]).unwrap();
        assert_eq!(cli.log_level, Some("debug".to_string()));
    }

    #[test]
    fn test_parse_invalid_log_level() {
        let result = Cli::try_parse_from(["qipu", "--log-level", "invalid", "list"]);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("invalid log level"));
        assert!(err.contains("error, warn, info, debug, trace"));
    }
}
