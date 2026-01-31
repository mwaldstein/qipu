//! Telemetry command argument structures

use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum TelemetryCommands {
    Show,
    Manifest,
}
