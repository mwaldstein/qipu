//! Path resolution utilities for CLI commands
//!
//! Provides shared helpers for consistent path resolution across commands.

use std::env;
use std::path::PathBuf;

/// Resolve the root path for store discovery.
///
/// If a root path is provided, returns it. Otherwise, falls back to the
/// current working directory, or "." if that cannot be determined.
///
/// This is the standard pattern used across all qipu commands that need
/// to discover a store from the filesystem.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use qipu::cli::paths::resolve_root_path;
///
/// // With explicit path
/// let root = resolve_root_path(Some(PathBuf::from("/tmp/myproject")));
/// assert_eq!(root, PathBuf::from("/tmp/myproject"));
///
/// // Without path (falls back to current dir or ".")
/// let root = resolve_root_path(None);
/// // root will be current_dir() or PathBuf::from(".")
/// ```
pub fn resolve_root_path(root: Option<PathBuf>) -> PathBuf {
    root.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_resolve_with_explicit_path() {
        let explicit = PathBuf::from("/tmp/test/path");
        let result = resolve_root_path(Some(explicit.clone()));
        assert_eq!(result, explicit);
    }

    #[test]
    fn test_resolve_without_path_uses_current_dir() {
        // When no path is provided, should fall back to current_dir or "."
        let result = resolve_root_path(None);
        // Result should either be current_dir or "."
        if let Ok(current) = env::current_dir() {
            assert!(result == current || result == PathBuf::from("."));
        } else {
            assert_eq!(result, PathBuf::from("."));
        }
    }
}
