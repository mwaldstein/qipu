//! Tests for qipu compaction command
//!
//! Compaction allows merging multiple source notes into a digest note.
//! Tests cover annotation display, apply commands, reporting, and suggestions.

mod annotations;
mod apply_errors;
mod apply_file;
mod apply_mixed;
mod apply_stdin;
mod multi_level;
mod report;
mod show;
mod status;
mod suggest;
mod suggest_value_default;
mod suggest_value_low;
mod suggest_value_mixed;
