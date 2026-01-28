use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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
        let templates_base = if Path::new("crates/llm-tool-test/fixtures/templates").exists() {
            Path::new("crates/llm-tool-test/fixtures/templates")
        } else {
            Path::new("fixtures/templates")
        };
        let fixture_src = templates_base.join(fixture_name);
        if !fixture_src.exists() {
            anyhow::bail!("Fixture not found: {:?}", fixture_src);
        }
        copy_dir_recursive(&fixture_src, &self.root)?;
        Ok(())
    }

    /// Run `qipu prime` in the test environment and return its output.
    /// Returns empty string if the command fails (e.g., no .qipu store yet).
    pub fn get_prime_output(&self) -> String {
        // Run qipu prime in the test environment directory
        let output = Command::new("qipu")
            .arg("prime")
            .current_dir(&self.root)
            .output();

        match output {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout).to_string()
            }
            _ => String::new(), // Return empty string if prime fails or store doesn't exist yet
        }
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
