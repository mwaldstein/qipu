//! Telemetry command argument structures

use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum TelemetryCommands {
    Enable,
    Disable,
    Status,
    /// Show pending telemetry events (dry run of what would be uploaded)
    Show,
}
