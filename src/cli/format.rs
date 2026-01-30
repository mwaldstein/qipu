//! Format output dispatch helpers
//!
//! Provides macros to eliminate repetitive format match blocks.

/// Macro to dispatch output by format with minimal boilerplate.
///
/// # Examples
///
/// ```rust,ignore
/// output_by_format!(cli.format,
///     json => { output_json()? },
///     human => { output_human(); },
///     records => { output_records(); }
/// );
/// ```
#[macro_export]
macro_rules! output_by_format {
    ($format:expr, json => $json:block, human => $human:block, records => $records:block) => {
        match $format {
            $crate::cli::OutputFormat::Json => $json,
            $crate::cli::OutputFormat::Human => $human,
            $crate::cli::OutputFormat::Records => $records,
        }
    };
}

/// Macro for format dispatch that automatically wraps result handling.
/// Use when json branch returns Result and human/records return ().
#[macro_export]
macro_rules! output_by_format_result {
    ($format:expr, json => $json:expr, human => $human:block, records => $records:block) => {
        match $format {
            $crate::cli::OutputFormat::Json => $json,
            $crate::cli::OutputFormat::Human => {
                $human;
                Ok(())
            }
            $crate::cli::OutputFormat::Records => {
                $records;
                Ok(())
            }
        }
    };
}
