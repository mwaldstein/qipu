use super::ToolAdapter;
use crate::scenario::Scenario;
use crate::session::SessionRunner;
use serde_json::Value;
use std::path::Path;

pub struct OpenCodeAdapter;

fn extract_json_lines(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|line| line.starts_with('{'))
        .collect()
}

fn is_step_finish_event(json: &Value) -> bool {
    json.get("type") == Some(&Value::String("step_finish".to_string()))
}

fn extract_tokens_from_event(json: &Value) -> Option<(u64, u64)> {
    let tokens = json.get("part").and_then(|p| p.get("tokens"))?;
    let input = tokens.get("input").and_then(|v| v.as_u64()).unwrap_or(0);
    let output = tokens.get("output").and_then(|v| v.as_u64()).unwrap_or(0);
    let reasoning = tokens
        .get("reasoning")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    Some((input + reasoning, output))
}

fn accumulate_token_usage(lines: &[&str]) -> (u64, u64) {
    let mut total_input = 0u64;
    let mut total_output = 0u64;

    for line in lines {
        if let Ok(json) = serde_json::from_str::<Value>(line) {
            if is_step_finish_event(&json) {
                if let Some((input, output)) = extract_tokens_from_event(&json) {
                    total_input += input;
                    total_output += output;
                }
            }
        }
    }

    (total_input, total_output)
}

fn parse_token_usage_from_json(output: &str) -> Option<super::TokenUsage> {
    let lines = extract_json_lines(output);
    let (total_input, total_output) = accumulate_token_usage(&lines);

    if total_input > 0 || total_output > 0 {
        Some(super::TokenUsage {
            input: total_input as usize,
            output: total_output as usize,
        })
    } else {
        None
    }
}

impl ToolAdapter for OpenCodeAdapter {
    fn is_available(&self) -> Result<super::ToolStatus, super::AdapterError> {
        let runner = SessionRunner::new();
        match runner.run_command("opencode", &["--version"], Path::new("."), 10) {
            Ok(_) => {
                Ok(super::ToolStatus {
                    available: true,
                    authenticated: true, // opencode doesn't require auth check
                })
            }
            Err(e) => Err(super::AdapterError::NotAvailable(format!(
                "OpenCode tool not found: {}",
                e
            ))),
        }
    }

    fn check_availability(&self) -> anyhow::Result<()> {
        let runner = SessionRunner::new();
        match runner.run_command("opencode", &["--version"], Path::new("."), 10) {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::anyhow!("OpenCode tool not found: {}", e)),
        }
    }

    fn run(
        &self,
        scenario: &Scenario,
        cwd: &Path,
        model: Option<&str>,
        timeout_secs: u64,
    ) -> anyhow::Result<(String, i32, Option<f64>, Option<super::TokenUsage>)> {
        let runner = SessionRunner::new();

        // Use 'opencode run' with JSON format for token extraction
        let mut args = vec!["run", "--format", "json"];
        if let Some(model) = model {
            args.push("--model");
            args.push(model);
        }
        args.push(&scenario.task.prompt);

        // Isolate opencode from global AGENTS.md by using a temp XDG_CONFIG_HOME
        // This ensures test results aren't skewed by global prompts/rules/tools
        // while still allowing authentication to work
        let xdg_config_dir = cwd.join(".opencode_config");
        std::fs::create_dir_all(&xdg_config_dir).ok(); // Create if doesn't exist, ignore errors
        let env_vars: Vec<(String, String)> = vec![(
            "XDG_CONFIG_HOME".to_string(),
            xdg_config_dir.to_string_lossy().to_string(),
        )];

        let (output, exit_code) =
            runner.run_command_with_env("opencode", &args, cwd, timeout_secs, &env_vars)?;
        let token_usage = parse_token_usage_from_json(&output);

        Ok((output, exit_code, None, token_usage))
    }
}
