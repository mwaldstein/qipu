use super::*;
use crate::cli::OutputFormat;
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

fn setup_agents_md(temp_dir: &std::path::Path) -> PathBuf {
    let path = temp_dir.join("AGENTS.md");
    fs::write(&path, "Some content").unwrap();
    path
}

fn setup_cursor_rules(temp_dir: &std::path::Path) -> PathBuf {
    let rules_dir = temp_dir.join(".cursor").join("rules");
    fs::create_dir_all(&rules_dir).unwrap();
    let path = rules_dir.join("qipu.mdc");
    fs::write(&path, "Some cursor rules content").unwrap();
    path
}

fn assert_install_agents_md_success(format: OutputFormat, verify_content: bool) -> PathBuf {
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

fn assert_install_cursor_success(format: OutputFormat, verify_content: bool) -> PathBuf {
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

fn assert_execute_ok<F>(func: F)
where
    F: Fn(&Cli, std::time::Instant) -> Result<(), QipuError>,
{
    func(&create_cli(OutputFormat::Human), std::time::Instant::now()).unwrap();
    func(&create_cli(OutputFormat::Json), std::time::Instant::now()).unwrap();
    func(
        &create_cli(OutputFormat::Records),
        std::time::Instant::now(),
    )
    .unwrap();
}

#[cfg(test)]
#[allow(clippy::module_inception)]
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

    // AGENTS.md tests
    #[test]
    fn test_execute_install_agents_md_success() {
        assert_install_agents_md_success(OutputFormat::Human, true);
    }

    #[test]
    fn test_execute_install_agents_md_json() {
        assert_install_agents_md_success(OutputFormat::Json, false);
    }

    #[test]
    fn test_execute_install_agents_md_records() {
        assert_install_agents_md_success(OutputFormat::Records, false);
    }

    #[test]
    fn test_execute_install_agents_md_already_exists() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Json, Some(temp_dir.path().to_path_buf()));
        let path = setup_agents_md(temp_dir.path());

        let result = execute_install_agents_md(&cli);
        assert!(result.is_ok());
        assert!(path.exists());

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "Some content");
    }

    #[test]
    fn test_execute_check_agents_md_installed() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Json, Some(temp_dir.path().to_path_buf()));
        setup_agents_md(temp_dir.path());

        let result = execute_check_agents_md(&cli);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_check_agents_md_not_installed() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Json, Some(temp_dir.path().to_path_buf()));

        let result = execute_check_agents_md(&cli);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_check_agents_md_human() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Human, Some(temp_dir.path().to_path_buf()));

        let result = execute_check_agents_md(&cli);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_remove_agents_md_success() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Human, Some(temp_dir.path().to_path_buf()));
        let path = setup_agents_md(temp_dir.path());

        let result = execute_remove_agents_md(&cli);
        assert!(result.is_ok());
        assert!(!path.exists());
    }

    #[test]
    fn test_execute_remove_agents_md_json() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Json, Some(temp_dir.path().to_path_buf()));
        let path = setup_agents_md(temp_dir.path());

        let result = execute_remove_agents_md(&cli);
        assert!(result.is_ok());
        assert!(!path.exists());
    }

    #[test]
    fn test_execute_remove_agents_md_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Json, Some(temp_dir.path().to_path_buf()));

        let result = execute_remove_agents_md(&cli);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_remove_agents_md_records() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Records, Some(temp_dir.path().to_path_buf()));
        let path = setup_agents_md(temp_dir.path());

        let result = execute_remove_agents_md(&cli);
        assert!(result.is_ok());
        assert!(!path.exists());
    }

    // Cursor tests
    #[test]
    fn test_execute_install_cursor_success() {
        assert_install_cursor_success(OutputFormat::Human, true);
    }

    #[test]
    fn test_execute_install_cursor_json() {
        assert_install_cursor_success(OutputFormat::Json, false);
    }

    #[test]
    fn test_execute_install_cursor_records() {
        assert_install_cursor_success(OutputFormat::Records, false);
    }

    #[test]
    fn test_execute_install_cursor_already_exists() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Json, Some(temp_dir.path().to_path_buf()));
        let path = setup_cursor_rules(temp_dir.path());

        let result = execute_install_cursor(&cli);
        assert!(result.is_ok());
        assert!(path.exists());

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "Some cursor rules content");
    }

    #[test]
    fn test_execute_check_cursor_installed() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Json, Some(temp_dir.path().to_path_buf()));
        setup_cursor_rules(temp_dir.path());

        let result = execute_check_cursor(&cli);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_check_cursor_not_installed() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Json, Some(temp_dir.path().to_path_buf()));

        let result = execute_check_cursor(&cli);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_check_cursor_human() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Human, Some(temp_dir.path().to_path_buf()));

        let result = execute_check_cursor(&cli);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_remove_cursor_success() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Human, Some(temp_dir.path().to_path_buf()));
        let path = setup_cursor_rules(temp_dir.path());

        let result = execute_remove_cursor(&cli);
        assert!(result.is_ok());
        assert!(!path.exists());
    }

    #[test]
    fn test_execute_remove_cursor_json() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Json, Some(temp_dir.path().to_path_buf()));
        let path = setup_cursor_rules(temp_dir.path());

        let result = execute_remove_cursor(&cli);
        assert!(result.is_ok());
        assert!(!path.exists());
    }

    #[test]
    fn test_execute_remove_cursor_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Json, Some(temp_dir.path().to_path_buf()));

        let result = execute_remove_cursor(&cli);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_remove_cursor_records() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Records, Some(temp_dir.path().to_path_buf()));
        let path = setup_cursor_rules(temp_dir.path());

        let result = execute_remove_cursor(&cli);
        assert!(result.is_ok());
        assert!(!path.exists());
    }

    // Execute dispatcher tests
    #[test]
    fn test_execute_with_list_flag() {
        let cli = create_cli(OutputFormat::Human);
        let result = execute(
            &cli,
            true,
            None,
            false,
            false,
            false,
            std::time::Instant::now(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_with_print_flag() {
        let cli = create_cli(OutputFormat::Human);
        let result = execute(
            &cli,
            false,
            None,
            true,
            false,
            false,
            std::time::Instant::now(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_with_check_flag_agents_md() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Human, Some(temp_dir.path().to_path_buf()));

        let result = execute(
            &cli,
            false,
            Some("agents-md"),
            false,
            true,
            false,
            std::time::Instant::now(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_with_check_flag_cursor() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Human, Some(temp_dir.path().to_path_buf()));

        let result = execute(
            &cli,
            false,
            Some("cursor"),
            false,
            true,
            false,
            std::time::Instant::now(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_with_remove_flag_agents_md() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Human, Some(temp_dir.path().to_path_buf()));
        let path = setup_agents_md(temp_dir.path());

        let result = execute(
            &cli,
            false,
            Some("agents-md"),
            false,
            false,
            true,
            std::time::Instant::now(),
        );
        assert!(result.is_ok());
        assert!(!path.exists());
    }

    #[test]
    fn test_execute_with_remove_flag_cursor() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Human, Some(temp_dir.path().to_path_buf()));
        let path = setup_cursor_rules(temp_dir.path());

        let result = execute(
            &cli,
            false,
            Some("cursor"),
            false,
            false,
            true,
            std::time::Instant::now(),
        );
        assert!(result.is_ok());
        assert!(!path.exists());
    }

    #[test]
    fn test_execute_no_args() {
        let cli = create_cli(OutputFormat::Human);
        let result = execute(
            &cli,
            false,
            None,
            false,
            false,
            false,
            std::time::Instant::now(),
        );
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
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Json, Some(temp_dir.path().to_path_buf()));
        setup_agents_md(temp_dir.path());

        let result = execute(
            &cli,
            true,
            None,
            false,
            false,
            false,
            std::time::Instant::now(),
        );
        assert!(result.is_ok());

        let result = execute(
            &cli,
            false,
            None,
            true,
            false,
            false,
            std::time::Instant::now(),
        );
        assert!(result.is_ok());

        let result = execute(
            &cli,
            false,
            Some("agents-md"),
            false,
            true,
            false,
            std::time::Instant::now(),
        );
        assert!(result.is_ok());

        let result = execute(
            &cli,
            false,
            Some("agents-md"),
            false,
            false,
            true,
            std::time::Instant::now(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_records_output_all_branches() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Records, Some(temp_dir.path().to_path_buf()));
        setup_agents_md(temp_dir.path());

        let result = execute(
            &cli,
            true,
            None,
            false,
            false,
            false,
            std::time::Instant::now(),
        );
        assert!(result.is_ok());

        let result = execute(
            &cli,
            false,
            None,
            true,
            false,
            false,
            std::time::Instant::now(),
        );
        assert!(result.is_ok());

        let result = execute(
            &cli,
            false,
            Some("agents-md"),
            false,
            true,
            false,
            std::time::Instant::now(),
        );
        assert!(result.is_ok());

        let result = execute(
            &cli,
            false,
            Some("agents-md"),
            false,
            false,
            true,
            std::time::Instant::now(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_cursor_json_output_all_branches() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Json, Some(temp_dir.path().to_path_buf()));
        setup_cursor_rules(temp_dir.path());

        let result = execute(
            &cli,
            true,
            None,
            false,
            false,
            false,
            std::time::Instant::now(),
        );
        assert!(result.is_ok());

        let result = execute(
            &cli,
            false,
            None,
            true,
            false,
            false,
            std::time::Instant::now(),
        );
        assert!(result.is_ok());

        let result = execute(
            &cli,
            false,
            Some("cursor"),
            false,
            true,
            false,
            std::time::Instant::now(),
        );
        assert!(result.is_ok());

        let result = execute(
            &cli,
            false,
            Some("cursor"),
            false,
            false,
            true,
            std::time::Instant::now(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_cursor_records_output_all_branches() {
        let temp_dir = TempDir::new().unwrap();
        let cli = create_cli_with_root(OutputFormat::Records, Some(temp_dir.path().to_path_buf()));
        setup_cursor_rules(temp_dir.path());

        let result = execute(
            &cli,
            true,
            None,
            false,
            false,
            false,
            std::time::Instant::now(),
        );
        assert!(result.is_ok());

        let result = execute(
            &cli,
            false,
            None,
            true,
            false,
            false,
            std::time::Instant::now(),
        );
        assert!(result.is_ok());

        let result = execute(
            &cli,
            false,
            Some("cursor"),
            false,
            true,
            false,
            std::time::Instant::now(),
        );
        assert!(result.is_ok());

        let result = execute(
            &cli,
            false,
            Some("cursor"),
            false,
            false,
            true,
            std::time::Instant::now(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_onboard_all_formats() {
        assert_execute_ok(execute_onboard);
    }
}
