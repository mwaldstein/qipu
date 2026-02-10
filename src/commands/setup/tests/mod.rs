use crate::cli::OutputFormat;
use crate::commands::setup::{execute_install_agents_md, execute_install_cursor, Cli};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

pub fn create_cli(format: OutputFormat) -> Cli {
    create_cli_with_root(format, None)
}

pub fn create_cli_with_root(format: OutputFormat, root: Option<PathBuf>) -> Cli {
    Cli {
        root,
        store: None,
        format,
        quiet: false,
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

pub fn setup_agents_md(temp_dir: &std::path::Path) -> PathBuf {
    let path = temp_dir.join("AGENTS.md");
    fs::write(&path, "Some content").unwrap();
    path
}

pub fn setup_cursor_rules(temp_dir: &std::path::Path) -> PathBuf {
    let rules_dir = temp_dir.join(".cursor").join("rules");
    fs::create_dir_all(&rules_dir).unwrap();
    let path = rules_dir.join("qipu.mdc");
    fs::write(&path, "Some cursor rules content").unwrap();
    path
}

pub fn assert_install_agents_md_success(format: OutputFormat, verify_content: bool) -> PathBuf {
    let temp_dir = TempDir::new().unwrap();
    let cli = create_cli_with_root(format, Some(temp_dir.path().to_path_buf()));

    let result = execute_install_agents_md(&cli);
    assert!(result.is_ok());

    let agents_md_path = temp_dir.path().join("AGENTS.md");
    assert!(agents_md_path.exists());

    if verify_content {
        let content = fs::read_to_string(&agents_md_path).unwrap();
        assert!(content.contains("Qipu is a Zettelkasten-inspired"));
        assert!(content.contains("Quick Start"));
    }

    agents_md_path
}

pub fn assert_install_cursor_success(format: OutputFormat, verify_content: bool) -> PathBuf {
    let temp_dir = TempDir::new().unwrap();
    let cli = create_cli_with_root(format, Some(temp_dir.path().to_path_buf()));

    let result = execute_install_cursor(&cli);
    assert!(result.is_ok());

    let cursor_rules_path = temp_dir
        .path()
        .join(".cursor")
        .join("rules")
        .join("qipu.mdc");
    assert!(cursor_rules_path.exists());

    if verify_content {
        let content = fs::read_to_string(&cursor_rules_path).unwrap();
        assert!(content.contains("Qipu Knowledge Management"));
        assert!(content.contains("description: Qipu Knowledge Management Integration"));
    }

    cursor_rules_path
}

#[cfg(test)]
mod agents_md;
#[cfg(test)]
mod cursor;
#[cfg(test)]
mod dispatcher;
#[cfg(test)]
mod output;
