use super::*;
use std::fs;
use tempfile::TempDir;

fn create_cli(format: OutputFormat) -> Cli {
    create_cli_with_root(format, None)
}

fn create_cli_with_root(format: OutputFormat, root: Option<PathBuf>) -> Cli {
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

fn assert_unknown_tool_error(result: Result<(), QipuError>) {
    match result.unwrap_err() {
        QipuError::UsageError(msg) => {
            assert!(msg.contains("Unknown integration"));
        }
        _ => panic!("Expected UsageError"),
    }
}

fn setup_agents_md(temp_dir: &PathBuf) -> PathBuf {
    let path = temp_dir.join("AGENTS.md");
    fs::write(&path, "Some content").unwrap();
    path
}

fn assert_install_success(format: OutputFormat, verify_content: bool) -> PathBuf {
    let temp_dir = TempDir::new().unwrap();
    let cli = create_cli_with_root(format, Some(temp_dir.path().to_path_buf()));

    let result = execute_install(&cli, "agents-md");
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

fn assert_execute_ok<F>(func: F)
where
    F: Fn(&Cli) -> Result<(), QipuError>,
{
    func(&create_cli(OutputFormat::Human)).unwrap();
    func(&create_cli(OutputFormat::Json)).unwrap();
    func(&create_cli(OutputFormat::Records)).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_list_all_formats() {
        assert_execute_ok(execute_list);
    }

    #[test]
    fn test_execute_print_all_formats() {
        assert_execute_ok(execute_print);
    }

    #[test]
    fn test_execute_install_success() {
        assert_install_success(OutputFormat::Human, true);
    }

    #[test]
    fn test_execute_install_json() {
        assert_install_success(OutputFormat::Json, false);
    }

    #[test]
    fn test_execute_install_records() {
        assert_install_success(OutputFormat::Records, false);
    }

    #[test]
    fn test_execute_install_unknown_tool() {
        let cli = create_cli(OutputFormat::Human);
        assert_unknown_tool_error(execute_install(&cli, "unknown-tool"));
    }

    #[test]
    fn test_execute_install_already_exists() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Json, Some(temp_dir.path().to_path_buf()));
        let path = setup_agents_md(&temp_dir.path().to_path_buf());

        let result = execute_install(&cli, "agents-md");
        assert!(result.is_ok());
        assert!(path.exists());

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "Some content");
    }

    #[test]
    fn test_execute_check_installed() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Json, Some(temp_dir.path().to_path_buf()));
        setup_agents_md(&temp_dir.path().to_path_buf());

        let result = execute_check(&cli, "agents-md");
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_check_not_installed() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Json, Some(temp_dir.path().to_path_buf()));

        let result = execute_check(&cli, "agents-md");
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_check_human() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Human, Some(temp_dir.path().to_path_buf()));

        let result = execute_check(&cli, "agents-md");
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_check_unknown_tool() {
        let cli = create_cli(OutputFormat::Human);
        assert_unknown_tool_error(execute_check(&cli, "unknown-tool"));
    }

    #[test]
    fn test_execute_remove_success() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Human, Some(temp_dir.path().to_path_buf()));
        let path = setup_agents_md(&temp_dir.path().to_path_buf());

        let result = execute_remove(&cli, "agents-md");
        assert!(result.is_ok());
        assert!(!path.exists());
    }

    #[test]
    fn test_execute_remove_json() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Json, Some(temp_dir.path().to_path_buf()));
        let path = setup_agents_md(&temp_dir.path().to_path_buf());

        let result = execute_remove(&cli, "agents-md");
        assert!(result.is_ok());
        assert!(!path.exists());
    }

    #[test]
    fn test_execute_remove_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Json, Some(temp_dir.path().to_path_buf()));

        let result = execute_remove(&cli, "agents-md");
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_remove_records() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Records, Some(temp_dir.path().to_path_buf()));
        let path = setup_agents_md(&temp_dir.path().to_path_buf());

        let result = execute_remove(&cli, "agents-md");
        assert!(result.is_ok());
        assert!(!path.exists());
    }

    #[test]
    fn test_execute_remove_unknown_tool() {
        let cli = create_cli(OutputFormat::Human);
        assert_unknown_tool_error(execute_remove(&cli, "unknown-tool"));
    }

    #[test]
    fn test_execute_with_list_flag() {
        let cli = create_cli(OutputFormat::Human);
        let result = execute(&cli, true, None, false, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_with_print_flag() {
        let cli = create_cli(OutputFormat::Human);
        let result = execute(&cli, false, None, true, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_with_check_flag() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Human, Some(temp_dir.path().to_path_buf()));

        let result = execute(&cli, false, Some("agents-md"), false, true, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_with_remove_flag() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Human, Some(temp_dir.path().to_path_buf()));
        let path = setup_agents_md(&temp_dir.path().to_path_buf());

        let result = execute(&cli, false, Some("agents-md"), false, false, true);
        assert!(result.is_ok());
        assert!(!path.exists());
    }

    #[test]
    fn test_execute_no_args() {
        let cli = create_cli(OutputFormat::Human);
        let result = execute(&cli, false, None, false, false, false);
        assert!(result.is_err());

        match result.unwrap_err() {
            QipuError::UsageError(msg) => {
                assert!(msg.contains("Specify --list, --print, or provide a tool name"));
            }
            _ => panic!("Expected UsageError"),
        }
    }

    #[test]
    fn test_execute_json_output_all_branches() {
        let cli = create_cli(OutputFormat::Json);

        let result = execute(&cli, true, None, false, false, false);
        assert!(result.is_ok());

        let result = execute(&cli, false, None, true, false, false);
        assert!(result.is_ok());

        let result = execute(&cli, false, Some("agents-md"), false, true, false);
        assert!(result.is_ok());

        let result = execute(&cli, false, Some("agents-md"), false, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_records_output_all_branches() {
        let cli = create_cli(OutputFormat::Records);

        let result = execute(&cli, true, None, false, false, false);
        assert!(result.is_ok());

        let result = execute(&cli, false, None, true, false, false);
        assert!(result.is_ok());

        let result = execute(&cli, false, Some("agents-md"), false, true, false);
        assert!(result.is_ok());

        let result = execute(&cli, false, Some("agents-md"), false, false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_onboard_all_formats() {
        assert_execute_ok(execute_onboard);
    }
}
