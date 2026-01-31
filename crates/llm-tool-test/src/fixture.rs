use crate::run::utils::copy_dir_recursive;
use crate::utils::resolve_fixtures_path;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub struct TestEnv {
    pub root: PathBuf,
}

impl TestEnv {
    pub fn new(root: PathBuf) -> anyhow::Result<Self> {
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    pub fn setup_fixture(&self, fixture_name: &str) -> anyhow::Result<()> {
        let templates_base = resolve_fixtures_path("templates");
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
