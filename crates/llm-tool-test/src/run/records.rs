use crate::evaluation::EvaluationMetrics;
use crate::output;
use crate::results::{Cache, CacheKey, EvaluationMetricsRecord, ResultRecord, ResultsDB};
use crate::scenario::Scenario;
use std::path::PathBuf;

pub fn build_result_record(
    s: &Scenario,
    tool: &str,
    model: &str,
    qipu_version: &str,
    cache_key: &CacheKey,
    metrics: EvaluationMetrics,
    outcome: String,
    duration_secs: f64,
    cost: f64,
    transcript_path: String,
) -> ResultRecord {
    use crate::results::{EfficiencyMetricsRecord, GateResultRecord, QualityMetricsRecord};

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

pub fn handle_dry_run(
    s: &Scenario,
    tool: &str,
    model: &str,
    qipu_version: &str,
    cache_key: &CacheKey,
) -> anyhow::Result<ResultRecord> {
    use crate::results::{EfficiencyMetricsRecord, EvaluationMetricsRecord, QualityMetricsRecord};

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

pub fn finalize_execution(
    results_db: &ResultsDB,
    cache: &Cache,
    cache_key: &CacheKey,
    record: &ResultRecord,
    results_dir: &PathBuf,
    setup_success: bool,
) -> anyhow::Result<ResultRecord> {
    results_db.append(record)?;
    cache.put(cache_key, record)?;

    let metrics_json = serde_json::to_string_pretty(&record.metrics)?;
    std::fs::write(results_dir.join("metrics.json"), metrics_json)?;

    println!("\nRun completed: {}", record.id);
    println!("Artifacts written to: {}", results_dir.display());

    if !setup_success {
        println!("\nWarning: Setup commands failed. Results may be invalid.");
    }

    output::print_result_summary(record);
    Ok(record.clone())
}
