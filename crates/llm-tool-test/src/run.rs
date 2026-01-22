use crate::adapter::{
    amp::AmpAdapter, claude_code::ClaudeCodeAdapter, mock::MockAdapter, opencode::OpenCodeAdapter,
    ToolAdapter,
};
use crate::output;
use crate::results::{
    Cache, CacheKey, EfficiencyMetricsRecord, EvaluationMetricsRecord, GateResultRecord,
    QualityMetricsRecord, ResultRecord, ResultsDB,
};

use std::time::Instant;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::Scenario;
    use std::path::PathBuf;

    #[test]
    fn test_budget_enforcement_cli_max_usd() {
        let scenario_yaml = r#"
name: budget_test
description: "Test CLI budget enforcement"
fixture: qipu
task:
  prompt: "Create a note"
evaluation:
  gates:
    - type: min_notes
      count: 1
"#;
        let scenario: Scenario = serde_yaml::from_str(scenario_yaml).unwrap();
        let base_dir = PathBuf::from("target/test_budget");
        std::fs::create_dir_all(&base_dir).unwrap();

        let results_db = ResultsDB::new(&base_dir);
        let cache = Cache::new(&base_dir);

        // Create a fixture file for the test
        let fixture_dir = PathBuf::from("fixtures/qipu");
        std::fs::create_dir_all(&fixture_dir).unwrap();
        let fixture_file = fixture_dir.join("budget_test.yaml");
        std::fs::write(&fixture_file, scenario_yaml).unwrap();

        // Test with zero budget - should fail immediately
        let cli_max_usd = Some(0.0);
        let result = run_single_scenario(
            &scenario,
            "mock",
            "mock",
            false,
            true, // no_cache to avoid cache hits
            30,
            &cli_max_usd,
            &base_dir,
            &results_db,
            &cache,
        );

        // Clean up
        let _ = std::fs::remove_file(&fixture_file);

        match result {
            Err(e) => {
                assert!(
                    e.to_string().contains("Budget exhausted"),
                    "Error message should mention budget, got: {}",
                    e
                );
            }
            Ok(_) => panic!("Should fail with zero budget, but succeeded"),
        }
    }

    #[test]
    fn test_budget_enforcement_scenario_max_usd() {
        let scenario_yaml = r#"
name: budget_test_scenario
description: "Test scenario budget enforcement"
fixture: qipu
task:
  prompt: "Create a note"
evaluation:
  gates:
    - type: min_notes
      count: 1
cost:
  max_usd: 0.0
"#;
        let scenario: Scenario = serde_yaml::from_str(scenario_yaml).unwrap();
        let base_dir = PathBuf::from("target/test_budget");
        std::fs::create_dir_all(&base_dir).unwrap();

        let results_db = ResultsDB::new(&base_dir);
        let cache = Cache::new(&base_dir);

        // Create a fixture file for the test
        let fixture_dir = PathBuf::from("fixtures/qipu");
        std::fs::create_dir_all(&fixture_dir).unwrap();
        let fixture_file = fixture_dir.join("budget_test_scenario.yaml");
        std::fs::write(&fixture_file, scenario_yaml).unwrap();

        // Test with zero budget from scenario
        let result = run_single_scenario(
            &scenario,
            "mock",
            "mock",
            false,
            true, // no_cache to avoid cache hits
            30,
            &None, // No CLI budget
            &base_dir,
            &results_db,
            &cache,
        );

        // Clean up
        let _ = std::fs::remove_file(&fixture_file);

        assert!(
            result.is_err(),
            "Should fail with zero budget from scenario"
        );
        assert!(
            result.unwrap_err().to_string().contains("Budget exhausted"),
            "Error message should mention budget"
        );
    }

    #[test]
    fn test_budget_enforcement_takes_minimum() {
        let scenario_yaml = r#"
name: budget_test_min
description: "Test budget enforcement takes minimum"
fixture: qipu
task:
  prompt: "Create a note"
evaluation:
  gates:
    - type: min_notes
      count: 1
cost:
  max_usd: 0.5
"#;
        let scenario: Scenario = serde_yaml::from_str(scenario_yaml).unwrap();
        let base_dir = PathBuf::from("target/test_budget");
        std::fs::create_dir_all(&base_dir).unwrap();

        let results_db = ResultsDB::new(&base_dir);
        let cache = Cache::new(&base_dir);

        // Create a fixture file for the test
        let fixture_dir = PathBuf::from("fixtures/qipu");
        std::fs::create_dir_all(&fixture_dir).unwrap();
        let fixture_file = fixture_dir.join("budget_test_min.yaml");
        std::fs::write(&fixture_file, scenario_yaml).unwrap();

        // CLI has lower budget than scenario (0.0 vs 0.5) - should use CLI
        let cli_max_usd = Some(0.0);
        let result = run_single_scenario(
            &scenario,
            "mock",
            "mock",
            false,
            true, // no_cache to avoid cache hits
            30,
            &cli_max_usd,
            &base_dir,
            &results_db,
            &cache,
        );

        // Clean up
        let _ = std::fs::remove_file(&fixture_file);

        assert!(
            result.is_err(),
            "Should fail with CLI budget (lower than scenario)"
        );
        assert!(
            result.unwrap_err().to_string().contains("Budget exhausted"),
            "Error message should mention budget"
        );
    }
}

pub fn run_single_scenario(
    s: &crate::scenario::Scenario,
    tool: &str,
    model: &str,
    dry_run: bool,
    no_cache: bool,
    timeout_secs: u64,
    cli_max_usd: &Option<f64>,
    _base_dir: &std::path::Path,
    results_db: &ResultsDB,
    cache: &Cache,
) -> anyhow::Result<ResultRecord> {
    let scenario_path = format!("fixtures/{}/{}.yaml", s.fixture, s.name);
    let scenario_yaml = std::fs::read_to_string(&scenario_path)?;
    let prompt = s.task.prompt.clone();
    let qipu_version = crate::results::get_qipu_version()?;

    // Budget enforcement: check max_usd from CLI and scenario BEFORE setting up fixture
    let effective_max_usd = match (cli_max_usd, s.cost.as_ref().and_then(|c| c.max_usd)) {
        (Some(cli), Some(scenario)) => Some(cli.min(scenario)),
        (Some(cli), None) => Some(*cli),
        (None, Some(scenario)) => Some(scenario),
        (None, None) => None,
    };

    if let Some(max_usd) = effective_max_usd {
        if max_usd <= 0.0 {
            anyhow::bail!(
                "Budget exhausted: max_usd is ${:.4}. Cannot run scenario.",
                max_usd
            );
        }
        println!("Budget limit: ${:.4}", max_usd);
    }

    // Set up fixture to get prime output for cache key
    println!("Setting up environment for fixture: {}", s.fixture);
    let env = crate::fixture::TestEnv::new(&s.name)?;
    env.setup_fixture(&s.fixture)?;
    println!("Environment created at: {:?}", env.root);

    // Get prime output for cache key (empty string if no store yet)
    let prime_output = env.get_prime_output();

    // Compute cache key with prime output hash
    let cache_key = CacheKey::compute(
        &scenario_yaml,
        &prompt,
        &prime_output,
        tool,
        model,
        &qipu_version,
    );

    if !no_cache {
        if let Some(cached) = cache.get(&cache_key) {
            println!("Cache HIT! Using cached result: {}", cached.id);
            output::print_result_summary(&cached);
            return Ok(cached);
        }
    }

    let adapter: Box<dyn ToolAdapter> = match tool {
        "amp" => Box::new(AmpAdapter),
        "claude-code" => Box::new(ClaudeCodeAdapter),
        "mock" => Box::new(MockAdapter),
        "opencode" => Box::new(OpenCodeAdapter),
        _ => anyhow::bail!("Unknown tool: {}", tool),
    };

    if !dry_run {
        println!("Checking availability for tool: {}", tool);
        if let Err(e) = adapter.check_availability() {
            anyhow::bail!("Tool unavailable: {}", e);
        }

        if let Some(setup_steps) = &s.setup {
            println!("Running {} setup step(s)...", setup_steps.len());
            let runner = crate::session::SessionRunner::new();
            for (i, step) in setup_steps.iter().enumerate() {
                let args: Vec<&str> = step.args.iter().map(|s| s.as_str()).collect();
                println!(
                    "  Step {}/{}: {} {}",
                    i + 1,
                    setup_steps.len(),
                    step.command,
                    args.join(" ")
                );
                let (output, exit_code) =
                    runner.run_command(&step.command, &args, &env.root, timeout_secs)?;
                if exit_code != 0 {
                    anyhow::bail!(
                        "Setup step {}/{} failed: {} exited with code {}. Output: {}",
                        i + 1,
                        setup_steps.len(),
                        step.command,
                        exit_code,
                        output
                    );
                }
            }
            println!("Setup complete.");
        }

        let start_time = Instant::now();
        println!("Running tool '{}' with model '{}'...", tool, model);
        let (output, exit_code, cost) = adapter.run(s, &env.root, Some(model), timeout_secs)?;
        let duration = start_time.elapsed();

        // Check if we exceeded the budget
        if let Some(max_usd) = effective_max_usd {
            if cost > max_usd {
                eprintln!(
                    "WARNING: Run cost ${:.4} exceeded budget ${:.4}",
                    cost, max_usd
                );
            }
        }

        let transcript_dir = env.root.join("artifacts");
        std::fs::create_dir_all(&transcript_dir)?;
        let writer = crate::transcript::TranscriptWriter::new(transcript_dir.clone())?;
        writer.write_raw(&output)?;
        writer.append_event(&serde_json::json!({
            "type": "execution",
            "tool": tool,
            "output": output,
            "exit_code": exit_code,
            "cost_usd": cost
        }))?;

        println!("Running evaluation...");
        let metrics = crate::evaluation::evaluate(s, &env.root)?;
        println!("Evaluation metrics: {:?}", metrics);

        let outcome = if metrics.gates_passed < metrics.gates_total {
            format!(
                "Fail: {}/{} gates passed",
                metrics.gates_passed, metrics.gates_total
            )
        } else {
            "Pass".to_string()
        };

        // Write run.json metadata
        let run_metadata = crate::transcript::RunMetadata {
            scenario_id: s.name.clone(),
            scenario_hash: cache_key.scenario_hash.clone(),
            tool: tool.to_string(),
            model: model.to_string(),
            qipu_version: qipu_version.clone(),
            qipu_commit: qipu_version.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration_secs: duration.as_secs_f64(),
            cost_estimate_usd: cost,
            token_usage: None, // TODO: Extract from adapter if available
        };
        writer.write_run_metadata(&run_metadata)?;

        // Create store snapshot
        writer.create_store_snapshot(&env.root)?;

        // Write report.md
        let report = crate::transcript::RunReport {
            scenario_id: s.name.clone(),
            tool: tool.to_string(),
            model: model.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration_secs: duration.as_secs_f64(),
            cost_usd: cost,
            token_usage: None, // TODO: Extract from adapter if available
            outcome: outcome.clone(),
            gates_passed: metrics.gates_passed,
            gates_total: metrics.gates_total,
            note_count: metrics.note_count,
            link_count: metrics.link_count,
            composite_score: Some(metrics.composite_score),
            gate_details: metrics
                .details
                .iter()
                .map(|d| crate::transcript::GateDetail {
                    gate_type: d.gate_type.clone(),
                    passed: d.passed,
                    message: d.message.clone(),
                })
                .collect(),
            efficiency: crate::transcript::EfficiencyReport {
                total_commands: metrics.efficiency.total_commands,
                unique_commands: metrics.efficiency.unique_commands,
                error_count: metrics.efficiency.error_count,
                first_try_success_rate: metrics.efficiency.first_try_success_rate,
                iteration_ratio: metrics.efficiency.iteration_ratio,
            },
            quality: crate::transcript::QualityReport {
                avg_title_length: metrics.quality.avg_title_length,
                avg_body_length: metrics.quality.avg_body_length,
                avg_tags_per_note: metrics.quality.avg_tags_per_note,
                links_per_note: metrics.quality.links_per_note,
                orphan_notes: metrics.quality.orphan_notes,
            },
        };
        writer.write_report(&report)?;

        let transcript_path = transcript_dir.to_string_lossy().to_string();

        let record = ResultRecord {
            id: crate::results::generate_run_id(),
            scenario_id: s.name.clone(),
            scenario_hash: cache_key.scenario_hash.clone(),
            tool: tool.to_string(),
            model: model.to_string(),
            qipu_commit: qipu_version.clone(),
            timestamp: chrono::Utc::now(),
            duration_secs: duration.as_secs_f64(),
            cost_usd: cost,
            gates_passed: metrics.gates_passed >= metrics.gates_total,
            metrics: EvaluationMetricsRecord {
                gates_passed: metrics.gates_passed,
                gates_total: metrics.gates_total,
                note_count: metrics.note_count,
                link_count: metrics.link_count,
                details: metrics
                    .details
                    .into_iter()
                    .map(|d| GateResultRecord {
                        gate_type: d.gate_type,
                        passed: d.passed,
                        message: d.message,
                    })
                    .collect(),
                efficiency: EfficiencyMetricsRecord {
                    total_commands: metrics.efficiency.total_commands,
                    unique_commands: metrics.efficiency.unique_commands,
                    error_count: metrics.efficiency.error_count,
                    retry_count: metrics.efficiency.retry_count,
                    help_invocations: metrics.efficiency.help_invocations,
                    first_try_success_rate: metrics.efficiency.first_try_success_rate,
                    iteration_ratio: metrics.efficiency.iteration_ratio,
                },
                quality: QualityMetricsRecord {
                    avg_title_length: metrics.quality.avg_title_length,
                    avg_body_length: metrics.quality.avg_body_length,
                    avg_tags_per_note: metrics.quality.avg_tags_per_note,
                    notes_without_tags: metrics.quality.notes_without_tags,
                    links_per_note: metrics.quality.links_per_note,
                    orphan_notes: metrics.quality.orphan_notes,
                    link_type_diversity: metrics.quality.link_type_diversity,
                    type_distribution: metrics.quality.type_distribution,
                    total_notes: metrics.quality.total_notes,
                    total_links: metrics.quality.total_links,
                },
                composite_score: metrics.composite_score,
            },
            judge_score: metrics.judge_score,
            outcome,
            transcript_path: transcript_path.clone(),
            cache_key: Some(cache_key.as_string()),
            human_review: None,
        };

        results_db.append(&record)?;
        cache.put(&cache_key, &record)?;

        if let Some(baseline) = results_db.load_baseline(&s.name, tool)? {
            let report = crate::results::compare_runs(&record, &baseline);
            output::print_regression_report(&report);
        }

        println!("\nRun completed: {}", record.id);
        println!("Transcript written to: {}", transcript_path);
        output::print_result_summary(&record);

        Ok(record)
    } else {
        println!("Dry run - skipping execution");

        let record = ResultRecord {
            id: crate::results::generate_run_id(),
            scenario_id: s.name.clone(),
            scenario_hash: cache_key.scenario_hash.clone(),
            tool: tool.to_string(),
            model: model.to_string(),
            qipu_commit: qipu_version.clone(),
            timestamp: chrono::Utc::now(),
            duration_secs: 0.0,
            cost_usd: 0.0,
            gates_passed: true,
            metrics: EvaluationMetricsRecord {
                gates_passed: 0,
                gates_total: 0,
                note_count: 0,
                link_count: 0,
                details: vec![],
                efficiency: EfficiencyMetricsRecord {
                    total_commands: 0,
                    unique_commands: 0,
                    error_count: 0,
                    retry_count: 0,
                    help_invocations: 0,
                    first_try_success_rate: 0.0,
                    iteration_ratio: 0.0,
                },
                quality: QualityMetricsRecord {
                    avg_title_length: 0.0,
                    avg_body_length: 0.0,
                    avg_tags_per_note: 0.0,
                    notes_without_tags: 0,
                    links_per_note: 0.0,
                    orphan_notes: 0,
                    link_type_diversity: 0,
                    type_distribution: std::collections::HashMap::new(),
                    total_notes: 0,
                    total_links: 0,
                },
                composite_score: 0.0,
            },
            judge_score: None,
            outcome: "Dry run".to_string(),
            transcript_path: String::new(),
            cache_key: Some(cache_key.as_string()),
            human_review: None,
        };

        output::print_result_summary(&record);
        Ok(record)
    }
}
