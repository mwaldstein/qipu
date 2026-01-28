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
    fn test_scenario_timeout_overrides_cli() {
        let scenario_yaml = r#"
name: timeout_test_override
description: "Test scenario timeout overrides CLI"
template_folder: qipu
task:
  prompt: "Create a note"
evaluation:
  gates:
    - type: min_notes
      count: 1
run:
  timeout_secs: 120
"#;
        let scenario: Scenario = serde_yaml::from_str(scenario_yaml).unwrap();
        let base_dir = PathBuf::from("target/test_timeout");
        std::fs::create_dir_all(&base_dir).unwrap();

        let results_db = ResultsDB::new(&base_dir);
        let cache = Cache::new(&base_dir);

        let fixture_dir = PathBuf::from("fixtures/qipu");
        std::fs::create_dir_all(&fixture_dir).unwrap();
        let fixture_file = fixture_dir.join("timeout_test_override.yaml");
        std::fs::write(&fixture_file, scenario_yaml).unwrap();

        let cli_timeout = 300;
        let result = run_single_scenario(
            &scenario,
            "mock",
            "mock",
            false,
            true,
            cli_timeout,
            &base_dir,
            &results_db,
            &cache,
        );

        let _ = std::fs::remove_file(&fixture_file);

        assert!(result.is_ok(), "Should succeed with mock adapter");
    }

    #[test]
    fn test_cli_timeout_used_when_scenario_none() {
        let scenario_yaml = r#"
name: timeout_test_cli
description: "Test CLI timeout is used when scenario doesn't specify"
template_folder: qipu
task:
  prompt: "Create a note"
evaluation:
  gates:
    - type: min_notes
      count: 1
"#;
        let scenario: Scenario = serde_yaml::from_str(scenario_yaml).unwrap();
        let base_dir = PathBuf::from("target/test_timeout");
        std::fs::create_dir_all(&base_dir).unwrap();

        let results_db = ResultsDB::new(&base_dir);
        let cache = Cache::new(&base_dir);

        let fixture_dir = PathBuf::from("fixtures/qipu");
        std::fs::create_dir_all(&fixture_dir).unwrap();
        let fixture_file = fixture_dir.join("timeout_test_cli.yaml");
        std::fs::write(&fixture_file, scenario_yaml).unwrap();

        let cli_timeout = 60;
        let result = run_single_scenario(
            &scenario,
            "mock",
            "mock",
            false,
            true,
            cli_timeout,
            &base_dir,
            &results_db,
            &cache,
        );

        let _ = std::fs::remove_file(&fixture_file);

        assert!(result.is_ok(), "Should succeed with mock adapter");
    }
}

pub fn run_single_scenario(
    s: &crate::scenario::Scenario,
    tool: &str,
    model: &str,
    dry_run: bool,
    no_cache: bool,
    timeout_secs: u64,
    _base_dir: &std::path::Path,
    results_db: &ResultsDB,
    cache: &Cache,
) -> anyhow::Result<ResultRecord> {
    let scenario_path = format!("fixtures/{}/{}.yaml", s.template_folder, s.name);
    let scenario_yaml = std::fs::read_to_string(&scenario_path)?;
    let prompt = s.task.prompt.clone();
    let qipu_version = crate::results::get_qipu_version()?;

    // Effective timeout: scenario timeout overrides CLI timeout
    let effective_timeout = s
        .run
        .as_ref()
        .and_then(|r| r.timeout_secs)
        .unwrap_or(timeout_secs);

    // Set up template folder to get prime output for cache key
    println!(
        "Setting up environment for template folder: {}",
        s.template_folder
    );
    let env = crate::fixture::TestEnv::new(&s.name)?;
    env.setup_fixture(&s.template_folder)?;
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

        if let Some(setup) = &s.setup {
            println!("Running {} setup command(s)...", setup.commands.len());
            let runner = crate::session::SessionRunner::new();
            for (i, cmd) in setup.commands.iter().enumerate() {
                println!("  Command {}/{}: {}", i + 1, setup.commands.len(), cmd);
                let (output, exit_code) =
                    runner.run_command("sh", &["-c", cmd], &env.root, effective_timeout)?;
                if exit_code != 0 {
                    anyhow::bail!(
                        "Setup command {}/{} failed: {} exited with code {}. Output: {}",
                        i + 1,
                        setup.commands.len(),
                        cmd,
                        exit_code,
                        output
                    );
                }
            }
            println!("Setup complete.");
        }

        let start_time = Instant::now();
        println!("Running tool '{}' with model '{}'...", tool, model);
        let (output, exit_code, cost_opt, token_usage) =
            adapter.run(s, &env.root, Some(model), effective_timeout)?;
        let duration = start_time.elapsed();

        let cost = cost_opt.unwrap_or(0.0);

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
            token_usage: token_usage.clone().map(|t| crate::transcript::TokenUsage {
                input: t.input,
                output: t.output,
            }),
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
            token_usage: token_usage.map(|t| crate::transcript::TokenUsage {
                input: t.input,
                output: t.output,
            }),
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
