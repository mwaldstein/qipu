use crate::config::Config;
use std::fs;
use std::path::{Path, PathBuf};

pub fn copy_dir_recursive(src: &Path, dst: &Path) -> anyhow::Result<()> {
    copy_dir_recursive_with_exclusions(src, dst, &["scenarios"])
}

pub fn copy_dir_recursive_with_exclusions(
    src: &Path,
    dst: &Path,
    excluded_dirs: &[&str],
) -> anyhow::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(&file_name);
        if ty.is_dir() {
            let file_name_str = file_name.to_string_lossy();
            if !excluded_dirs.iter().any(|excl| file_name_str == *excl) {
                copy_dir_recursive_with_exclusions(&src_path, &dst_path, excluded_dirs)?;
            }
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

pub fn get_results_dir(tool: &str, model: &str, scenario_name: &str) -> PathBuf {
    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
    // Sanitize model name to avoid creating subdirectories from path separators
    let safe_model = model.replace(['/', '\\'], "_");
    let dir_name = format!("{}-{}-{}-{}", timestamp, tool, safe_model, scenario_name);
    let config = Config::load_or_default();
    let base_path = config.get_results_path();
    PathBuf::from(base_path).join(dir_name)
}
