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
                        QipuError::UsageError(err.to_string())
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
    println!();
    println!("{TOP_LEVEL_HIDDEN_HELP_NOTICE}");

    if !include_advanced {
        return;
    }

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
