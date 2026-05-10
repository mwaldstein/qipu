//! Qipu - Knowledge graph CLI for scripts and agents
//!
//! A command-line tool for capturing research, distilling insights,
//! and navigating knowledge via links, tags, and linked collection roots.

// QipuError is intentionally rich with context; errors are exceptional paths
#![allow(clippy::result_large_err)]

mod cli;
mod commands;

use std::env;
use std::process::ExitCode;
use std::time::Instant;

use clap::{Command, CommandFactory, FromArgMatches};

use cli::{Cli, OutputFormat};
use qipu_core::error::{ExitCode as QipuExitCode, QipuError};
use qipu_core::logging;

const ADVANCED_GLOBAL_HELP_NOTICE: &str = "Advanced global options are hidden from this help to keep common workflows readable. Run `qipu --help-advanced` to show compaction and traversal tuning flags.";
const TOP_LEVEL_HIDDEN_HELP_NOTICE: &str = "Command aliases, extension commands, and advanced global options are hidden from this help to keep common workflows readable. Run `qipu --help-advanced` to show hidden commands and compaction/traversal tuning flags.";

fn main() -> ExitCode {
    let start = Instant::now();

    if argv_requests_advanced_help() {
        print_top_level_help(true);
        return ExitCode::from(QipuExitCode::Success as u8);
    }

    if argv_requests_top_level_help() {
        print_top_level_help(false);
        return ExitCode::from(QipuExitCode::Success as u8);
    }

    let argv_format_json = argv_requests_json();

    let cli = match parse_cli() {
        Ok(cli) => cli,
        Err(err) => {
            let guidance = custom_parse_error_guidance(&err);
            // `--format` is a global flag, but clap may fail parsing before we can
            // inspect `Cli.format`. If the user requested JSON output, emit a
            // structured error envelope.
            if argv_format_json {
                let qipu_error = match err.kind() {
                    // Help and version are informational, not errors - let clap handle them
                    clap::error::ErrorKind::DisplayHelp
                    | clap::error::ErrorKind::DisplayVersion => err.exit(),
                    clap::error::ErrorKind::ValueValidation
                    | clap::error::ErrorKind::InvalidValue
                    | clap::error::ErrorKind::InvalidSubcommand
                    | clap::error::ErrorKind::UnknownArgument
                    | clap::error::ErrorKind::MissingRequiredArgument => {
                        QipuError::UsageError(guidance.unwrap_or_else(|| err.to_string()))
                    }
                    clap::error::ErrorKind::ArgumentConflict => {
                        // This includes duplicate `--format`.
                        QipuError::DuplicateFormat
                    }
                    _ => QipuError::Other(err.to_string()),
                };

                eprintln!("{}", qipu_error.to_json());
                return ExitCode::from(qipu_error.exit_code() as u8);
            }

            if let Some(guidance) = guidance {
                eprintln!("error: {}", guidance);
                return ExitCode::from(QipuExitCode::Usage as u8);
            }

            err.exit();
        }
    };

    // Initialize structured logging
    if let Err(e) = logging::init_tracing(cli.verbose, cli.log_level.as_deref(), cli.log_json) {
        // If tracing initialization fails, fall back to stderr
        eprintln!("Warning: Failed to initialize logging: {}", e);
    }

    tracing::debug!(elapsed = ?start.elapsed(), "parse_args");

    let result = commands::dispatch::run(&cli, start);

    match result {
        Ok(()) => ExitCode::from(QipuExitCode::Success as u8),
        Err(e) => {
            let exit_code = e.exit_code();

            if cli.format == OutputFormat::Json {
                eprintln!("{}", e.to_json());
            } else if !cli.quiet {
                eprintln!("error: {}", e);
            }

            ExitCode::from(exit_code as u8)
        }
    }
}

fn parse_cli() -> Result<Cli, clap::Error> {
    let matches = command_with_help_notices().try_get_matches()?;
    Cli::from_arg_matches(&matches)
}

fn command_with_help_notices() -> Command {
    let mut command = Cli::command().after_help(TOP_LEVEL_HIDDEN_HELP_NOTICE);
    for subcommand in command.get_subcommands_mut() {
        add_advanced_global_help_notice(subcommand);
    }
    command
}

fn add_advanced_global_help_notice(command: &mut Command) {
    *command = command.clone().after_help(ADVANCED_GLOBAL_HELP_NOTICE);
    for subcommand in command.get_subcommands_mut() {
        add_advanced_global_help_notice(subcommand);
    }
}

fn argv_requests_advanced_help() -> bool {
    env::args().skip(1).any(|arg| arg == "--help-advanced")
}

fn argv_requests_top_level_help() -> bool {
    let args: Vec<String> = env::args().skip(1).collect();
    matches!(args.as_slice(), [arg] if arg == "--help" || arg == "-h" || arg == "help")
}

fn print_top_level_help(include_advanced: bool) {
    println!("Knowledge graph CLI designed for scripts and agents");
    println!();
    println!("Usage: qipu [OPTIONS] [COMMAND]");
    println!();
    print_top_level_command_groups();
    print_top_level_options();
    println!();
    println!("{TOP_LEVEL_HIDDEN_HELP_NOTICE}");

    if include_advanced {
        print_top_level_advanced_help();
    }
}

fn print_top_level_command_groups() {
    println!("Core commands:");
    println!("  capture     Create a new note from stdin");
    println!("  create      Create a new note");
    println!("  list        List notes");
    println!("  show        Show a note");
    println!("  search      Search notes by title and body");
    println!("  inbox       List unprocessed notes (fleeting/literature)");
    println!("  edit        Open a note in $EDITOR and update the index upon completion");
    println!("  update      Update a note's metadata or content non-interactively");
    println!();
    println!("Graph and context commands:");
    println!("  link        Manage and traverse note links");
    println!("  context     Build context bundle for LLM integration");
    println!("  export      Export notes to a single document");
    println!("  dump        Dump notes to a pack file");
    println!("  load        Load notes from a pack file");
    println!();
    println!("Agent commands:");
    println!("  prime       Output session-start primer for LLM agents");
    println!("  quickstart  Quick start guide for common workflows");
    println!("  onboard     Display minimal AGENTS.md snippet for agent integration");
    println!("  setup       Install qipu integration instructions for agent tools");
    println!();
    println!("Metadata commands:");
    println!("  value       Manage note value (quality/importance score)");
    println!("  tags        Manage and query tags");
    println!("  verify      Toggle verification status of a note");
    println!();
    println!("Maintenance commands:");
    println!("  init        Initialize a new qipu store");
    println!("  status      Check whether a usable qipu store is available");
    println!("  doctor      Validate store invariants and optionally repair issues");
    println!("  sync        Sync store: update indexes and optionally validate");
    println!("  index       Build or refresh derived indexes");
    println!("  store       Manage the qipu store");
    println!();
    println!("Configuration commands:");
    println!("  ontology    Manage and display ontology configuration");
    println!("  telemetry   Manage anonymous usage analytics");
    println!("  hooks       Manage git hooks for automatic store sync");
    println!();
    println!("Advanced commands:");
    println!("  compact     Manage note compaction (digest-first navigation)");
    println!("  workspace   Manage and navigate isolated workspaces");
    println!("  merge       Merge note id1 into id2");
    println!();
    println!("Help:");
    println!("  help        Print this message or the help of the given subcommand(s)");
    println!("  <command> --help");
    println!("              Print help for a command");
    println!();
}

fn print_top_level_options() {
    println!("Options:");
    println!("      --root <ROOT>            Base directory for resolving the store");
    println!("      --store <STORE>          Explicit store root path [env: QIPU_STORE=]");
    println!("      --format <FORMAT>        Output format [default: human]");
    println!("  -q, --quiet                  Suppress non-essential output");
    println!("  -v, --verbose                Report timing for major phases");
    println!("      --log-level <LEVEL>      Set log level (error, warn, info, debug, trace)");
    println!("      --log-json               Output logs in JSON format");
    println!("      --workspace <WORKSPACE>  Target workspace name");
    println!("  -h, --help                   Print help");
    println!("  -V, --version                Print version");
}

fn print_top_level_advanced_help() {
    println!();
    println!("Hidden commands:");
    println!("  new         Alias for create");
    println!("  custom      Manage custom note metadata (for applications building on qipu)");
    println!();
    println!(
        "These commands are hidden from standard help because they are aliases or extension surfaces rather than common user workflows."
    );
    println!();
    println!("Advanced global options:");
    println!(
        "      --no-resolve-compaction\n          Disable compaction resolution (show raw compacted notes)"
    );
    println!("      --with-compaction-ids\n          Include compacted note IDs in output");
    println!(
        "      --compaction-depth <COMPACTION_DEPTH>\n          Compaction traversal depth (requires --with-compaction-ids)"
    );
    println!(
        "      --compaction-max-nodes <COMPACTION_MAX_NODES>\n          Maximum compacted notes to include in output"
    );
    println!(
        "      --expand-compaction\n          Expand compacted notes to include full content (context command only)"
    );
    println!(
        "      --no-semantic-inversion\n          Disable semantic inversion for link listing/traversal"
    );
    println!();
    println!(
        "These options are hidden from standard help because they tune advanced compaction and graph traversal behavior. They are still supported anywhere global options are accepted."
    );
}

fn argv_requests_json() -> bool {
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--format" {
            if args.next().is_some_and(|v| v == "json") {
                return true;
            }
        } else if arg == "--format=json" {
            return true;
        }
    }
    false
}

fn custom_parse_error_guidance(err: &clap::Error) -> Option<String> {
    use clap::error::ErrorKind;

    if matches!(
        err.kind(),
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
    ) {
        return None;
    }

    let args: Vec<String> = env::args().skip(1).collect();
    let command_args: Vec<&str> = args
        .iter()
        .filter(|arg| !matches!(arg.as_str(), "--format" | "human" | "json" | "records"))
        .map(String::as_str)
        .collect();

    if args
        .windows(2)
        .any(|w| w[0] == "--format" && w[1] == "yaml")
        || args.iter().any(|arg| arg == "--format=yaml")
    {
        return Some(
            "unknown format: yaml (expected: human, json, or records)\n\nUse: qipu --format json status\nOther formats: human, records.\nRun `qipu status --help` for command details."
                .to_string(),
        );
    }

    match command_args.as_slice() {
        ["show"] => Some(note_id_guidance("show", "qipu show <id-or-path>")),
        ["edit"] => Some(note_id_guidance("edit", "qipu edit <id-or-path>")),
        ["update", ..] if err.kind() == ErrorKind::MissingRequiredArgument => {
            Some("update requires a note id or path\n\nUse: qipu update <id-or-path> --title \"Title\"\nReplace body: printf \"new body\" | qipu update <id-or-path>\nFind notes: qipu list OR qipu search \"query\"\nRun `qipu update --help` for full and advanced details.".to_string())
        }
        ["verify"] => Some(note_id_guidance("verify", "qipu verify <id-or-path>")),
        ["link", "add", ..] | ["link", "remove", ..]
            if command_args.iter().any(|arg| matches!(*arg, "--from" | "--to")) =>
        {
            Some("use positional note IDs\n\nUse: qipu link add <from> <to> --type <type>\nRun `qipu link --help` for full and advanced details.".to_string())
        }
        ["context", ..] if command_args.contains(&"--id") => Some(
            "Use: qipu context --note <id>\nOther selectors: --tag, --moc/--collection-root, --query, --walk.\nRun `qipu context --help` for full and advanced details."
                .to_string(),
        ),
        ["export", value, ..] if !value.starts_with('-') => Some(
            "Use: qipu export --note <id> [--output <file>]\nOther selectors: --tag, --moc/--collection-root, --query.\nRun `qipu export --help` for full and advanced details."
                .to_string(),
        ),
        ["dump", ..] if command_args.contains(&"--id") => Some(
            "Use: qipu dump --note <id> [--output <file>]\nPositional FILE is the pack output path, not a note selector.\nRun `qipu dump --help` for full and advanced details."
                .to_string(),
        ),
        ["link", "materialize", ..] if command_args.contains(&"--note") => Some(
            "Use: qipu link materialize <id-or-path> [--type <type>] [--dry-run]\nRun `qipu link materialize --help` for full and advanced details."
                .to_string(),
        ),
        ["compact", "apply", ..] if command_args.contains(&"--digest") => Some(
            "Use: qipu compact apply <digest-id> --note <source-id> [--note <source-id>...]\nRun `qipu compact apply --help` for full and advanced details."
                .to_string(),
        ),
        ["compact", "show", ..] if command_args.contains(&"--digest") => Some(
            "Use: qipu compact show <digest-id>\nRun `qipu compact show --help` for full and advanced details."
                .to_string(),
        ),
        ["compact", "status", ..] if command_args.contains(&"--note") => Some(
            "Use: qipu compact status <id>\nRun `qipu compact status --help` for full and advanced details."
                .to_string(),
        ),
        ["store"] => Some(missing_subcommand_guidance("store", "qipu store stats")),
        ["ontology"] => Some(missing_subcommand_guidance("ontology", "qipu ontology show")),
        ["telemetry"] => Some(missing_subcommand_guidance("telemetry", "qipu telemetry status")),
        ["hooks"] => Some(missing_subcommand_guidance("hooks", "qipu hooks status")),
        ["workspace"] => Some(missing_subcommand_guidance("workspace", "qipu workspace list")),
        ["capture", value, ..] if !value.starts_with('-') => Some(capture_usage_guidance()),
        ["capture", ..] if command_args.contains(&"--body") => Some(capture_usage_guidance()),
        ["update", _, "--body", ..] | ["update", _, "-c", ..] => Some(
            "update reads replacement body from stdin\n\nUse: printf \"new text\" | qipu update <id-or-path>\nTo update metadata only: qipu update <id-or-path> --title \"Title\" --tag tag\nRun `qipu update --help` for full and advanced details."
                .to_string(),
        ),
        _ => None,
    }
}

fn note_id_guidance(command: &str, usage: &str) -> String {
    format!(
        "{command} requires a note id or path\n\nUse: {usage}\nFind notes: qipu list OR qipu search \"query\"\nRun `qipu {command} --help` for full and advanced details."
    )
}

fn missing_subcommand_guidance(command: &str, usage: &str) -> String {
    format!(
        "missing {command} subcommand\n\nUse: {usage}\nRun `qipu {command} --help` for full and advanced details."
    )
}

fn capture_usage_guidance() -> String {
    "capture reads content from stdin\n\nUse: printf \"Body text\" | qipu capture --title \"Title\"\nFor inline body text: qipu create \"Title\" --body \"Body text\"\nRun `qipu capture --help` for full and advanced details."
        .to_string()
}
