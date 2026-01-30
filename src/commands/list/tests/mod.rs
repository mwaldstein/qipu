//! Tests for `qipu list` command

pub mod compaction;
pub mod filter;
pub mod format;

use crate::cli::{Cli, OutputFormat};
use qipu_core::store::{InitOptions, Store};
use tempfile::TempDir;

pub fn create_cli(format: OutputFormat, quiet: bool) -> Cli {
    Cli {
        root: None,
        store: None,
        format,
        quiet,
        verbose: false,
        log_level: None,
        log_json: false,
        no_resolve_compaction: false,
        with_compaction_ids: false,
        compaction_depth: None,
        compaction_max_nodes: None,
        expand_compaction: false,
        workspace: None,
        no_semantic_inversion: false,
        command: None,
    }
}

pub fn create_test_store() -> (TempDir, Store) {
    let temp_dir = TempDir::new().unwrap();
    let store = Store::init(temp_dir.path(), InitOptions::default()).unwrap();
    (temp_dir, store)
}
