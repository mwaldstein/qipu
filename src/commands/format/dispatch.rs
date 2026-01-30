//! Format dispatch macros for command output
//!
//! Provides macros to eliminate repetitive format match blocks across commands.
//! These macros are defined in `crate::cli::format` and re-exported here for
//! convenient access from command modules.
//!
//! # Available Macros
//!
//! - `output_by_format!` - Simple dispatch when all branches return the same type
//! - `output_by_format_result!` - Dispatch when JSON returns Result and others return ()
//! - `output_by_format_with_path!` - Dispatch with custom format type path
//!
//! # Examples
//!
/// ```rust,ignore
/// use crate::commands::format::output_by_format_result;
///
/// // When json returns Result and others return ()
/// output_by_format_result!(cli.format,
///     json => {
///         serde_json::to_string_pretty(&data)?;
///         Ok(())
///     },
///     human => { println!("Done"); },
///     records => { println!("H mode=done"); }
/// )?;
/// ```
// Re-export macros from cli::format for convenient access
pub use crate::output_by_format;
pub use crate::output_by_format_result;
