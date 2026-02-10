use crate::cli::OutputFormat;
use crate::commands::setup::tests::{
    assert_install_cursor_success, create_cli_with_root, setup_cursor_rules,
};
use crate::commands::setup::{execute_check_cursor, execute_install_cursor, execute_remove_cursor};
use std::fs;
use tempfile::TempDir;

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
