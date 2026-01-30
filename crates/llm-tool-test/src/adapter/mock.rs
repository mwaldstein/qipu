use super::ToolAdapter;
use crate::scenario::{Gate, Scenario};
use crate::session::SessionRunner;
use std::path::Path;

pub struct MockAdapter;

impl MockAdapter {
    pub fn generate_transcript(&self, scenario: &Scenario) -> String {
        let mut commands = Vec::new();

        for gate in &scenario.evaluation.gates {
            match gate {
                Gate::MinNotes { count } => {
                    for i in 0..*count {
                        commands.push(format!("qipu create 'Mock Note {}'", i + 1));
                    }
                }
                Gate::NoteExists { id } => {
                    commands.push(format!("qipu create --id {} 'Note {}'", id, id));
                }
                Gate::LinkExists {
                    from,
                    to,
                    link_type,
                } => {
                    commands.push(format!(
                        "qipu link add --type {} {} {}",
                        link_type, from, to
                    ));
                }
                Gate::SearchHit { query } => {
                    commands.push(format!("qipu create 'Search Result - {}'", query));
                }
                Gate::TagExists { tag } => {
                    commands.push(format!("qipu create --tag {} 'Tagged Note'", tag));
                }
                Gate::ContentContains { id, substring } => {
                    commands.push(format!(
                        "qipu create --id {} 'Content Note - {}'",
                        id, substring
                    ));
                }
                Gate::CommandSucceeds { command } => {
                    commands.push(format!("qipu {}", command));
                }
                #[allow(clippy::if_same_then_else)]
                Gate::MinLinks { count } => {
                    // Create notes with links to satisfy the minimum link count
                    // Strategy: Create count+1 notes, where each note (except the first)
                    // links to the previous note, resulting in exactly 'count' links
                    for i in 0..=*count {
                        if i == 0 {
                            commands.push(format!(
                                "qipu create --id mock-link-{} 'Link Node {}'",
                                i, i
                            ));
                        } else {
                            commands.push(format!(
                                "qipu create --id mock-link-{} 'Link Node {}'",
                                i, i
                            ));
                        }
                    }
                    // Create links between notes
                    for i in 1..=*count {
                        commands.push(format!(
                            "qipu link add --type related mock-link-{} mock-link-{}",
                            i,
                            i - 1
                        ));
                    }
                }
                Gate::DoctorPasses => {
                    // Doctor check is automatic, no specific command needed
                }
                Gate::NoTranscriptErrors => {
                    // Transcript error checking is automatic, no specific command needed
                }
            }
        }

        if commands.is_empty() {
            commands.push("qipu list".to_string());
        }

        commands.join("\n")
    }
}

impl ToolAdapter for MockAdapter {
    fn is_available(&self) -> Result<super::ToolStatus, super::AdapterError> {
        Ok(super::ToolStatus {
            available: true,
            authenticated: true,
        })
    }

    fn check_availability(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn run(
        &self,
        scenario: &Scenario,
        cwd: &Path,
        _model: Option<&str>,
        timeout_secs: u64,
    ) -> anyhow::Result<(String, i32, Option<f64>, Option<super::TokenUsage>)> {
        let runner = SessionRunner::new();

        let transcript = self.generate_transcript(scenario);
        let mut full_output = String::new();

        let commands: Vec<&str> = transcript.lines().collect();
        let mut exit_code = 0;

        let (init_output, init_code) = runner.run_command("qipu", &["init"], cwd, timeout_secs)?;
        full_output.push_str("qipu init");
        if !init_output.is_empty() {
            full_output.push('\n');
            full_output.push_str(&init_output);
        }
        if init_code != 0 && exit_code == 0 {
            exit_code = init_code;
        }

        for (i, command) in commands.iter().enumerate() {
            let parts: Vec<String> = shlex::split(command).unwrap_or_default();
            if parts.is_empty() || !parts[0].starts_with("qipu") {
                continue;
            }

            let cmd_name = &parts[0];
            let args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();

            let (output, code) = runner.run_command(cmd_name, &args, cwd, timeout_secs)?;

            if i > 0 {
                full_output.push('\n');
            }
            full_output.push_str(command);
            if !output.is_empty() {
                full_output.push('\n');
                full_output.push_str(&output);
            }

            if code != 0 && exit_code == 0 {
                exit_code = code;
            }
        }

        Ok((full_output, exit_code, None, None))
    }
}
