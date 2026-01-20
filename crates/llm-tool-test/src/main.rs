mod adapter;
mod cli;
mod commands;
mod evaluation;
mod fixture;
mod judge;
mod results;
mod run;
mod scenario;
mod session;
mod store_analysis;
mod transcript;

use clap::Parser;
use cli::{Cli, Commands};
use evaluation::ScoreTier;
use results::{Cache, RegressionReport, ResultRecord, ResultsDB};
use scenario::ToolConfig;
use std::collections::HashMap;
use std::iter::Iterator;

#[derive(Debug, Clone)]
struct ToolModelConfig {
    tool: String,
    model: String,
}

pub fn build_tool_matrix(
    cli_tools: &Option<String>,
    cli_models: &Option<String>,
    cli_tool: &str,
    cli_model: &Option<String>,
    scenario_matrix: &Option<Vec<ToolConfig>>,
) -> Vec<ToolModelConfig> {
    if let (Some(tools_str), Some(models_str)) = (cli_tools, cli_models) {
        let tools: Vec<String> = tools_str.split(',').map(|s| s.trim().to_string()).collect();
        let models: Vec<String> = models_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let mut matrix = Vec::new();
        for tool in &tools {
            for model in &models {
                matrix.push(ToolModelConfig {
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
                matrix.push(ToolModelConfig {
                    tool: config.tool.clone(),
                    model,
                });
            }
        }
        matrix
    } else {
        vec![ToolModelConfig {
            tool: cli_tool.to_string(),
            model: cli_model.as_deref().unwrap_or("default").to_string(),
        }]
    }
}

pub fn print_matrix_summary(results: &[(ToolModelConfig, anyhow::Result<ResultRecord>)]) {
    println!("\n--- Matrix Summary ---");

    let mut table: HashMap<String, HashMap<String, String>> = HashMap::new();

    for (config, result) in results {
        let outcome = match result {
            Ok(record) => record.outcome.clone(),
            Err(e) => format!("Error: {}", e),
        };

        table
            .entry(config.tool.clone())
            .or_default()
            .insert(config.model.clone(), outcome);
    }

    println!("{:<20} |", "Tool");
    let all_models: std::collections::BTreeSet<_> = table
        .values()
        .flat_map(|models| models.keys().cloned())
        .collect();

    for model in &all_models {
        print!(" {:<20} |", model);
    }
    println!();

    println!("{}", "-".repeat(22));
    for _ in &all_models {
        println!("{}", "-".repeat(22));
    }
    println!();

    let mut tools: Vec<_> = table.keys().collect();
    tools.sort();

    for tool in tools {
        print!("{:<20} |", tool);
        for model in &all_models {
            let default = "-";
            let outcome = table
                .get(tool)
                .and_then(|m| m.get(model))
                .map_or(default, |v| v.as_str());
            print!(" {:<20} |", outcome);
        }
        println!();
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let base_dir = std::path::PathBuf::from("target/llm_test_runs");
    let results_db = ResultsDB::new(&base_dir);
    let cache = Cache::new(&base_dir);

    match &cli.command {
        Commands::Run {
            scenario,
            tags: _,
            tier: _,
            tool,
            model,
            tools,
            models,
            max_usd: _,
            dry_run,
            no_cache,
            judge_model,
            timeout_secs,
        } => {
            if let Some(path) = scenario {
                let s = scenario::load(path)?;
                commands::handle_run_command(
                    scenario,
                    tool,
                    model,
                    tools,
                    models,
                    *dry_run,
                    *no_cache,
                    *timeout_secs,
                    judge_model,
                    &s.tool_matrix,
                    &base_dir,
                    &results_db,
                    &cache,
                )?;
            } else {
                println!("No scenario specified. Use --scenario <path>");
            }
        }
        Commands::List {
            tags: _,
            tier,
            pending_review,
        } => {
            commands::handle_list_command(tier, *pending_review, &results_db)?;
        }
        Commands::Show { name } => {
            commands::handle_show_command(name, &results_db)?;
        }
        Commands::Compare { run_ids } => {
            commands::handle_compare_command(run_ids, &results_db)?;
        }
        Commands::Clean => {
            commands::handle_clean_command(&cache)?;
        }
        Commands::Review {
            run_id,
            dimension,
            notes,
        } => {
            commands::handle_review_command(run_id, dimension, notes, &results_db)?;
        }
    }
    Ok(())
}

pub fn print_result_summary(record: &ResultRecord) {
    println!("\n--- Result Summary ---");
    println!("ID: {}", record.id);
    println!("Scenario: {}", record.scenario_id);
    println!("Tool: {}", record.tool);
    println!("Outcome: {}", record.outcome);
    println!(
        "Gates: {}/{}",
        record.metrics.gates_passed, record.metrics.gates_total
    );
    println!("Notes: {}", record.metrics.note_count);
    println!("Links: {}", record.metrics.link_count);
    println!("Duration: {:.2}s", record.duration_secs);
    println!(
        "Commands: {} ({} unique, {} errors, {} help, {} retries)",
        record.metrics.efficiency.total_commands,
        record.metrics.efficiency.unique_commands,
        record.metrics.efficiency.error_count,
        record.metrics.efficiency.help_invocations,
        record.metrics.efficiency.retry_count
    );
    println!(
        "First-try success: {:.0}%, iteration ratio: {:.2}",
        record.metrics.efficiency.first_try_success_rate * 100.0,
        record.metrics.efficiency.iteration_ratio
    );
    if let Some(score) = record.judge_score {
        let tier = ScoreTier::from_score(score);
        println!("Judge Score: {:.2} ({})", score, tier);
    }
    let composite_tier = ScoreTier::from_score(record.metrics.composite_score);
    println!(
        "Composite Score: {:.2} ({})",
        record.metrics.composite_score, composite_tier
    );
    if record.human_review.is_some() {
        println!("Human Review: Yes");
    }
}

pub fn print_regression_report(report: &RegressionReport) {
    println!("\n--- Regression Report ---");
    println!("Current: {}", report.run_id);
    println!("Baseline: {}", report.baseline_id);

    if let Some(score_change) = report.score_change_pct {
        println!("Score change: {:.1}%", score_change);
    }

    println!("Cost change: {:.1}%", report.cost_change_pct);

    if !report.warnings.is_empty() {
        println!("\nWarnings:");
        for warning in &report.warnings {
            println!("  - {}", warning);
        }
    }

    if !report.alerts.is_empty() {
        println!("\nAlerts:");
        for alert in &report.alerts {
            println!("  - {}", alert);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scenario::ToolConfig;

    fn assert_matrix_contains(matrix: &[ToolModelConfig], tool: &str, model: &str) {
        assert!(
            matrix
                .iter()
                .any(|c| c.tool == tool && c.model == model),
            "Matrix should contain ({}, {}), got: {:?}",
            tool,
            model,
            matrix
        );
    }

    #[test]
    fn test_build_tool_matrix_cli_both() {
        let result = build_tool_matrix(
            &Some("opencode,amp".to_string()),
            &Some("gpt-4o,claude-sonnet".to_string()),
            "opencode",
            &None,
            &None,
        );

        assert_eq!(result.len(), 4);
        assert_matrix_contains(&result, "opencode", "gpt-4o");
        assert_matrix_contains(&result, "opencode", "claude-sonnet");
        assert_matrix_contains(&result, "amp", "gpt-4o");
        assert_matrix_contains(&result, "amp", "claude-sonnet");
    }

    #[test]
    fn test_build_tool_matrix_cli_whitespace_handling() {
        let result = build_tool_matrix(
            &Some(" opencode , amp ".to_string()),
            &Some(" gpt-4o , claude-sonnet ".to_string()),
            "opencode",
            &None,
            &None,
        );

        assert_eq!(result.len(), 4);
        assert_matrix_contains(&result, "opencode", "gpt-4o");
        assert_matrix_contains(&result, "opencode", "claude-sonnet");
        assert_matrix_contains(&result, "amp", "gpt-4o");
        assert_matrix_contains(&result, "amp", "claude-sonnet");
    }

    #[test]
    fn test_build_tool_matrix_scenario_matrix_with_models() {
        let scenario_matrix = vec![
            ToolConfig {
                tool: "opencode".to_string(),
                models: vec!["gpt-4o".to_string(), "claude-sonnet".to_string()],
            },
            ToolConfig {
                tool: "amp".to_string(),
                models: vec!["default".to_string()],
            },
        ];

        let result = build_tool_matrix(&None, &None, "opencode", &None, &Some(scenario_matrix));

        assert_eq!(result.len(), 3);
        assert_matrix_contains(&result, "opencode", "gpt-4o");
        assert_matrix_contains(&result, "opencode", "claude-sonnet");
        assert_matrix_contains(&result, "amp", "default");
    }

    #[test]
    fn test_build_tool_matrix_scenario_matrix_empty_models() {
        let scenario_matrix = vec![
            ToolConfig {
                tool: "opencode".to_string(),
                models: vec![],
            },
            ToolConfig {
                tool: "amp".to_string(),
                models: vec![],
            },
        ];

        let result = build_tool_matrix(&None, &None, "opencode", &None, &Some(scenario_matrix));

        assert_eq!(result.len(), 2);
        assert_matrix_contains(&result, "opencode", "default");
        assert_matrix_contains(&result, "amp", "default");
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
        let result = build_tool_matrix(&Some("opencode,amp".to_string()), &None, "opencode", &None, &None);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].tool, "opencode");
        assert_eq!(result[0].model, "default");
    }

    #[test]
    fn test_build_tool_matrix_cli_models_only() {
        let result = build_tool_matrix(&None, &Some("gpt-4o,claude-sonnet".to_string()), "opencode", &None, &None);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].tool, "opencode");
        assert_eq!(result[0].model, "default");
    }

    #[test]
    fn test_build_tool_matrix_single_tool_empty_strings() {
        let result = build_tool_matrix(
            &Some("opencode,,amp".to_string()),
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
        assert_matrix_contains(&result, "amp", "gpt-4o");
        assert_matrix_contains(&result, "amp", "");
        assert_matrix_contains(&result, "amp", "claude-sonnet");
    }
}
