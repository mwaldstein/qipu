use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub struct TranscriptWriter {
    pub base_dir: PathBuf,
}

impl TranscriptWriter {
    pub fn new(base_dir: PathBuf) -> anyhow::Result<Self> {
        if !base_dir.exists() {
            fs::create_dir_all(&base_dir)?;
        }
        Ok(Self { base_dir })
    }

    pub fn write_raw(&self, content: &str) -> anyhow::Result<()> {
        fs::write(self.base_dir.join("transcript.raw.txt"), content)?;
        Ok(())
    }

    pub fn append_event(&self, event: &serde_json::Value) -> anyhow::Result<()> {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.base_dir.join("events.jsonl"))?;
        writeln!(file, "{}", serde_json::to_string(event)?)?;
        Ok(())
    }

    /// Log a tool_call event (when an LLM tool invokes a command)
    pub fn log_tool_call(&self, tool: &str, command: &str) -> anyhow::Result<()> {
        let event = json!({
            "ts": Self::timestamp(),
            "event": "tool_call",
            "tool": tool,
            "command": command,
        });
        self.append_event(&event)
    }

    /// Log a tool_result event (when a command completes)
    pub fn log_tool_result(&self, output: &str, exit_code: i32) -> anyhow::Result<()> {
        let event = json!({
            "ts": Self::timestamp(),
            "event": "tool_result",
            "output": output,
            "exit_code": exit_code,
        });
        self.append_event(&event)
    }

    /// Log a spawn event (when a command is spawned)
    pub fn log_spawn(&self, command: &str, args: &[String]) -> anyhow::Result<()> {
        let event = json!({
            "ts": Self::timestamp(),
            "event": "spawn",
            "command": command,
            "args": args,
        });
        self.append_event(&event)
    }

    /// Log an output event (general output text)
    pub fn log_output(&self, text: &str) -> anyhow::Result<()> {
        let event = json!({
            "ts": Self::timestamp(),
            "event": "output",
            "text": text,
        });
        self.append_event(&event)
    }

    /// Log a complete event (when the session completes)
    pub fn log_complete(&self, exit_code: i32, duration_secs: f64) -> anyhow::Result<()> {
        let event = json!({
            "ts": Self::timestamp(),
            "event": "complete",
            "exit_code": exit_code,
            "duration_secs": duration_secs,
        });
        self.append_event(&event)
    }

    fn timestamp() -> f64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64()
    }

    pub fn read_events(&self) -> anyhow::Result<Vec<serde_json::Value>> {
        let path = self.base_dir.join("events.jsonl");
        if !path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&path)?;
        let mut events = Vec::new();
        for line in content.lines() {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(line) {
                events.push(value);
            }
        }
        Ok(events)
    }

    /// Write run.json with run metadata
    pub fn write_run_metadata(&self, metadata: &RunMetadata) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(metadata)?;
        fs::write(self.base_dir.join("run.json"), json)?;
        Ok(())
    }

    /// Create store snapshot by copying .qipu/ and running qipu dump
    pub fn create_store_snapshot(&self, work_dir: &std::path::Path) -> anyhow::Result<()> {
        let snapshot_dir = self.base_dir.join("store_snapshot");
        fs::create_dir_all(&snapshot_dir)?;

        let qipu_dir = work_dir.join(".qipu");

        // Copy .qipu/ directory if it exists
        if qipu_dir.exists() {
            let snapshot_qipu_dir = snapshot_dir.join(".qipu");
            self.copy_dir(&qipu_dir, &snapshot_qipu_dir)?;
        }

        // Run qipu dump --format json to export the store
        let output = std::process::Command::new("qipu")
            .arg("dump")
            .arg("--format")
            .arg("json")
            .current_dir(work_dir)
            .output();

        match output {
            Ok(result) if result.status.success() => {
                fs::write(snapshot_dir.join("export.json"), result.stdout)?;
                Ok(())
            }
            Ok(result) => {
                // Store failed, but don't fail the entire run
                eprintln!(
                    "Warning: Failed to create store snapshot: {}",
                    String::from_utf8_lossy(&result.stderr)
                );
                Ok(())
            }
            Err(e) => {
                eprintln!("Warning: Failed to run qipu dump: {}", e);
                Ok(())
            }
        }
    }

    /// Copy directory recursively
    fn copy_dir(&self, src: &PathBuf, dst: &PathBuf) -> anyhow::Result<()> {
        fs::create_dir_all(dst)?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if file_type.is_file() {
                fs::copy(&src_path, &dst_path)?;
            } else if file_type.is_dir() {
                self.copy_dir(&src_path, &dst_path)?;
            }
        }

        Ok(())
    }

    /// Write report.md with human-readable summary
    pub fn write_report(&self, report: &RunReport) -> anyhow::Result<()> {
        let mut content = String::new();
        content.push_str(&format!("# Test Run Report\n\n"));
        content.push_str(&format!("## Scenario\n\n"));
        content.push_str(&format!("- **ID**: {}\n", report.scenario_id));
        content.push_str(&format!("- **Tool**: {}\n", report.tool));
        content.push_str(&format!("- **Model**: {}\n", report.model));
        content.push_str(&format!("- **Timestamp**: {}\n\n", report.timestamp));

        content.push_str(&format!("## Execution\n\n"));
        content.push_str(&format!("- **Duration**: {:.2}s\n", report.duration_secs));
        content.push_str(&format!("- **Cost**: ${:.4}\n", report.cost_usd));
        if let Some(ref usage) = report.token_usage {
            content.push_str(&format!(
                "- **Token Usage**: {} input, {} output\n",
                usage.input, usage.output
            ));
        }
        content.push_str(&format!("- **Outcome**: {}\n\n", report.outcome));

        content.push_str(&format!("## Evaluation Metrics\n\n"));
        content.push_str(&format!(
            "- **Gates Passed**: {}/{}\n",
            report.gates_passed, report.gates_total
        ));
        content.push_str(&format!("- **Notes Created**: {}\n", report.note_count));
        content.push_str(&format!("- **Links Created**: {}\n", report.link_count));
        if let Some(score) = report.composite_score {
            content.push_str(&format!("- **Composite Score**: {:.2}\n", score));
        }
        content.push_str("\n");

        if !report.gate_details.is_empty() {
            content.push_str("### Gate Details\n\n");
            for detail in &report.gate_details {
                let status = if detail.passed { "✓" } else { "✗" };
                content.push_str(&format!(
                    "- {} {}: {}\n",
                    status, detail.gate_type, detail.message
                ));
            }
            content.push_str("\n");
        }

        content.push_str(&format!("## Efficiency\n\n"));
        content.push_str(&format!(
            "- **Total Commands**: {}\n",
            report.efficiency.total_commands
        ));
        content.push_str(&format!(
            "- **Unique Commands**: {}\n",
            report.efficiency.unique_commands
        ));
        content.push_str(&format!(
            "- **Error Count**: {}\n",
            report.efficiency.error_count
        ));
        content.push_str(&format!(
            "- **First Try Success Rate**: {:.1}%\n",
            report.efficiency.first_try_success_rate * 100.0
        ));
        content.push_str(&format!(
            "- **Iteration Ratio**: {:.2}\n\n",
            report.efficiency.iteration_ratio
        ));

        content.push_str(&format!("## Quality\n\n"));
        content.push_str(&format!(
            "- **Average Title Length**: {:.1}\n",
            report.quality.avg_title_length
        ));
        content.push_str(&format!(
            "- **Average Body Length**: {:.1}\n",
            report.quality.avg_body_length
        ));
        content.push_str(&format!(
            "- **Average Tags per Note**: {:.2}\n",
            report.quality.avg_tags_per_note
        ));
        content.push_str(&format!(
            "- **Links per Note**: {:.2}\n",
            report.quality.links_per_note
        ));
        content.push_str(&format!(
            "- **Orphan Notes**: {}\n",
            report.quality.orphan_notes
        ));

        fs::write(self.base_dir.join("report.md"), content)?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunMetadata {
    pub scenario_id: String,
    pub scenario_hash: String,
    pub tool: String,
    pub model: String,
    pub qipu_version: String,
    pub qipu_commit: String,
    pub timestamp: String,
    pub duration_secs: f64,
    pub cost_estimate_usd: f64,
    pub token_usage: Option<TokenUsage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input: usize,
    pub output: usize,
}

#[derive(Debug)]
pub struct RunReport {
    pub scenario_id: String,
    pub tool: String,
    pub model: String,
    pub timestamp: String,
    pub duration_secs: f64,
    pub cost_usd: f64,
    pub token_usage: Option<TokenUsage>,
    pub outcome: String,
    pub gates_passed: usize,
    pub gates_total: usize,
    pub note_count: usize,
    pub link_count: usize,
    pub composite_score: Option<f64>,
    pub gate_details: Vec<GateDetail>,
    pub efficiency: EfficiencyReport,
    pub quality: QualityReport,
}

#[derive(Debug)]
pub struct GateDetail {
    pub gate_type: String,
    pub passed: bool,
    pub message: String,
}

#[derive(Debug)]
pub struct EfficiencyReport {
    pub total_commands: usize,
    pub unique_commands: usize,
    pub error_count: usize,
    pub first_try_success_rate: f64,
    pub iteration_ratio: f64,
}

#[derive(Debug)]
pub struct QualityReport {
    pub avg_title_length: f64,
    pub avg_body_length: f64,
    pub avg_tags_per_note: f64,
    pub links_per_note: f64,
    pub orphan_notes: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EfficiencyMetrics {
    pub total_commands: usize,
    pub unique_commands: usize,
    pub error_count: usize,
    pub retry_count: usize,
    pub help_invocations: usize,
    pub first_try_success_rate: f64,
    pub iteration_ratio: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandEvent {
    pub command: String,
    pub exit_code: Option<i32>,
}

pub struct TranscriptAnalyzer;

impl TranscriptAnalyzer {
    pub fn analyze(transcript: &str) -> EfficiencyMetrics {
        Self::analyze_with_events(transcript, None)
    }

    pub fn analyze_with_exit_codes(transcript: &str) -> EfficiencyMetrics {
        let commands = Self::extract_commands_with_exit_codes(transcript);
        Self::analyze_with_events(transcript, Some(commands))
    }

    pub fn analyze_with_events(
        transcript: &str,
        events: Option<Vec<CommandEvent>>,
    ) -> EfficiencyMetrics {
        let command_regex = Regex::new(r"qipu\s+(\S+)").unwrap();
        let lines: Vec<&str> = transcript.lines().collect();

        let mut commands: Vec<(String, bool)> = Vec::new();

        if let Some(command_events) = events {
            for event in command_events {
                let is_error = event.exit_code.map(|code| code != 0).unwrap_or(false);
                commands.push((event.command, is_error));
            }
        } else {
            for (i, line) in lines.iter().enumerate() {
                if let Some(caps) = command_regex.captures(line) {
                    let subcommand = caps[1].to_string();
                    let is_help = subcommand == "--help" || line.contains("--help");
                    let is_error =
                        !is_help && i + 1 < lines.len() && Self::is_error_line(lines[i + 1]);

                    if is_help {
                        commands.push(("help".to_string(), false));
                    } else {
                        commands.push((subcommand, is_error));
                    }
                }
            }
        }

        let total_commands = commands.len();
        let error_count = commands.iter().filter(|(_, e)| *e).count();
        let help_invocations = commands.iter().filter(|(c, _)| c == "help").count();

        let unique_commands: std::collections::HashSet<_> =
            commands.iter().map(|(c, _)| c.clone()).collect();
        let retry_count = total_commands.saturating_sub(unique_commands.len());

        let first_try_success_count = commands
            .iter()
            .filter(|(cmd, _)| {
                commands.iter().take_while(|(c, _)| c != cmd).count()
                    == commands.iter().position(|(c, _)| c == cmd).unwrap_or(0)
                    && !commands
                        .iter()
                        .take_while(|(c, _)| c != cmd)
                        .any(|(_, e)| *e)
            })
            .count();

        let first_try_success_rate = if total_commands > 0 {
            first_try_success_count as f64 / total_commands as f64
        } else {
            0.0
        };

        let iteration_ratio = if unique_commands.len() > 0 {
            total_commands as f64 / unique_commands.len() as f64
        } else {
            0.0
        };

        EfficiencyMetrics {
            total_commands,
            unique_commands: unique_commands.len(),
            error_count,
            retry_count,
            help_invocations,
            first_try_success_rate,
            iteration_ratio,
        }
    }

    fn is_error_line(line: &str) -> bool {
        let line_lower = line.to_lowercase();
        line_lower.contains("error")
            || line_lower.contains("failed")
            || line_lower.contains("exit code")
            || line_lower.contains("non-zero")
    }

    fn extract_commands_with_exit_codes(transcript: &str) -> Vec<CommandEvent> {
        let command_regex = Regex::new(r"qipu\s+(\S+)").unwrap();
        let exit_code_regex = Regex::new(r"(?i)exit\s+(?:code|status):?\s*(\d+)").unwrap();

        let lines: Vec<&str> = transcript.lines().collect();
        let mut commands = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            if let Some(caps) = command_regex.captures(line) {
                let subcommand = caps[1].to_string();
                let is_help = subcommand == "--help" || line.contains("--help");

                if is_help {
                    commands.push(CommandEvent {
                        command: "help".to_string(),
                        exit_code: Some(0),
                    });
                } else {
                    let next_lines: Vec<&str> = lines[i + 1..].iter().take(20).cloned().collect();
                    let joined = next_lines.join("\n");

                    let exit_code = if let Some(exit_caps) = exit_code_regex.captures(&joined) {
                        exit_caps[1].parse().unwrap_or(-1)
                    } else if Self::is_error_line(&joined) {
                        1
                    } else {
                        0
                    };

                    commands.push(CommandEvent {
                        command: subcommand,
                        exit_code: Some(exit_code),
                    });
                }
            }
        }

        commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_empty_transcript() {
        let transcript = "";
        let metrics = TranscriptAnalyzer::analyze(transcript);

        assert_eq!(metrics.total_commands, 0);
        assert_eq!(metrics.unique_commands, 0);
        assert_eq!(metrics.error_count, 0);
        assert_eq!(metrics.retry_count, 0);
        assert_eq!(metrics.help_invocations, 0);
        assert_eq!(metrics.first_try_success_rate, 0.0);
        assert_eq!(metrics.iteration_ratio, 0.0);
    }

    #[test]
    fn test_analyze_single_command() {
        let transcript = "qipu create --title 'Test Note'";
        let metrics = TranscriptAnalyzer::analyze(transcript);

        assert_eq!(metrics.total_commands, 1);
        assert_eq!(metrics.unique_commands, 1);
        assert_eq!(metrics.error_count, 0);
        assert_eq!(metrics.retry_count, 0);
        assert_eq!(metrics.first_try_success_rate, 1.0);
        assert_eq!(metrics.iteration_ratio, 1.0);
    }

    #[test]
    fn test_analyze_multiple_commands() {
        let transcript = "qipu create --title 'Test 1'\nqipu create --title 'Test 2'\nqipu list";
        let metrics = TranscriptAnalyzer::analyze(transcript);

        assert_eq!(metrics.total_commands, 3);
        assert_eq!(metrics.unique_commands, 2);
        assert_eq!(metrics.retry_count, 1);
    }

    #[test]
    fn test_analyze_with_errors() {
        let transcript =
            "qipu create --title 'Test 1'\nError: command failed\nqipu create --title 'Test 1'";
        let metrics = TranscriptAnalyzer::analyze(transcript);

        assert_eq!(metrics.total_commands, 2);
        assert_eq!(metrics.error_count, 1);
    }

    #[test]
    fn test_analyze_help_invocations() {
        let transcript = "qipu --help\nqipu create --title 'Test'\nqipu list --help";
        let metrics = TranscriptAnalyzer::analyze(transcript);

        assert_eq!(metrics.total_commands, 3);
        assert_eq!(metrics.help_invocations, 2);
    }

    #[test]
    fn test_iteration_ratio() {
        let transcript = "qipu create\nqipu create\nqipu create\nqipu list\nqipu list";
        let metrics = TranscriptAnalyzer::analyze(transcript);

        assert_eq!(metrics.total_commands, 5);
        assert_eq!(metrics.unique_commands, 2);
        assert_eq!(metrics.retry_count, 3);
        assert_eq!(metrics.iteration_ratio, 2.5);
    }

    #[test]
    fn test_extract_commands_basic() {
        let transcript = "qipu create --title 'Test'\nqipu list\nqipu link --from a --to b";
        let commands = TranscriptAnalyzer::extract_commands_with_exit_codes(transcript);

        assert_eq!(commands.len(), 3);
        assert_eq!(commands[0].command, "create");
        assert_eq!(commands[0].exit_code, Some(0));
        assert_eq!(commands[1].command, "list");
        assert_eq!(commands[1].exit_code, Some(0));
        assert_eq!(commands[2].command, "link");
        assert_eq!(commands[2].exit_code, Some(0));
    }

    #[test]
    fn test_extract_commands_with_explicit_exit_code() {
        let transcript = "qipu create --title 'Test'\nExit Code: 0\nqipu invalid\nExit status: 1";
        let commands = TranscriptAnalyzer::extract_commands_with_exit_codes(transcript);

        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].command, "create");
        assert_eq!(commands[0].exit_code, Some(0));
        assert_eq!(commands[1].command, "invalid");
        assert_eq!(commands[1].exit_code, Some(1));
    }

    #[test]
    fn test_extract_commands_with_implicit_error() {
        let transcript =
            "qipu create --title 'Test'\nError: something failed\nqipu create --title 'Test'";
        let commands = TranscriptAnalyzer::extract_commands_with_exit_codes(transcript);

        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].command, "create");
        assert_eq!(commands[0].exit_code, Some(1));
        assert_eq!(commands[1].command, "create");
        assert_eq!(commands[1].exit_code, Some(0));
    }

    #[test]
    fn test_extract_commands_help_detection() {
        let transcript = "qipu --help\nqipu create --help\nqipu list";
        let commands = TranscriptAnalyzer::extract_commands_with_exit_codes(transcript);

        assert_eq!(commands.len(), 3);
        assert_eq!(commands[0].command, "help");
        assert_eq!(commands[0].exit_code, Some(0));
        assert_eq!(commands[1].command, "help");
        assert_eq!(commands[1].exit_code, Some(0));
        assert_eq!(commands[2].command, "list");
        assert_eq!(commands[2].exit_code, Some(0));
    }

    #[test]
    fn test_extract_commands_various_exit_code_formats() {
        let transcript =
            "qipu create\nexit code: 0\nqipu delete\nExit Status: 127\nqipu search\nexit code 255";
        let commands = TranscriptAnalyzer::extract_commands_with_exit_codes(transcript);

        assert_eq!(commands.len(), 3);
        assert_eq!(commands[0].exit_code, Some(0));
        assert_eq!(commands[1].exit_code, Some(127));
        assert_eq!(commands[2].exit_code, Some(255));
    }

    #[test]
    fn test_extract_commands_empty_transcript() {
        let transcript = "";
        let commands = TranscriptAnalyzer::extract_commands_with_exit_codes(transcript);

        assert_eq!(commands.len(), 0);
    }

    #[test]
    fn test_extract_commands_no_matching_commands() {
        let transcript = "Some random text\nWithout commands\nJust output";
        let commands = TranscriptAnalyzer::extract_commands_with_exit_codes(transcript);

        assert_eq!(commands.len(), 0);
    }

    #[test]
    fn test_extract_commands_mixed_with_output() {
        let transcript = "Starting session...\nqipu create --title 'Test'\nNote created successfully\nqipu list\nList output\nDone";
        let commands = TranscriptAnalyzer::extract_commands_with_exit_codes(transcript);

        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].command, "create");
        assert_eq!(commands[1].command, "list");
    }

    #[test]
    fn test_extract_commands_case_insensitive_exit() {
        let transcript =
            "qipu create\nEXIT CODE: 0\nqipu delete\nexit code: 1\nqipu search\nExit Code: 2";
        let commands = TranscriptAnalyzer::extract_commands_with_exit_codes(transcript);

        assert_eq!(commands.len(), 3);
        assert_eq!(commands[0].exit_code, Some(0));
        assert_eq!(commands[1].exit_code, Some(1));
        assert_eq!(commands[2].exit_code, Some(2));
    }

    #[test]
    fn test_extract_commands_with_multiple_errors_keywords() {
        let transcript =
            "qipu create\nERROR: invalid input\nqipu delete\nFailed: not found\nqipu search";
        let commands = TranscriptAnalyzer::extract_commands_with_exit_codes(transcript);

        assert_eq!(commands.len(), 3);
        assert_eq!(commands[0].exit_code, Some(1));
        assert_eq!(commands[1].exit_code, Some(1));
        assert_eq!(commands[2].exit_code, Some(0));
    }

    #[test]
    fn test_extract_commands_nonzero_exit_code() {
        let transcript = "qipu create\nExit code: 130";
        let commands = TranscriptAnalyzer::extract_commands_with_exit_codes(transcript);

        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].command, "create");
        assert_eq!(commands[0].exit_code, Some(130));
    }

    #[test]
    fn test_extract_commands_large_exit_code() {
        let transcript = "qipu create\nExit code: 255";
        let commands = TranscriptAnalyzer::extract_commands_with_exit_codes(transcript);

        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].command, "create");
        assert_eq!(commands[0].exit_code, Some(255));
    }

    #[test]
    fn test_extract_commands_exit_code_takes_precedence() {
        let transcript = "qipu create\nExit code: 0\nqipu delete\nError: failed\nExit code: 1";
        let commands = TranscriptAnalyzer::extract_commands_with_exit_codes(transcript);

        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].exit_code, Some(0));
        assert_eq!(commands[1].exit_code, Some(1));
    }

    #[test]
    fn test_extract_commands_subcommand_with_flags() {
        let transcript = "qipu create --title 'Test' --tag work\nqipu list --format json\nqipu link --from a --to b --type reference";
        let commands = TranscriptAnalyzer::extract_commands_with_exit_codes(transcript);

        assert_eq!(commands.len(), 3);
        assert_eq!(commands[0].command, "create");
        assert_eq!(commands[1].command, "list");
        assert_eq!(commands[2].command, "link");
    }
}
