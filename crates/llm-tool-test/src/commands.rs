use crate::cli::Commands;
use crate::evaluation::ScoreTier;
use crate::results::{Cache, RegressionReport, ResultRecord, ResultsDB};
use crate::run;
use crate::scenario::ToolConfig;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ToolModelConfig {
    pub tool: String,
    pub model: String,
}

pub fn handle_run(
    command: &Commands,
    base_dir: &PathBuf,
    results_db: &ResultsDB,
    cache: &Cache,
) -> anyhow::Result<()> {
    let Commands::Run {
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
    } = command
    else {
        return Err(anyhow::anyhow!("Expected Run command"));
    };

    if let Some(model) = judge_model {
        std::env::set_var("LLM_TOOL_TEST_JUDGE", model);
    }

    if let Some(path) = scenario {
        let s = crate::scenario::load(path)?;
        println!("Loaded scenario: {}", s.name);

        let matrix = build_tool_matrix(tools, model, tool, model, &s.tool_matrix);

        if matrix.len() > 1 {
            println!("Matrix run: {} tool×model combinations", matrix.len());
        }

        let mut results = Vec::new();

        for config in &matrix {
            println!("\n=== Running: {} / {} ===", config.tool, config.model);

            let result = run::run_single_scenario(
                &s,
                &config.tool,
                &config.model,
                *dry_run,
                *no_cache,
                *timeout_secs,
                base_dir,
                results_db,
                cache,
            );

            results.push((config.clone(), result));
        }

        if matrix.len() > 1 {
            print_matrix_summary(&results);
        }
    } else {
        println!("No scenario specified. Use --scenario <path>");
    }

    Ok(())
}

pub fn handle_list(command: &Commands, results_db: &ResultsDB) -> anyhow::Result<()> {
    let Commands::List {
        tags: _,
        tier,
        pending_review,
    } = command
    else {
        return Err(anyhow::anyhow!("Expected List command"));
    };

    if *pending_review {
        let pending = results_db.load_pending_review()?;
        if pending.is_empty() {
            println!("No runs pending review");
        } else {
            println!("Runs pending review ({}):", pending.len());
            for r in pending {
                println!("  [{}] {} - {} ({})", r.id, r.scenario_id, r.tool, r.model);
            }
        }
        return Ok(());
    }

    let mut scenarios = Vec::new();

    fn find_scenarios(dir: &std::path::Path, scenarios: &mut Vec<(String, usize, String)>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "yaml" {
                            if let Ok(s) = crate::scenario::load(&path) {
                                scenarios.push((s.name.clone(), s.tier, s.description));
                            }
                        }
                    }
                } else if path.is_dir() {
                    find_scenarios(&path, scenarios);
                }
            }
        }
    }

    let fixtures_dir = std::path::PathBuf::from("fixtures");
    if fixtures_dir.exists() {
        find_scenarios(&fixtures_dir, &mut scenarios);
    }

    scenarios.sort_by(|a, b| a.1.cmp(&b.1));

    let tier_label = match *tier {
        0 => "smoke",
        1 => "quick",
        2 => "standard",
        3 => "comprehensive",
        _ => "unknown",
    };
    println!("Available scenarios (tier {}):", tier_label);
    for (name, _tier, description) in &scenarios {
        println!("  [{}] {} - {}", tier_label, name, description);
    }

    Ok(())
}

pub fn handle_show(command: &Commands, results_db: &ResultsDB) -> anyhow::Result<()> {
    let Commands::Show { name } = command else {
        return Err(anyhow::anyhow!("Expected Show command"));
    };

    let record = results_db.load_by_id(name)?;
    match record {
        Some(r) => {
            println!("Run ID: {}", r.id);
            println!("Scenario: {}", r.scenario_id);
            println!("Tool: {}", r.tool);
            println!("Timestamp: {}", r.timestamp);
            println!("Duration: {:.2}s", r.duration_secs);
            println!("Cost: ${:.4}", r.cost_usd);
            println!("Outcome: {}", r.outcome);
            println!(
                "Gates: {}/{}",
                r.metrics.gates_passed, r.metrics.gates_total
            );
            println!("Notes: {}", r.metrics.note_count);
            println!("Links: {}", r.metrics.link_count);
            if let Some(score) = r.judge_score {
                let tier = ScoreTier::from_score(score);
                println!("Judge Score: {:.2} ({})", score, tier);
            }
            let composite_tier = ScoreTier::from_score(r.metrics.composite_score);
            println!(
                "Composite Score: {:.2} ({})",
                r.metrics.composite_score, composite_tier
            );
            println!("Transcript: {}", r.transcript_path);
            if let Some(review) = r.human_review {
                println!("Human Review:");
                for (dim, score) in &review.dimensions {
                    println!("  {}: {:.2}", dim, score);
                }
                if let Some(notes) = &review.notes {
                    println!("  Notes: {}", notes);
                }
                println!("  Reviewed: {}", review.timestamp);
            }
        }
        None => println!("Run not found: {}", name),
    }

    Ok(())
}

pub fn handle_compare(command: &Commands, results_db: &ResultsDB) -> anyhow::Result<()> {
    let Commands::Compare { run_ids } = command else {
        return Err(anyhow::anyhow!("Expected Compare command"));
    };

    if run_ids.len() != 2 {
        anyhow::bail!("Compare requires exactly 2 run IDs");
    }

    let r1 = results_db.load_by_id(&run_ids[0])?;
    let r2 = results_db.load_by_id(&run_ids[1])?;

    match (r1, r2) {
        (Some(run1), Some(run2)) => {
            let report = crate::results::compare_runs(&run1, &run2);
            print_regression_report(&report);
        }
        _ => anyhow::bail!("One or both runs not found"),
    }

    Ok(())
}

pub fn handle_clean(cache: &Cache) -> anyhow::Result<()> {
    println!("Cleaning cache...");
    cache.clear()?;
    println!("Cache cleared");
    Ok(())
}

pub fn handle_review(command: &Commands, results_db: &ResultsDB) -> anyhow::Result<()> {
    use chrono::Utc;

    let Commands::Review {
        run_id,
        dimension,
        notes,
    } = command
    else {
        return Err(anyhow::anyhow!("Expected Review command"));
    };

    let record = results_db.load_by_id(&run_id)?;
    match record {
        Some(_) => {
            let dimensions_map: HashMap<String, f64> = dimension
                .iter()
                .map(|(k, v): &(String, f64)| (k.clone(), *v))
                .collect();
            let human_review = crate::results::HumanReviewRecord {
                dimensions: dimensions_map,
                notes: notes.clone(),
                timestamp: Utc::now(),
            };

            results_db.update_human_review(&run_id, human_review)?;
            println!("Review added for run: {}", run_id);
        }
        None => anyhow::bail!("Run not found: {}", run_id),
    }

    Ok(())
}

fn build_tool_matrix(
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

fn print_matrix_summary(results: &[(ToolModelConfig, anyhow::Result<ResultRecord>)]) {
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
        let result = build_tool_matrix(
            &Some("opencode,amp".to_string()),
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
