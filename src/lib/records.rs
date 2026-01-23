use std::path::Path;

/// Utilities for records output format
/// Escape double quotes in a string for records format.
/// Replaces `"` with `\"` to allow safe embedding in quoted fields.
pub fn escape_quotes(s: &str) -> String {
    s.replace('\"', r#"\""#)
}

/// Convert an absolute path to a path relative to the current working directory
pub fn path_relative_to_cwd(path: &Path) -> String {
    if let Ok(cwd) = std::env::current_dir() {
        path.strip_prefix(&cwd)
            .ok()
            .map(|p| {
                let s = p.display().to_string();
                if s.is_empty() {
                    ".".to_string()
                } else {
                    s
                }
            })
            .unwrap_or_else(|| path.display().to_string())
    } else {
        path.display().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_escape_quotes() {
        assert_eq!(escape_quotes("no quotes"), "no quotes");
        assert_eq!(escape_quotes(r#"has "quotes""#), r#"has \"quotes\""#);
        assert_eq!(
            escape_quotes(r#"multiple "quotes" in "text""#),
            r#"multiple \"quotes\" in \"text\""#
        );
        assert_eq!(escape_quotes(""), "");
        assert_eq!(escape_quotes(r#""""#), r#"\"\""#);
    }

    #[test]
    fn test_path_relative_to_cwd() {
        let Ok(cwd) = std::env::current_dir() else {
            // Skip test if current directory is not available (test isolation issue)
            return;
        };

        // Test path that's exactly the CWD
        assert_eq!(path_relative_to_cwd(&cwd), ".");

        // Test path that's a subdirectory of CWD
        let subdir = cwd.join("subdir");
        assert_eq!(path_relative_to_cwd(&subdir), "subdir");

        // Test path that's a nested subdirectory
        let nested = cwd.join("a").join("b").join("c");
        assert_eq!(path_relative_to_cwd(&nested), "a/b/c");

        // Test absolute path outside CWD (should return absolute path as fallback)
        let other = if cfg!(unix) {
            PathBuf::from("/some/other/path")
        } else {
            PathBuf::from("C:\\some\\other\\path")
        };
        let result = path_relative_to_cwd(&other);
        assert!(
            result.starts_with("/") || result.contains(":"),
            "Path outside CWD should return absolute path"
        );
    }
}
