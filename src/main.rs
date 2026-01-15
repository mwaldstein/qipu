//! Qipu - Zettelkasten-inspired knowledge management CLI
//!
//! A command-line tool for capturing research, distilling insights,
//! and navigating knowledge via links, tags, and Maps of Content.

mod cli;
mod commands;
#[path = "lib/mod.rs"]
mod lib;

use std::env;
use std::process::ExitCode;
use std::time::Instant;

use clap::Parser;

use cli::{Cli, OutputFormat};
use lib::error::{ExitCode as QipuExitCode, QipuError};
use lib::logging;

fn main() -> ExitCode {
    let start = Instant::now();

    let argv_format_json = argv_requests_json();

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            // `--format` is a global flag, but clap may fail parsing before we can
            // inspect `Cli.format`. If the user requested JSON output, emit a
            // structured error envelope.
            if argv_format_json {
                let qipu_error = match err.kind() {
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

    logging::set_verbose(cli.verbose);

    if cli.verbose {
        eprintln!("parse_args: {:?}", start.elapsed());
    }

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

fn argv_requests_json() -> bool {
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--format" {
            if let Some(value) = args.next() {
                if value == "json" {
                    return true;
                }
            }
        }
    }
    false
}
