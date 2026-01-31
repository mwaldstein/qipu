pub mod utils;

use crate::adapter::{
    claude_code::ClaudeCodeAdapter, mock::MockAdapter, opencode::OpenCodeAdapter, ToolAdapter,
};
use crate::output;
use crate::results::{
    Cache, CacheKey, EfficiencyMetricsRecord, EvaluationMetricsRecord, GateResultRecord,
    QualityMetricsRecord, ResultRecord, ResultsDB,
};
use crate::run::utils::{copy_dir_recursive, get_results_dir};

use std::path::PathBuf;
use std::time::Instant;

fn compute_cache_key(
    scenario_yaml: &str,
    prompt: &str,
    prime_output: &str,
    tool: &str,
    model: &str,
    qipu_version: &str,
) -> CacheKey {
    CacheKey::compute(
        scenario_yaml,
        prompt,
        prime_output,
        tool,
        model,
        qipu_version,
    )
}

fn check_cache(cache: &Cache, cache_key: &CacheKey) -> anyhow::Result<Option<ResultRecord>> {
    Ok(cache.get(cache_key))
}

fn setup_scenario_env(
    s: &crate::scenario::Scenario,
) -> anyhow::Result<(crate::fixture::TestEnv, String, String)> {
    let fixtures_path = if PathBuf::from("crates/llm-tool-test/fixtures").exists() {
        "crates/llm-tool-test/fixtures"
    } else {
        "fixtures"
    };
    let scenario_path = format!("{}/{}.yaml", fixtures_path, s.name);
    let scenario_yaml = std::fs::read_to_string(&scenario_path)?;
    let prompt = s.task.prompt.clone();

    println!(
        "Setting up environment for template folder: {}",
        s.template_folder
    );
    let env = crate::fixture::TestEnv::new(&s.name)?;
    env.setup_fixture(&s.template_folder)?;
    println!("Environment created at: {:?}", env.root);

    let _prime_output = env.get_prime_output();

    Ok((env, scenario_yaml, prompt))
}

fn execute_setup_commands(
    setup: &crate::scenario::Setup,
    env: &crate::fixture::TestEnv,
    writer: &crate::transcript::TranscriptWriter,
    effective_timeout: u64,
) -> anyhow::Result<(bool, Vec<(String, bool, String)>)> {
    println!("Running {} setup command(s)...", setup.commands.len());
    let runner = crate::session::SessionRunner::new();
    let mut setup_success = true;
    let mut setup_commands: Vec<(String, bool, String)> = Vec::new();

    for (i, cmd) in setup.commands.iter().enumerate() {
        println!("  Command {}/{}: {}", i + 1, setup.commands.len(), cmd);
        let (output, exit_code) =
            runner.run_command("sh", &["-c", cmd], &env.root, effective_timeout)?;

        let success = exit_code == 0;
        setup_commands.push((cmd.to_string(), success, output.clone()));

        writer.append_event(&serde_json::json!({
            "type": "setup_command",
            "index": i,
            "command": cmd,
            "exit_code": exit_code,
            "output": output,
            "success": success,
        }))?;

        if !success {
            setup_success = false;
            println!("  Command failed with exit code {}", exit_code);
        }
    }
    println!("Setup complete.");

    Ok((setup_success, setup_commands))
}

fn execute_tool(
    adapter: &Box<dyn ToolAdapter>,
    s: &crate::scenario::Scenario,
    env: &crate::fixture::TestEnv,
    tool: &str,
    model: &str,
    effective_timeout: u64,
) -> anyhow::Result<(String, i32, f64, Option<crate::adapter::TokenUsage>)> {
    let start_time = Instant::now();
    println!("Running tool '{}' with model '{}'...", tool, model);
    let (output, exit_code, cost_opt, token_usage) =
        adapter.run(s, &env.root, Some(model), effective_timeout)?;
    let _duration = start_time.elapsed();

    let cost = cost_opt.unwrap_or(0.0);

    Ok((output, exit_code, cost, token_usage))
}

fn write_transcript_files(
    writer: &crate::transcript::TranscriptWriter,
    s: &crate::scenario::Scenario,
    tool: &str,
    model: &str,
    qipu_version: &str,
    cache_key: &CacheKey,
    output: &str,
    exit_code: i32,
    cost: f64,
    token_usage: Option<crate::adapter::TokenUsage>,
    duration: std::time::Duration,
    metrics: &crate::evaluation::EvaluationMetrics,
    outcome: &str,
    setup_success: bool,
    setup_commands: Vec<(String, bool, String)>,
    env: &crate::fixture::TestEnv,
) -> anyhow::Result<()> {
    writer.write_raw(output)?;
    writer.append_event(&serde_json::json!({
        "type": "execution",
        "tool": tool,
        "output": output,
        "exit_code": exit_code,
        "cost_usd": cost
    }))?;

    writer.create_store_snapshot(&env.root)?;

    let run_metadata = crate::transcript::RunMetadata {
        scenario_id: s.name.clone(),
        scenario_hash: cache_key.scenario_hash.clone(),
        tool: tool.to_string(),
        model: model.to_string(),
        qipu_version: qipu_version.to_string(),
        qipu_commit: qipu_version.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        duration_secs: duration.as_secs_f64(),
        cost_estimate_usd: cost,
        token_usage: token_usage.clone().map(|t| crate::transcript::TokenUsage {
            input: t.input,
            output: t.output,
        }),
    };
    writer.write_run_metadata(&run_metadata)?;

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
        outcome: outcome.to_string(),
        gates_passed: metrics.gates_passed,
        gates_total: metrics.gates_total,
        note_count: metrics.note_count,
        link_count: metrics.link_count,
        composite_score: Some(metrics.composite_score),
        gate_details: metrics
            .details
            .iter()
            .map(|d| crate::transcript::types::GateDetail {
                gate_type: d.gate_type.clone(),
                passed: d.passed,
                message: d.message.clone(),
            })
            .collect(),
        efficiency: crate::transcript::types::EfficiencyReport {
            total_commands: metrics.efficiency.total_commands,
            unique_commands: metrics.efficiency.unique_commands,
            error_count: metrics.efficiency.error_count,
            first_try_success_rate: metrics.efficiency.first_try_success_rate,
            iteration_ratio: metrics.efficiency.iteration_ratio,
        },
        quality: crate::transcript::types::QualityReport {
            avg_title_length: metrics.quality.avg_title_length,
            avg_body_length: metrics.quality.avg_body_length,
            avg_tags_per_note: metrics.quality.avg_tags_per_note,
            links_per_note: metrics.quality.links_per_note,
            orphan_notes: metrics.quality.orphan_notes,
        },
        setup_success,
        setup_commands: setup_commands
            .into_iter()
            .map(
                |(cmd, success, output)| crate::transcript::types::SetupCommandResult {
                    command: cmd,
                    success,
                    output,
                },
            )
            .collect(),
    };
    writer.write_report(&report)?;

    let judge_score_1_to_5 = metrics.judge_score.map(|score| (score * 5.0).round());
    let judge_feedback = if let Some(ref response) = metrics.judge_response {
        let mut feedback = Vec::new();
        if !response.issues.is_empty() {
            feedback.push(format!("**Issues:**\n{}", response.issues.join("\n")));
        }
        if !response.highlights.is_empty() {
            feedback.push(format!(
                "**Highlights:**\n{}",
                response.highlights.join("\n")
            ));
        }
        if !response.scores.is_empty() {
            let scores_text: Vec<String> = response
                .scores
                .iter()
                .map(|(k, v)| format!("- {}: {:.2}", k, v))
                .collect();
            feedback.push(format!("**Criteria Scores:**\n{}", scores_text.join("\n")));
        }
        feedback
    } else {
        Vec::new()
    };

    let evaluation = crate::transcript::EvaluationReport {
        scenario_id: s.name.clone(),
        tool: tool.to_string(),
        model: model.to_string(),
        outcome: outcome.to_string(),
        judge_score_1_to_5,
        gates_passed: metrics.gates_passed,
        gates_total: metrics.gates_total,
        note_count: metrics.note_count,
        link_count: metrics.link_count,
        duration_secs: duration.as_secs_f64(),
        cost_usd: cost,
        composite_score: metrics.composite_score,
        judge_feedback,
    };
    writer.write_evaluation(&evaluation)?;

    Ok(())
}

fn build_result_record(
    s: &crate::scenario::Scenario,
    tool: &str,
    model: &str,
    qipu_version: &str,
    cache_key: &CacheKey,
    metrics: crate::evaluation::EvaluationMetrics,
    outcome: String,
    duration_secs: f64,
    cost: f64,
    transcript_path: String,
) -> ResultRecord {
    ResultRecord {
        id: crate::results::generate_run_id(),
        scenario_id: s.name.clone(),
        scenario_hash: cache_key.scenario_hash.clone(),
        tool: tool.to_string(),
        model: model.to_string(),
        qipu_commit: qipu_version.to_string(),
        timestamp: chrono::Utc::now(),
        duration_secs,
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
    }
}

fn save_artifacts(
    transcript_dir: &PathBuf,
    env: &crate::fixture::TestEnv,
    tool: &str,
    model: &str,
    scenario_name: &str,
) -> anyhow::Result<PathBuf> {
    let results_dir = get_results_dir(tool, model, scenario_name);
    std::fs::create_dir_all(&results_dir)?;

    copy_dir_recursive(transcript_dir, &results_dir)?;

    let fixture_dir = results_dir.join("fixture");
    copy_dir_recursive(&env.root, &fixture_dir)?;

    println!("\nArtifacts written to: {}", results_dir.display());

    Ok(results_dir)
}

fn handle_dry_run(
    s: &crate::scenario::Scenario,
    tool: &str,
    model: &str,
    qipu_version: &str,
    cache_key: &CacheKey,
) -> anyhow::Result<ResultRecord> {
    println!("Dry run - skipping execution");

    let record = ResultRecord {
        id: crate::results::generate_run_id(),
        scenario_id: s.name.clone(),
        scenario_hash: cache_key.scenario_hash.clone(),
        tool: tool.to_string(),
        model: model.to_string(),
        qipu_commit: qipu_version.to_string(),
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
    };

    output::print_result_summary(&record);
    Ok(record)
}

fn prepare_writer_and_setup(
    env: &crate::fixture::TestEnv,
    s: &crate::scenario::Scenario,
    effective_timeout: u64,
) -> anyhow::Result<(
    PathBuf,
    crate::transcript::TranscriptWriter,
    bool,
    Vec<(String, bool, String)>,
)> {
    let transcript_dir = env.root.join("artifacts");
    std::fs::create_dir_all(&transcript_dir)?;
    let writer = crate::transcript::TranscriptWriter::new(transcript_dir.clone())?;

    let (setup_success, setup_commands) = if let Some(setup) = &s.setup {
        execute_setup_commands(setup, env, &writer, effective_timeout)?
    } else {
        (true, vec![])
    };

    Ok((transcript_dir, writer, setup_success, setup_commands))
}

fn run_evaluation_flow(
    adapter: &Box<dyn ToolAdapter>,
    s: &crate::scenario::Scenario,
    env: &crate::fixture::TestEnv,
    tool: &str,
    model: &str,
    effective_timeout: u64,
    no_judge: bool,
) -> anyhow::Result<(
    String,
    i32,
    f64,
    Option<crate::adapter::TokenUsage>,
    std::time::Duration,
    crate::evaluation::EvaluationMetrics,
)> {
    let (output, exit_code, cost, token_usage) =
        execute_tool(adapter, s, env, tool, model, effective_timeout)?;
    let duration = std::time::Instant::now().elapsed();

    println!("Running evaluation...");
    let metrics = crate::evaluation::evaluate(s, &env.root, no_judge)?;
    println!("Evaluation metrics: {:?}", metrics);

    Ok((output, exit_code, cost, token_usage, duration, metrics))
}

fn create_adapter_and_check(tool: &str) -> anyhow::Result<Box<dyn ToolAdapter>> {
    let adapter: Box<dyn ToolAdapter> = match tool {
        "claude-code" => Box::new(ClaudeCodeAdapter),
        "mock" => Box::new(MockAdapter),
        "opencode" => Box::new(OpenCodeAdapter),
        _ => anyhow::bail!("Unknown tool: {}", tool),
    };

    println!("Checking availability for tool: {}", tool);
    adapter.check_availability()?;

    Ok(adapter)
}

fn determine_outcome(metrics: &crate::evaluation::EvaluationMetrics) -> String {
    if metrics.gates_passed < metrics.gates_total {
        format!(
            "Fail: {}/{} gates passed",
            metrics.gates_passed, metrics.gates_total
        )
    } else {
        "Pass".to_string()
    }
}

fn finalize_execution(
    results_db: &ResultsDB,
    cache: &Cache,
    cache_key: &CacheKey,
    record: &ResultRecord,
    transcript_dir: &PathBuf,
    env: &crate::fixture::TestEnv,
    tool: &str,
    model: &str,
    scenario_name: &str,
    setup_success: bool,
) -> anyhow::Result<ResultRecord> {
    results_db.append(record)?;
    cache.put(cache_key, record)?;

    let results_dir = save_artifacts(transcript_dir, env, tool, model, scenario_name)?;

    let metrics_json = serde_json::to_string_pretty(&record.metrics)?;
    std::fs::write(results_dir.join("metrics.json"), metrics_json)?;

    println!("\nRun completed: {}", record.id);
    println!("Transcript written to: {}", record.transcript_path);

    if !setup_success {
        println!("\nWarning: Setup commands failed. Results may be invalid.");
    }

    output::print_result_summary(record);
    Ok(record.clone())
}

#[allow(clippy::too_many_arguments)]
pub fn run_single_scenario(
    s: &crate::scenario::Scenario,
    tool: &str,
    model: &str,
    dry_run: bool,
    no_cache: bool,
    timeout_secs: u64,
    no_judge: bool,
    _base_dir: &std::path::Path,
    results_db: &ResultsDB,
    cache: &Cache,
) -> anyhow::Result<ResultRecord> {
    let effective_timeout = s
        .run
        .as_ref()
        .and_then(|r| r.timeout_secs)
        .unwrap_or(timeout_secs);

    let qipu_version = crate::results::get_qipu_version()?;

    let (env, scenario_yaml, prompt) = setup_scenario_env(s)?;
    let prime_output = env.get_prime_output();
    let cache_key = compute_cache_key(
        &scenario_yaml,
        &prompt,
        &prime_output,
        tool,
        model,
        &qipu_version,
    );

    if !no_cache {
        if let Some(cached) = check_cache(cache, &cache_key)? {
            println!("Cache HIT! Using cached result: {}", cached.id);
            output::print_result_summary(&cached);
            return Ok(cached);
        }
    }

    if dry_run {
        return handle_dry_run(s, tool, model, &qipu_version, &cache_key);
    }

    let adapter = create_adapter_and_check(tool)?;

    let (transcript_dir, writer, setup_success, setup_commands) =
        prepare_writer_and_setup(&env, &s, effective_timeout)?;

    let (output, exit_code, cost, token_usage, duration, metrics) =
        run_evaluation_flow(&adapter, s, &env, tool, model, effective_timeout, no_judge)?;

    let outcome = determine_outcome(&metrics);

    write_transcript_files(
        &writer,
        s,
        tool,
        model,
        &qipu_version,
        &cache_key,
        &output,
        exit_code,
        cost,
        token_usage,
        duration,
        &metrics,
        &outcome,
        setup_success,
        setup_commands,
        &env,
    )?;

    let transcript_path = transcript_dir.to_string_lossy().to_string();
    let record = build_result_record(
        s,
        tool,
        model,
        &qipu_version,
        &cache_key,
        metrics,
        outcome,
        duration.as_secs_f64(),
        cost,
        transcript_path,
    );

    finalize_execution(
        results_db,
        cache,
        &cache_key,
        &record,
        &transcript_dir,
        &env,
        tool,
        model,
        &s.name,
        setup_success,
    )
}

#[cfg(test)]
mod tests;
