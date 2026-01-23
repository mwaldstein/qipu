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
    // Check if we are already on this branch to avoid unnecessary errors/work
    if let Ok(current) = current_branch(repo_path) {
        if current == branch_name {
            return Ok(());
        }
    }

    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("checkout")
        .arg(branch_name)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // If the branch doesn't exist, we might have been given a default like "master"
        // for a repo that hasn't been initialized yet.
        // We only fail if the branch SHOULD exist.
        if stderr.contains("did not match any file(s) known to git") {
            // Check if ANY branch exists. If not, this is a fresh repo and we don't need to checkout.
            let has_any_commits = Command::new("git")
                .arg("-C")
                .arg(repo_path)
                .arg("rev-parse")
                .arg("--verify")
                .arg("HEAD")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if !has_any_commits {
                return Ok(());
            }
        }

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
    let original_branch = current_branch(repo_path).ok();

    // Check if branch exists
    if branch_exists(repo_path, branch_name)? {
        // Switch to existing branch
        checkout_branch(repo_path, branch_name)?;
    } else {
        // Create new orphan branch (no shared history with main)
        create_orphan_branch(repo_path, branch_name)?;
    }

    Ok(original_branch.unwrap_or_else(|| "master".to_string()))
}

/// Add files to the staging area
pub fn add(repo_path: &Path, path_spec: &str) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("add")
        .arg(path_spec)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(QipuError::Other(format!(
            "Failed to add files '{}': {}",
            path_spec, stderr
        )));
    }

    Ok(())
}

/// Commit staged changes
pub fn commit(repo_path: &Path, message: &str) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("commit")
        .arg("-m")
        .arg(message)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(QipuError::Other(format!(
            "Failed to commit changes: {}",
            stderr
        )));
    }

    Ok(())
}

/// Push changes to a remote repository
pub fn push(repo_path: &Path, remote: &str, branch: &str) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("push")
        .arg(remote)
        .arg(branch)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(QipuError::Other(format!(
            "Failed to push to '{}/{}': {}",
            remote, branch, stderr
        )));
    }

    Ok(())
}

/// Check if there are any uncommitted changes (including untracked files)
pub fn has_changes(repo_path: &Path) -> Result<bool> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("status")
        .arg("--porcelain")
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(QipuError::Other(format!(
            "Failed to check git status: {}",
            stderr
        )));
    }

    Ok(!output.stdout.is_empty())
}

/// Get all note IDs from all git branches to avoid cross-branch collisions
///
/// This function searches all branches for note files and extracts their IDs
/// from the frontmatter. This provides collision avoidance for multi-branch workflows.
pub fn get_ids_from_all_branches(
    repo_path: &Path,
    store_subpath: &str,
) -> Result<std::collections::HashSet<String>> {
    use std::collections::HashSet;

    // First, check if we're in a git repo
    if !is_git_available() {
        return Ok(HashSet::new());
    }

    // Check if repo has any commits
    let has_commits = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("rev-parse")
        .arg("--verify")
        .arg("HEAD")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_commits {
        return Ok(HashSet::new());
    }

    let mut all_ids = HashSet::new();

    // Get all branches (local and remote)
    let branches_output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("branch")
        .arg("-a")
        .arg("--format=%(refname)")
        .output()?;

    if !branches_output.status.success() {
        // If we can't list branches, just return empty set (no additional protection)
        return Ok(HashSet::new());
    }

    let branches = String::from_utf8_lossy(&branches_output.stdout);

    // For each branch, list all note files and extract IDs
    for branch_ref in branches.lines() {
        let branch_ref = branch_ref.trim();
        if branch_ref.is_empty() {
            continue;
        }

        // List all markdown files in notes/ and mocs/ directories
        for dir in &["notes", "mocs"] {
            let path_pattern = format!("{}{}/**/*.md", store_subpath, dir);

            let ls_output = Command::new("git")
                .arg("-C")
                .arg(repo_path)
                .arg("ls-tree")
                .arg("-r")
                .arg("--name-only")
                .arg(branch_ref)
                .arg(&path_pattern)
                .output();

            if let Ok(output) = ls_output {
                if output.status.success() {
                    let files = String::from_utf8_lossy(&output.stdout);

                    // Extract IDs from filenames (format: <id>-<slug>.md)
                    for file_path in files.lines() {
                        let file_path = file_path.trim();
                        if file_path.is_empty() {
                            continue;
                        }

                        // Get filename from path
                        if let Some(filename) = file_path.rsplit('/').next() {
                            // Extract ID (format: qp-<hash>-<slug>.md or qp-<hash>.md)
                            if let Some(id_part) = filename.strip_suffix(".md") {
                                // Find the first hyphen after "qp-"
                                if let Some(rest) = id_part.strip_prefix("qp-") {
                                    // The ID is everything up to the next hyphen (if any)
                                    // or the entire string if no more hyphens
                                    let id = if let Some(pos) = rest.find('-') {
                                        &id_part[..pos + 3]
                                    } else {
                                        id_part
                                    };
                                    all_ids.insert(id.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(all_ids)
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
