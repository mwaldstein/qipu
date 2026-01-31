use crate::cli::Cli;
use qipu_core::error::Result;
use qipu_core::note::Note;
use qipu_core::store::Store;
use std::path::Path;
use tracing::debug;

/// Discover or open store for compact commands
/// Resolves root directory, handles --store flag, or discovers store from root
pub fn discover_compact_store(cli: &Cli, root: &Path) -> Result<Store> {
    let store = if let Some(path) = &cli.store {
        let resolved = if path.is_absolute() {
            path.clone()
        } else {
            root.join(path)
        };
        Store::open(&resolved)?
    } else {
        Store::discover(root)?
    };

    if cli.verbose {
        debug!(store = %store.root().display(), "discover_store");
    }

    Ok(store)
}

/// Estimate note size for compaction metrics
/// Uses summary-sized content (same as records output)
pub fn estimate_size(note: &Note) -> usize {
    // Use summary if present
    if let Some(summary) = &note.frontmatter.summary {
        return summary.len();
    }

    // Otherwise use first paragraph or truncated body
    let summary = note.summary();
    summary.len()
}
