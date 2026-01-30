use std::fs;
use std::path::{Path, PathBuf};

pub fn copy_dir_recursive(src: &Path, dst: &Path) -> anyhow::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

pub fn get_results_dir(tool: &str, model: &str, scenario_name: &str) -> PathBuf {
    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let dir_name = format!("{}-{}-{}-{}", timestamp, tool, model, scenario_name);
    PathBuf::from("llm-tool-test-results").join(dir_name)
}
