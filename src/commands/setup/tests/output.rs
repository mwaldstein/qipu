use crate::cli::OutputFormat;
use crate::commands::setup::execute;
use crate::commands::setup::tests::{create_cli_with_root, setup_agents_md, setup_cursor_rules};
use tempfile::TempDir;

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
