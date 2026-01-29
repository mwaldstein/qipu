use crate::error::Result;
use std::fs;
use std::path::Path;

/// Options for store initialization
#[derive(Debug, Clone, Default)]
pub struct InitOptions {
    /// Use visible store directory (`qipu/` instead of `.qipu/`)
    pub visible: bool,
    /// Stealth mode (add store to .gitignore)
    pub stealth: bool,
    /// Protected branch workflow (store notes on separate git branch)
    pub branch: Option<String>,
    /// Skip automatic indexing
    pub no_index: bool,
    /// Override auto-indexing strategy ("full", "incremental", "quick", "adaptive")
    pub index_strategy: Option<String>,
}

pub fn ensure_project_gitignore_entry(path: &Path, entry: &str) -> Result<()> {
    if path.exists() {
        let mut content = fs::read_to_string(path)?;
        if content.lines().any(|l| l.trim() == entry.trim()) {
            return Ok(());
        }

        if !content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str(entry.trim_end_matches('\n'));
        content.push('\n');
        fs::write(path, content)?;
    } else {
        fs::write(path, format!("{}\n", entry.trim_end_matches('\n')))?;
    }

    Ok(())
}
