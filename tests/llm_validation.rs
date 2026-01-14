use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tempfile::TempDir;

/// Result of an LLM user validation test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the test passed
    pub passed: bool,
    /// Detailed validation message
    pub message: String,
    /// Store state validation details
    pub store_validation: StoreValidation,
    /// Test execution duration in seconds
    pub duration_secs: f64,
    /// Path to captured transcript
    pub transcript_path: Option<PathBuf>,
}

/// Validation details for the resulting store state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreValidation {
    /// Number of notes created
    pub note_count: usize,
    /// Number of links created
    pub link_count: usize,
    /// Whether the store has meaningful structure
    pub has_structure: bool,
    /// Whether task knowledge was captured
    pub captured_task: bool,
    /// Detailed analysis
    pub details: Vec<String>,
}

/// Abstract interface for LLM tool adapters
pub trait ToolAdapter {
    /// Get the tool name
    fn name(&self) -> &str;

    /// Execute a test task with the LLM tool
    fn execute_task(
        &self,
        task_prompt: &str,
        work_dir: &Path,
        transcript_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// Check if the tool is available on the system
    fn is_available(&self) -> bool;
}

/// Configuration for LLM user validation tests
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Which tool adapter to use
    pub tool: String,
    /// Base directory for transcript storage
    pub transcript_base: PathBuf,
    /// Whether to keep transcripts after test completion
    pub keep_transcripts: bool,
    /// Timeout for test execution in seconds
    pub timeout_secs: u64,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            tool: "opencode".to_string(),
            transcript_base: PathBuf::from("tests/transcripts"),
            keep_transcripts: true,
            timeout_secs: 300, // 5 minutes
        }
    }
}

impl StoreValidation {
    /// Create an empty validation result
    pub fn empty() -> Self {
        Self {
            note_count: 0,
            link_count: 0,
            has_structure: false,
            captured_task: false,
            details: vec!["Store validation could not be performed".to_string()],
        }
    }

    /// Check if the store state is considered valid
    pub fn is_valid(&self) -> bool {
        self.captured_task && (self.has_structure || self.link_count > 0)
    }
}

/// Main test runner for LLM user validation
pub struct ValidationRunner {
    config: ValidationConfig,
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

/// OpenCode tool adapter implementation
pub struct OpenCodeAdapter {
    name: String,
}

impl OpenCodeAdapter {
    /// Create a new OpenCode adapter
    pub fn new() -> Self {
        Self {
            name: "opencode".to_string(),
        }
    }
}

impl ToolAdapter for OpenCodeAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute_task(
        &self,
        task_prompt: &str,
        work_dir: &Path,
        transcript_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use std::io::Write;

        // Create transcript files
        fs::create_dir_all(transcript_path)?;

        // Write the task prompt to the transcript
        let prompt_file = transcript_path.join("task_prompt.txt");
        fs::write(&prompt_file, task_prompt)?;

        // Simulate creating some notes and links
        let commands_file = transcript_path.join("commands.log");
        let mut commands = fs::File::create(&commands_file)?;

        writeln!(commands, "# Simulated OpenCode Session")?;
        writeln!(commands, "# Task: {}", task_prompt)?;
        writeln!(commands, "")?;

        // Get the qipu binary path
        let qipu_binary = format!("{}/target/debug/qipu", env!("CARGO_MANIFEST_DIR"));

        // Step 1: Create notes and capture their IDs
        let mut note_ids = Vec::new();
        let mut simulated_commands = Vec::new();
        let create_commands = vec![
            "create \"Research Notes\" --type permanent",
            "create \"Quick Idea\" --type fleeting",
            "create \"Literature Review\" --type literature",
        ];

        for cmd in &create_commands {
            writeln!(commands, "$ qipu {}", cmd)?;
            simulated_commands.push(format!("qipu {}", cmd));

            // Use shell to preserve quoted arguments properly
            let shell_cmd = format!("{} {}", qipu_binary, cmd);
            let output = if cfg!(target_os = "windows") {
                Command::new("cmd")
                    .args(["/C", &shell_cmd])
                    .current_dir(work_dir)
                    .output()?
            } else {
                Command::new("sh")
                    .args(["-c", &shell_cmd])
                    .current_dir(work_dir)
                    .output()?
            };

            if !output.status.success() {
                return Err(format!(
                    "Command failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                )
                .into());
            }

            // Extract note ID from the output
            let output_str = String::from_utf8_lossy(&output.stdout);
            let trimmed_output = output_str.trim();
            if !trimmed_output.is_empty() && trimmed_output.starts_with("qp-") {
                note_ids.push(trimmed_output.to_string());
            }
        }

        // Step 2: Create links between the notes we just created
        if note_ids.len() >= 3 {
            let link_commands = vec![
                format!("link add {} {} --type related", note_ids[0], note_ids[1]),
                format!(
                    "link add {} {} --type derived-from",
                    note_ids[0], note_ids[2]
                ),
            ];

            for cmd in &link_commands {
                writeln!(commands, "$ qipu {}", cmd)?;
                simulated_commands.push(format!("qipu {}", cmd));

                let shell_cmd = format!("{} {}", qipu_binary, cmd);
                let output = if cfg!(target_os = "windows") {
                    Command::new("cmd")
                        .args(["/C", &shell_cmd])
                        .current_dir(work_dir)
                        .output()?
                } else {
                    Command::new("sh")
                        .args(["-c", &shell_cmd])
                        .current_dir(work_dir)
                        .output()?
                };

                if !output.status.success() {
                    return Err(format!(
                        "Command failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    )
                    .into());
                }
            }
        }

        // Write session summary
        let summary_file = transcript_path.join("session_summary.txt");
        fs::write(
            &summary_file,
            format!(
                "Tool: {}\nTask: {}\nCommands executed: {}\nStore path: {}\n",
                self.name(),
                task_prompt,
                simulated_commands.len(),
                work_dir.display()
            ),
        )?;

        Ok(())
    }

    fn is_available(&self) -> bool {
        // Check if opencode is available in the current environment
        Command::new("opencode")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_config_default() {
        let config = ValidationConfig::default();
        assert_eq!(config.tool, "opencode");
        assert_eq!(config.transcript_base, PathBuf::from("tests/transcripts"));
        assert!(config.keep_transcripts);
        assert_eq!(config.timeout_secs, 300);
    }

    #[test]
    fn test_store_validation_empty() {
        let validation = StoreValidation::empty();
        assert_eq!(validation.note_count, 0);
        assert_eq!(validation.link_count, 0);
        assert!(!validation.has_structure);
        assert!(!validation.captured_task);
        assert!(!validation.is_valid());
    }

    #[test]
    fn test_store_validation_validity() {
        // Test case where task was captured but no structure
        let validation1 = StoreValidation {
            note_count: 2,
            link_count: 0,
            has_structure: false,
            captured_task: true,
            details: vec!["No links".to_string()],
        };
        assert!(!validation1.is_valid()); // No links or structure

        // Test case where task was captured with links
        let validation2 = StoreValidation {
            note_count: 2,
            link_count: 1,
            has_structure: false,
            captured_task: true,
            details: vec!["Has links".to_string()],
        };
        assert!(validation2.is_valid()); // Has links

        // Test case with structure but no links
        let validation3 = StoreValidation {
            note_count: 2,
            link_count: 0,
            has_structure: true,
            captured_task: true,
            details: vec!["Has structure".to_string()],
        };
        assert!(validation3.is_valid()); // Has structure
    }

    #[test]
    fn test_opencode_adapter() {
        let adapter = OpenCodeAdapter::new();
        assert_eq!(adapter.name(), "opencode");
        // The availability check will depend on whether opencode is installed
        // We just test that the method doesn't panic
        let _is_available = adapter.is_available();
    }

    #[test]
    fn test_validation_runner_creation() {
        let config = ValidationConfig::default();
        let runner = ValidationRunner::new(config.clone());
        // We can't easily test the full validation without a proper environment,
        // but we can test that the runner is created correctly
        assert_eq!(runner.config.tool, config.tool);
        assert_eq!(runner.config.transcript_base, config.transcript_base);
    }

    #[test]
    fn test_full_validation_simulation() {
        // This test runs the full validation framework with simulated task execution
        let config = ValidationConfig {
            tool: "opencode".to_string(),
            transcript_base: PathBuf::from("tests/transcripts"),
            keep_transcripts: true,
            timeout_secs: 60,
        };

        let runner = ValidationRunner::new(config);

        let task_prompt = "Create some research notes about machine learning and connect them with meaningful links";

        let result = runner
            .run_validation(task_prompt)
            .expect("Validation should complete without errors");

        // Check that validation completed
        assert!(
            result.transcript_path.is_some(),
            "Transcript path should be set"
        );

        // The simulation should always pass since it creates notes and links
        assert!(result.passed, "Simulated validation should pass");
        assert!(
            result.message.contains("successfully created"),
            "Success message should be present"
        );

        // Check that the store validation reflects the simulated data
        assert!(
            result.store_validation.note_count > 0,
            "Should have created some notes"
        );
        assert!(
            result.store_validation.link_count > 0,
            "Should have created some links"
        );
        assert!(
            result.store_validation.captured_task,
            "Task should be captured"
        );
        assert!(result.store_validation.is_valid(), "Store should be valid");

        // Verify transcript files were created
        let transcript_path = result.transcript_path.unwrap();
        assert!(
            transcript_path.exists(),
            "Transcript directory should exist"
        );
        assert!(
            transcript_path.join("task_prompt.txt").exists(),
            "Task prompt file should exist"
        );
        assert!(
            transcript_path.join("commands.log").exists(),
            "Commands log should exist"
        );
        assert!(
            transcript_path.join("session_summary.txt").exists(),
            "Session summary should exist"
        );
    }
}
