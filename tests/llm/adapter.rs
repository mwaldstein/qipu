use super::types::ToolAdapter;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

/// OpenCode tool adapter implementation
pub struct OpenCodeAdapter {
    name: String,
}

impl Default for OpenCodeAdapter {
    fn default() -> Self {
        Self::new()
    }
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
        writeln!(commands)?;

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
