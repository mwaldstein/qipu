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
