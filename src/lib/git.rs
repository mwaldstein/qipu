//! Git integration for protected branch workflow
//!
//! Provides minimal git operations needed for the protected branch workflow
//! where qipu notes can live on a separate git branch.

use std::path::Path;
use std::process::Command;

use crate::lib::error::{QipuError, Result};

/// Check if git is available on the system
pub fn is_git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Get the current git branch name
pub fn current_branch(repo_path: &Path) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output()?;

    if !output.status.success() {
        // If the repo has no commits yet, rev-parse will fail
        // In this case, try to get the symbolic-ref (default branch)
        let symbolic_output = Command::new("git")
            .arg("-C")
            .arg(repo_path)
            .arg("symbolic-ref")
            .arg("--short")
            .arg("HEAD")
            .output()?;

        if symbolic_output.status.success() {
            let branch = String::from_utf8_lossy(&symbolic_output.stdout)
                .trim()
                .to_string();
            return Ok(branch);
        }

        // If both fail, return the original error
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(QipuError::Other(format!(
            "Failed to get current branch: {}",
            stderr
        )));
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(branch)
}

/// Check if a branch exists
pub fn branch_exists(repo_path: &Path, branch_name: &str) -> Result<bool> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("rev-parse")
        .arg("--verify")
        .arg(branch_name)
        .output()?;

    Ok(output.status.success())
}

/// Create a new branch (orphan branch with no history)
pub fn create_orphan_branch(repo_path: &Path, branch_name: &str) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("checkout")
        .arg("--orphan")
        .arg(branch_name)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(QipuError::Other(format!(
            "Failed to create branch '{}': {}",
            branch_name, stderr
        )));
    }

    Ok(())
}

/// Switch to an existing branch
pub fn checkout_branch(repo_path: &Path, branch_name: &str) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("checkout")
        .arg(branch_name)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(QipuError::Other(format!(
            "Failed to checkout branch '{}': {}",
            branch_name, stderr
        )));
    }

    Ok(())
}

/// Initialize the protected branch workflow
///
/// This function:
/// 1. Saves the current branch
/// 2. Creates or switches to the qipu branch
/// 3. Returns the original branch name so caller can switch back
pub fn setup_branch_workflow(repo_path: &Path, branch_name: &str) -> Result<String> {
    // Save current branch
    let original_branch = current_branch(repo_path)?;

    // Check if branch exists
    if branch_exists(repo_path, branch_name)? {
        // Switch to existing branch
        checkout_branch(repo_path, branch_name)?;
    } else {
        // Create new orphan branch (no shared history with main)
        create_orphan_branch(repo_path, branch_name)?;
    }

    Ok(original_branch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_availability() {
        // This test might fail in environments without git,
        // but that's expected behavior
        let available = is_git_available();
        // Just verify the function doesn't panic
        let _ = available;
    }
}
