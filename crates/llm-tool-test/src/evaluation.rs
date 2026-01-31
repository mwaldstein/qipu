use crate::judge::{load_rubric, JudgeResponse};
use crate::scenario::{Gate, Scenario};
use crate::store_analysis::QualityMetrics;
use crate::transcript::EfficiencyMetrics;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

macro_rules! eval_gate {
    ($gate_type:expr, $expr:expr, |$result:ident| $closure:expr) => {
        match $expr {
            Ok($result) => {
                let (passed, message) = $closure;
                GateResult {
                    gate_type: $gate_type.to_string(),
                    passed,
                    message,
                }
            }
            Err(e) => GateResult {
                gate_type: $gate_type.to_string(),
                passed: false,
                message: format!("Evaluation error: {:#}", e),
            },
        }
    };
}

pub trait GateEvaluator {
    fn evaluate(&self, env_root: &Path) -> GateResult;
}

impl GateEvaluator for Gate {
    fn evaluate(&self, env_root: &Path) -> GateResult {
        match self {
            Gate::MinNotes { count } => eval_min_notes(*count, env_root),
            Gate::MinLinks { count } => eval_min_links(*count, env_root),
            Gate::SearchHit { query } => eval_search_hit(query, env_root),
            Gate::NoteExists { id } => eval_note_exists(id, env_root),
            Gate::LinkExists {
                from,
                to,
                link_type,
            } => eval_link_exists(from, to, link_type, env_root),
            Gate::TagExists { tag } => eval_tag_exists(tag, env_root),
            Gate::ContentContains { id, substring } => {
                eval_content_contains(id, substring, env_root)
            }
            Gate::CommandSucceeds { command } => eval_command_succeeds(command, env_root),
            Gate::DoctorPasses => eval_doctor_passes(env_root),
            Gate::NoTranscriptErrors => eval_no_transcript_errors(env_root),
        }
    }
}

fn eval_min_notes(count: usize, env_root: &Path) -> GateResult {
    eval_gate!(
        "MinNotes",
        crate::eval_helpers::count_notes(env_root).context("Failed to count notes"),
        |note_count| (
            note_count >= count,
            format!("Expected >= {}, found {}", count, note_count)
        )
    )
}

fn eval_min_links(count: usize, env_root: &Path) -> GateResult {
    eval_gate!(
        "MinLinks",
        crate::eval_helpers::count_links(env_root).context("Failed to count links"),
        |link_count| (
            link_count >= count,
            format!("Expected >= {}, found {}", count, link_count)
        )
    )
}

fn eval_search_hit(query: &str, env_root: &Path) -> GateResult {
    eval_gate!(
        "SearchHit",
        crate::eval_helpers::search_hit(query, env_root),
        |hit| (hit, format!("Query '{}' found: {}", query, hit))
    )
}

fn eval_note_exists(id: &str, env_root: &Path) -> GateResult {
    eval_gate!(
        "NoteExists",
        crate::eval_helpers::note_exists(id, env_root),
        |exists| (exists, format!("Note '{}' exists: {}", id, exists))
    )
}

fn eval_link_exists(from: &str, to: &str, link_type: &str, env_root: &Path) -> GateResult {
    eval_gate!(
        "LinkExists",
        crate::eval_helpers::link_exists(from, to, link_type, env_root),
        |exists| (
            exists,
            format!(
                "Link {} --[{}]--> {} exists: {}",
                from, link_type, to, exists
            )
        )
    )
}

fn eval_tag_exists(tag: &str, env_root: &Path) -> GateResult {
    eval_gate!(
        "TagExists",
        crate::eval_helpers::tag_exists(tag, env_root),
        |exists| (exists, format!("Tag '{}' exists: {}", tag, exists))
    )
}

fn eval_content_contains(id: &str, substring: &str, env_root: &Path) -> GateResult {
    eval_gate!(
        "ContentContains",
        crate::eval_helpers::content_contains(id, substring, env_root),
        |contains| (
            contains,
            format!("Note '{}' contains '{}': {}", id, substring, contains)
        )
    )
}

fn eval_command_succeeds(command: &str, env_root: &Path) -> GateResult {
    eval_gate!(
        "CommandSucceeds",
        crate::eval_helpers::command_succeeds(command, env_root),
        |succeeds| (
            succeeds,
            format!("Command '{}' succeeded: {}", command, succeeds)
        )
    )
}

fn eval_doctor_passes(env_root: &Path) -> GateResult {
    eval_gate!(
        "DoctorPasses",
        crate::eval_helpers::doctor_passes(env_root),
        |passes| (passes, format!("Store passes 'qipu doctor': {}", passes))
    )
}

fn eval_no_transcript_errors(env_root: &Path) -> GateResult {
    eval_gate!(
        "NoTranscriptErrors",
        crate::eval_helpers::no_transcript_errors(env_root),
        |no_errors| (
            no_errors,
            format!("Transcript has no command errors: {}", no_errors)
        )
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScoreTier {
    Excellent,
    Good,
    Acceptable,
    Poor,
}

impl ScoreTier {
    pub fn from_score(score: f64) -> Self {
        if score >= 0.9 {
            ScoreTier::Excellent
        } else if score >= 0.7 {
            ScoreTier::Good
        } else if score >= 0.5 {
            ScoreTier::Acceptable
        } else {
            ScoreTier::Poor
        }
    }
}

impl fmt::Display for ScoreTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScoreTier::Excellent => write!(f, "Excellent"),
            ScoreTier::Good => write!(f, "Good"),
            ScoreTier::Acceptable => write!(f, "Acceptable"),
            ScoreTier::Poor => write!(f, "Poor"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvaluationMetrics {
    pub gates_passed: usize,
    pub gates_total: usize,
    pub note_count: usize,
    pub link_count: usize,
    pub details: Vec<GateResult>,
    pub judge_score: Option<f64>,
    pub judge_response: Option<JudgeResponse>,
    pub efficiency: EfficiencyMetrics,
    pub quality: QualityMetrics,
    pub composite_score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GateResult {
    pub gate_type: String,
    pub passed: bool,
    pub message: String,
}

fn evaluate_gates(gates: &[Gate], env_root: &Path) -> (Vec<GateResult>, usize) {
    let mut details = Vec::new();
    let mut gates_passed = 0;

    for gate in gates {
        let result = gate.evaluate(env_root);

        if result.passed {
            println!("Gate {} passed: {}", result.gate_type, result.message);
            gates_passed += 1;
        } else {
            println!("Gate {} FAILED: {}", result.gate_type, result.message);
        }
        details.push(result);
    }

    (details, gates_passed)
}

fn run_judge_evaluation(
    scenario: &Scenario,
    env_root: &Path,
) -> Result<(Option<f64>, Option<JudgeResponse>)> {
    let judge_config = scenario.evaluation.judge.as_ref().unwrap();

    println!("Running LLM-as-judge evaluation...");
    let rubric_path = env_root.join(&judge_config.rubric);
    let _rubric = load_rubric(&rubric_path)
        .with_context(|| format!("Failed to load rubric from {}", judge_config.rubric))?;

    let transcript_path = env_root.join("artifacts/transcript.raw.txt");
    let store_path = env_root.join("artifacts/store_snapshot/export.json");

    let runner = crate::session::SessionRunner::new();
    let prompt = format!(
        r#"Evaluate this LLM tool interaction.

Task: {}

Files to review:
- @{} - The interaction transcript
- @{} - Store state after interaction

Use the rubric at {} for evaluation.

Return evaluation as JSON with this structure:
{{
  "scores": {{
    "criterion_id": <score_0_to_1>,
    ...
  }},
  "weighted_score": <weighted_average_0_to_1>,
  "confidence": <confidence_0_to_1>,
  "issues": ["issue1", "issue2", ...],
  "highlights": ["good_practice1", "good_practice2", ...]
}}

Provide JSON only, no additional text."#,
        scenario.task.prompt,
        transcript_path.display(),
        store_path.display(),
        rubric_path.display()
    );

    let (output, exit_code) = runner
        .run_command("opencode", &["run", &prompt], env_root, 300)
        .context("Judge execution failed")?;

    if exit_code != 0 {
        anyhow::bail!("Judge exited with code {}: {}", exit_code, output);
    }

    let response: JudgeResponse = serde_json::from_str(&output)
        .with_context(|| format!("Failed to parse judge response: {}", output))?;

    println!(
        "Judge score: {:.2} (confidence: {:.2})",
        response.weighted_score, response.confidence
    );
    if !response.issues.is_empty() {
        println!("Issues: {}", response.issues.join(", "));
    }
    if !response.highlights.is_empty() {
        println!("Highlights: {}", response.highlights.join(", "));
    }

    Ok((Some(response.weighted_score), Some(response)))
}

fn maybe_run_judge(
    scenario: &Scenario,
    env_root: &Path,
    no_judge: bool,
) -> Result<(Option<f64>, Option<JudgeResponse>)> {
    if let Some(judge_config) = &scenario.evaluation.judge {
        if judge_config.enabled && !no_judge {
            return run_judge_evaluation(scenario, env_root);
        }
    }
    Ok((None, None))
}

fn compute_efficiency_or_default(env_root: &Path) -> EfficiencyMetrics {
    crate::eval_helpers::compute_efficiency_metrics(env_root).unwrap_or(EfficiencyMetrics {
        total_commands: 0,
        unique_commands: 0,
        error_count: 0,
        retry_count: 0,
        help_invocations: 0,
        first_try_success_rate: 0.0,
        iteration_ratio: 0.0,
    })
}

fn compute_quality_or_default(env_root: &Path) -> QualityMetrics {
    crate::eval_helpers::compute_quality_metrics(env_root).unwrap_or_else(|_| QualityMetrics {
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
    })
}

fn build_metrics(
    scenario: &Scenario,
    env_root: &Path,
    details: Vec<GateResult>,
    gates_passed: usize,
    judge_score: Option<f64>,
    judge_response: Option<JudgeResponse>,
) -> EvaluationMetrics {
    let efficiency = compute_efficiency_or_default(env_root);
    let quality = compute_quality_or_default(env_root);
    let composite_score = crate::eval_helpers::compute_composite_score(
        judge_score,
        gates_passed,
        scenario.evaluation.gates.len(),
        &efficiency,
        &quality,
    );

    let note_count = crate::eval_helpers::count_notes(env_root).unwrap_or(0);
    let link_count = crate::eval_helpers::count_links(env_root).unwrap_or(0);

    EvaluationMetrics {
        gates_passed,
        gates_total: scenario.evaluation.gates.len(),
        note_count,
        link_count,
        details,
        judge_score,
        judge_response,
        efficiency,
        quality,
        composite_score,
    }
}

pub fn evaluate(scenario: &Scenario, env_root: &Path, no_judge: bool) -> Result<EvaluationMetrics> {
    println!("Evaluating results for scenario: {}", scenario.name);

    let (details, gates_passed) = evaluate_gates(&scenario.evaluation.gates, env_root);
    let (judge_score, judge_response) = maybe_run_judge(scenario, env_root, no_judge)?;
    let metrics = build_metrics(
        scenario,
        env_root,
        details,
        gates_passed,
        judge_score,
        judge_response,
    );

    Ok(metrics)
}
