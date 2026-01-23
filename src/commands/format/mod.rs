//! Shared output formatting helpers for commands
//!
//! Provides common patterns for JSON status messages, Records headers,
//! and other repeated formatting patterns across command modules.

pub mod status;

pub use status::{
    add_compaction_to_json, build_compaction_annotations, format_custom_value, print_json_status,
    print_records_header, wrap_records_body,
};
