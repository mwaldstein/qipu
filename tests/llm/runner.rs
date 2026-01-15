use super::adapter::OpenCodeAdapter;
use super::types::{StoreValidation, ToolAdapter, ValidationConfig, ValidationResult};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::TempDir;

/// Main test runner for LLM user validation
pub struct ValidationRunner {
    pub(crate) config: ValidationConfig,
}

impl ValidationRunner {
    /// Create a new validation runner with the given config
    pub fn new(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Run the validation test
    pub fn run_validation(
        &self,
        task_prompt: &str,
    ) -> Result<ValidationResult, Box<dyn std::error::Error>> {
        let start_time = SystemTime::now();
        let transcript_path = self.create_transcript_dir()?;

        // Create a temporary directory for the test store
        let temp_dir = TempDir::new()?;
        let store_path = temp_dir.path();

        // Initialize a fresh qipu store
        self.init_store(store_path)?;

        // Get the appropriate tool adapter
        let adapter = self.get_tool_adapter()?;

        if !adapter.is_available() {
            return Ok(ValidationResult {
                passed: false,
                message: format!("Tool '{}' is not available", adapter.name()),
                store_validation: StoreValidation::empty(),
                duration_secs: 0.0,
                transcript_path: Some(transcript_path),
            });
        }

        // Execute the task with the LLM
        adapter.execute_task(task_prompt, store_path, &transcript_path)?;

        // Validate the resulting store state
        let store_validation = self.validate_store(store_path)?;

        let duration = start_time
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();

        let passed = store_validation.is_valid();
        let message = if passed {
            format!(
                "LLM successfully created a valid store with {} notes and {} links",
                store_validation.note_count, store_validation.link_count
            )
        } else {
            format!(
                "LLM failed to create a valid store structure: {}",
                store_validation.details.join("; ")
            )
        };

        Ok(ValidationResult {
            passed,
            message,
            store_validation,
            duration_secs: duration,
            transcript_path: Some(transcript_path),
        })
    }

    /// Create a timestamped directory for storing transcripts
    fn create_transcript_dir(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let transcript_dir = self
            .config
            .transcript_base
            .join(&self.config.tool)
            .join(timestamp.to_string());

        fs::create_dir_all(&transcript_dir)?;
        Ok(transcript_dir)
    }

    /// Initialize a fresh qipu store in the given directory
    fn init_store(&self, store_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // Build qipu first to ensure we have the binary
        let build_output = Command::new("cargo")
            .args(["build"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()?;

        if !build_output.status.success() {
            return Err(format!(
                "Failed to build qipu: {}",
                String::from_utf8_lossy(&build_output.stderr)
            )
            .into());
        }

        // Get the qipu binary path
        let qipu_binary = format!("{}/target/debug/qipu", env!("CARGO_MANIFEST_DIR"));

        // Initialize the store using the binary
        let output = Command::new(&qipu_binary)
            .args(["init", "--stealth"])
            .current_dir(store_path)
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to initialize store: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        Ok(())
    }

    /// Get the configured tool adapter
    fn get_tool_adapter(&self) -> Result<Box<dyn ToolAdapter>, Box<dyn std::error::Error>> {
        match self.config.tool.as_str() {
            "opencode" => Ok(Box::new(OpenCodeAdapter::new())),
            _ => Err(format!("Unknown tool adapter: {}", self.config.tool).into()),
        }
    }

    /// Validate the resulting store state
    fn validate_store(
        &self,
        store_path: &Path,
    ) -> Result<StoreValidation, Box<dyn std::error::Error>> {
        // Get the qipu binary path
        let qipu_binary = format!("{}/target/debug/qipu", env!("CARGO_MANIFEST_DIR"));

        // Run qipu list to get note information
        let output = Command::new(&qipu_binary)
            .args(["list", "--format", "json"])
            .current_dir(store_path)
            .output()?;

        if !output.status.success() {
            return Ok(StoreValidation::empty());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let notes: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap_or_default();

        let mut details = Vec::new();
        let note_count = notes.len();
        let mut link_count = 0;
        let mut has_structure = false;

        // Count links and check for structure by reading note files directly
        for note in &notes {
            if let Some(id) = note.get("id").and_then(|i| i.as_str()) {
                // Extract the actual filename from the JSON path
                let note_path = if let Some(path) = note.get("path").and_then(|p| p.as_str()) {
                    PathBuf::from(path)
                } else {
                    // Fallback to original logic
                    store_path
                        .join(".qipu")
                        .join("notes")
                        .join(format!("{}.md", id))
                };

                if note_path.exists() {
                    if let Ok(content) = fs::read_to_string(&note_path) {
                        // Simple check for frontmatter links
                        if content.contains("links:") {
                            // Count links in frontmatter
                            if let Some(links_start) = content.find("links:") {
                                if let Some(frontmatter_end) = content.find("\n---") {
                                    if links_start < frontmatter_end {
                                        let links_section = &content[links_start..frontmatter_end];
                                        // Count "- type:" occurrences as proxy for link count
                                        link_count += links_section.matches("- type:").count();
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(note_type) = note.get("type").and_then(|t| t.as_str()) {
                    if note_type == "moc" {
                        has_structure = true;
                        details.push("Found MOC note indicating organized structure".to_string());
                    }
                }
            }
        }

        // Basic validity checks
        let captured_task = note_count > 0; // At least some notes were created

        if note_count == 0 {
            details.push("No notes were created".to_string());
        }

        if note_count > 0 && link_count == 0 {
            details.push("Notes created but no links between them".to_string());
        }

        if note_count >= 3 && link_count >= 2 {
            has_structure = true;
            details.push("Good structure found with multiple notes and links".to_string());
        }

        Ok(StoreValidation {
            note_count,
            link_count,
            has_structure,
            captured_task,
            details,
        })
    }
}
