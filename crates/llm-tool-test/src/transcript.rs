use regex::Regex;
use serde::{Deserialize, Serialize};
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

pub struct TranscriptAnalyzer;

impl TranscriptAnalyzer {
    pub fn analyze(transcript: &str) -> EfficiencyMetrics {
        let command_regex = Regex::new(r"qipu\s+(\S+)").unwrap();
        let lines: Vec<&str> = transcript.lines().collect();

        let mut commands: Vec<(String, bool)> = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            if let Some(caps) = command_regex.captures(line) {
                let subcommand = caps[1].to_string();
                let is_help = subcommand == "--help" || line.contains("--help");
                let is_error = !is_help && i + 1 < lines.len() && Self::is_error_line(lines[i + 1]);

                if is_help {
                    commands.push(("help".to_string(), false));
                } else {
                    commands.push((subcommand, is_error));
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
}
