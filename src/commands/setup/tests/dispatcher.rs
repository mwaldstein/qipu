use crate::cli::OutputFormat;
use crate::commands::setup::tests::{create_cli, create_cli_with_root, setup_agents_md};
use crate::commands::setup::{execute, execute_list, execute_onboard, execute_print};
use qipu_core::error::QipuError;
use tempfile::TempDir;

fn assert_execute_ok<F>(func: F)
where
    F: Fn(&crate::commands::setup::Cli, std::time::Instant) -> Result<(), QipuError>,
{
    func(&create_cli(OutputFormat::Human), std::time::Instant::now()).unwrap();
    func(&create_cli(OutputFormat::Json), std::time::Instant::now()).unwrap();
    func(
        &create_cli(OutputFormat::Records),
        std::time::Instant::now(),
    )
    .unwrap();
}

#[test]
fn test_execute_list_all_formats() {
    assert_execute_ok(execute_list);
}

#[test]
fn test_execute_print_all_formats() {
    assert_execute_ok(execute_print);
}

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
    let path = crate::commands::setup::tests::setup_cursor_rules(temp_dir.path());

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
fn test_execute_onboard_all_formats() {
    assert_execute_ok(execute_onboard);
}
