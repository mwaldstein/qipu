use std::path::{Path, PathBuf};

pub fn resolve_fixtures_path(relative_path: &str) -> PathBuf {
    if Path::new(relative_path).is_absolute() || Path::new(relative_path).exists() {
        PathBuf::from(relative_path)
    } else {
        let long_path = PathBuf::from("crates/llm-tool-test/fixtures").join(relative_path);
        if long_path.exists() {
            long_path
        } else {
            PathBuf::from("fixtures").join(relative_path)
        }
    }
}
