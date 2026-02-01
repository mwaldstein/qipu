//! LLM tool test runner
//!
//! Evaluates LLM tools against test scenarios with automatic judging.

mod adapter;
mod cli;
mod commands;
mod config;
mod eval_helpers;
#[cfg(test)]
mod eval_tests_doctor;
#[cfg(test)]
mod eval_tests_gates;
#[cfg(test)]
mod eval_tests_score;
mod evaluation;
mod fixture;
mod judge;
mod output;
mod pricing;
mod results;
mod run;
mod scenario;
mod session;
mod store_analysis;
mod transcript;
mod utils;

use clap::Parser;
use cli::Cli;
use cli::Commands;
use results::{Cache, ResultsDB};
use scenario::ToolConfig;
use std::iter::Iterator;

/// Build a matrix of tool-model configurations from CLI args or scenario config
pub fn build_tool_matrix(
    cli_tools: &Option<String>,
    cli_models: &Option<String>,
    cli_tool: &str,
    cli_model: &Option<String>,
    scenario_matrix: &Option<Vec<ToolConfig>>,
) -> Vec<output::ToolModelConfig> {
    if let (Some(tools_str), Some(models_str)) = (cli_tools, cli_models) {
        let tools: Vec<String> = tools_str.split(',').map(|s| s.trim().to_string()).collect();
        let models: Vec<String> = models_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let mut matrix = Vec::new();
        for tool in tools {
            for model in &models {
                matrix.push(output::ToolModelConfig {
                    tool: tool.clone(),
                    model: model.clone(),
                });
            }
        }
        matrix
    } else if let Some(scenario_matrix) = scenario_matrix {
        let mut matrix = Vec::new();
        for config in scenario_matrix {
            let models = if config.models.is_empty() {
                vec!["default".to_string()]
            } else {
                config.models.clone()
            };
            for model in models {
                matrix.push(output::ToolModelConfig {
                    tool: config.tool.clone(),
                    model,
                });
            }
        }
        matrix
    } else {
        vec![output::ToolModelConfig {
            tool: cli_tool.to_string(),
            model: cli_model.as_deref().unwrap_or("default").to_string(),
        }]
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Use the configured results path for cache and database
    // Resolve to absolute path to avoid issues with nested creation
    let config = crate::config::Config::load_or_default();
    let base_dir = std::path::PathBuf::from(config.get_results_path());
    let base_dir = base_dir.canonicalize().unwrap_or_else(|_| {
        // If canonicalize fails (path doesn't exist), create it and try again
        std::fs::create_dir_all(&base_dir).ok();
        base_dir.canonicalize().unwrap_or(base_dir)
    });
    let results_db = ResultsDB::new(&base_dir);
    let cache = Cache::new(&base_dir);

    match &cli.command {
        Commands::Run {
            scenario,
            all,
            tags,
            tier,
            tool,
            model,
            tools,
            models,
            dry_run,
            no_cache,
            judge_model,
            no_judge,
            timeout_secs,
            max_usd,
        } => {
            // Safety check: only run tests when explicitly enabled
            if std::env::var("LLM_TOOL_TEST_ENABLED").as_deref() != Ok("1") {
                anyhow::bail!(
                    "LLM tool test runs require LLM_TOOL_TEST_ENABLED=1 to be set as a safety measure.\n\
                     This prevents accidental expensive LLM API calls.\n\
                     \n\
                     To run tests, set:\n\
                     export LLM_TOOL_TEST_ENABLED=1"
                );
            }

            // Get session budget: CLI flag takes precedence over env var
            let session_budget = max_usd.or_else(|| {
                std::env::var("LLM_TOOL_TEST_BUDGET_USD")
                    .ok()
                    .and_then(|v| v.parse::<f64>().ok())
            });

            let selection = commands::ScenarioSelection {
                scenario: scenario.clone(),
                all: *all,
                tags: tags.clone(),
                tier: *tier,
            };

            let exec_config = commands::ExecutionConfig {
                tool: tool.clone(),
                model: model.clone(),
                tools: tools.clone(),
                models: models.clone(),
                dry_run: *dry_run,
                no_cache: *no_cache,
                timeout_secs: *timeout_secs,
                judge_model: judge_model.clone(),
                no_judge: *no_judge,
                session_budget,
            };

            let ctx = commands::ExecutionContext {
                base_dir: &base_dir,
                results_db: &results_db,
                cache: &cache,
            };

            if selection.scenario.is_some() || selection.all {
                commands::handle_run_command(&selection, &exec_config, &ctx)?;
            } else {
                println!("No scenario specified. Use --scenario <path> or --all");
            }
        }
        Commands::Scenarios { tags, tier } => {
            commands::handle_list_command(tags, tier, &results_db)?;
        }
        Commands::Show { name } => {
            commands::handle_show_command(name, &results_db)?;
        }
        Commands::Clean { older_than } => {
            commands::handle_clean_command(&cache, older_than, &base_dir)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use output::ToolModelConfig;
    use scenario::ToolConfig;

    fn assert_matrix_contains(matrix: &[ToolModelConfig], tool: &str, model: &str) {
        assert!(
            matrix.iter().any(|c| c.tool == tool && c.model == model),
            "Matrix should contain ({}, {}), got: {:?}",
            tool,
            model,
            matrix
        );
    }

    #[test]
    fn test_build_tool_matrix_cli_both() {
        let result = build_tool_matrix(
            &Some("opencode,claude-code".to_string()),
            &Some("gpt-4o,claude-sonnet".to_string()),
            "opencode",
            &None,
            &None,
        );

        assert_eq!(result.len(), 4);
        assert_matrix_contains(&result, "opencode", "gpt-4o");
        assert_matrix_contains(&result, "opencode", "claude-sonnet");
        assert_matrix_contains(&result, "claude-code", "gpt-4o");
        assert_matrix_contains(&result, "claude-code", "claude-sonnet");
    }

    #[test]
    fn test_build_tool_matrix_cli_whitespace_handling() {
        let result = build_tool_matrix(
            &Some(" opencode , claude-code ".to_string()),
            &Some(" gpt-4o , claude-sonnet ".to_string()),
            "opencode",
            &None,
            &None,
        );

        assert_eq!(result.len(), 4);
        assert_matrix_contains(&result, "opencode", "gpt-4o");
        assert_matrix_contains(&result, "opencode", "claude-sonnet");
        assert_matrix_contains(&result, "claude-code", "gpt-4o");
        assert_matrix_contains(&result, "claude-code", "claude-sonnet");
    }

    #[test]
    fn test_build_tool_matrix_scenario_matrix_with_models() {
        let scenario_matrix = vec![
            ToolConfig {
                tool: "opencode".to_string(),
                models: vec!["gpt-4o".to_string(), "claude-sonnet".to_string()],
            },
            ToolConfig {
                tool: "claude-code".to_string(),
                models: vec!["default".to_string()],
            },
        ];

        let result = build_tool_matrix(&None, &None, "opencode", &None, &Some(scenario_matrix));

        assert_eq!(result.len(), 3);
        assert_matrix_contains(&result, "opencode", "gpt-4o");
        assert_matrix_contains(&result, "opencode", "claude-sonnet");
        assert_matrix_contains(&result, "claude-code", "default");
    }

    #[test]
    fn test_build_tool_matrix_scenario_matrix_empty_models() {
        let scenario_matrix = vec![
            ToolConfig {
                tool: "opencode".to_string(),
                models: vec![],
            },
            ToolConfig {
                tool: "claude-code".to_string(),
                models: vec![],
            },
        ];

        let result = build_tool_matrix(&None, &None, "opencode", &None, &Some(scenario_matrix));

        assert_eq!(result.len(), 2);
        assert_matrix_contains(&result, "opencode", "default");
        assert_matrix_contains(&result, "claude-code", "default");
    }

    #[test]
    fn test_build_tool_matrix_single_tool_default_model() {
        let result = build_tool_matrix(&None, &None, "opencode", &None, &None);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].tool, "opencode");
        assert_eq!(result[0].model, "default");
    }

    #[test]
    fn test_build_tool_matrix_single_tool_with_model() {
        let result = build_tool_matrix(
            &None,
            &None,
            "opencode",
            &Some("claude-sonnet".to_string()),
            &None,
        );

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].tool, "opencode");
        assert_eq!(result[0].model, "claude-sonnet");
    }

    #[test]
    fn test_build_tool_matrix_scenario_matrix_empty() {
        let scenario_matrix: Vec<ToolConfig> = vec![];

        let result = build_tool_matrix(&None, &None, "opencode", &None, &Some(scenario_matrix));

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_build_tool_matrix_cli_tools_only() {
        let result = build_tool_matrix(
            &Some("opencode,claude-code".to_string()),
            &None,
            "opencode",
            &None,
            &None,
        );

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].tool, "opencode");
        assert_eq!(result[0].model, "default");
    }

    #[test]
    fn test_build_tool_matrix_cli_models_only() {
        let result = build_tool_matrix(
            &None,
            &Some("gpt-4o,claude-sonnet".to_string()),
            "opencode",
            &None,
            &None,
        );

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].tool, "opencode");
        assert_eq!(result[0].model, "default");
    }

    #[test]
    fn test_build_tool_matrix_single_tool_empty_strings() {
        let result = build_tool_matrix(
            &Some("opencode,,claude-code".to_string()),
            &Some("gpt-4o,,claude-sonnet".to_string()),
            "opencode",
            &None,
            &None,
        );

        assert_eq!(result.len(), 9);
        assert_matrix_contains(&result, "opencode", "gpt-4o");
        assert_matrix_contains(&result, "opencode", "");
        assert_matrix_contains(&result, "opencode", "claude-sonnet");
        assert_matrix_contains(&result, "", "gpt-4o");
        assert_matrix_contains(&result, "", "");
        assert_matrix_contains(&result, "", "claude-sonnet");
        assert_matrix_contains(&result, "claude-code", "gpt-4o");
        assert_matrix_contains(&result, "claude-code", "");
        assert_matrix_contains(&result, "claude-code", "claude-sonnet");
    }
}
