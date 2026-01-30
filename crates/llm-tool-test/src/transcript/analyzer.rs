use crate::transcript::types::{CommandEvent, EfficiencyMetrics};
use regex::Regex;

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

    pub(crate) fn extract_commands_with_exit_codes(transcript: &str) -> Vec<CommandEvent> {
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
