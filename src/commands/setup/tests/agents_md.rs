use crate::cli::OutputFormat;
use crate::commands::setup::tests::{
    assert_install_agents_md_success, create_cli_with_root, setup_agents_md,
};
use crate::commands::setup::{
    execute_check_agents_md, execute_install_agents_md, execute_remove_agents_md,
};
use std::fs;
use tempfile::TempDir;

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
