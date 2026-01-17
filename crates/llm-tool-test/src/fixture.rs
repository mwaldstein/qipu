use std::fs;
use std::path::{Path, PathBuf};

pub struct TestEnv {
    pub root: PathBuf,
}

impl TestEnv {
    pub fn new(scenario_name: &str) -> anyhow::Result<Self> {
        // Use a timestamp or unique ID to avoid conflicts if running in parallel,
        // but for now, deterministic path is fine for debugging.
        let root = Path::new("target/llm_test_runs").join(scenario_name);
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    pub fn setup_fixture(&self, fixture_name: &str) -> anyhow::Result<()> {
        let fixture_src = Path::new("crates/llm-tool-test/fixtures").join(fixture_name);
        if !fixture_src.exists() {
            anyhow::bail!("Fixture not found: {:?}", fixture_src);
        }
        copy_dir_recursive(&fixture_src, &self.root)?;
        Ok(())
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> anyhow::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            // Don't copy scenarios into the test environment, they are meta-data
            if entry.file_name() != "scenarios" {
                copy_dir_recursive(&src_path, &dst_path)?;
            }
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
