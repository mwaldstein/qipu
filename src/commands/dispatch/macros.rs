//! Macros for command timing and logging

/// Trace command execution with optional verbose output
///
/// Usage:
/// ```ignore
/// trace_command!(cli, start, "discover_store");
/// trace_command!(params.cli, params.start, "execute_command");
/// ```
macro_rules! trace_command {
    ($cli:expr, $start:expr, $label:expr) => {
        if $cli.verbose {
            ::tracing::debug!(elapsed = ?$start.elapsed(), $label);
        }
    };
}

/// Trace command execution without verbose check
///
/// Usage:
/// ```ignore
/// trace_command_always!(start, "value_set");
/// ```
macro_rules! trace_command_always {
    ($start:expr, $label:expr) => {
        ::tracing::debug!(elapsed = ?$start.elapsed(), $label);
    };
}

pub(crate) use trace_command;
pub(crate) use trace_command_always;
