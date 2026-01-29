use super::super::types::{EfficiencyReport, EvaluationReport, QualityReport, RunReport};
use super::super::writer::TranscriptWriter;
use std::fs;

#[test]
fn test_write_report_basic() {
    let dir = tempfile::tempdir().unwrap();
    let writer = TranscriptWriter::new(dir.path().to_path_buf()).unwrap();

    let report = RunReport {
        scenario_id: "test_scenario".to_string(),
        tool: "amp".to_string(),
        model: "claude-3-5-sonnet".to_string(),
        timestamp: "2025-01-27T12:00:00Z".to_string(),
        duration_secs: 45.3,
        cost_usd: 0.0234,
        token_usage: None,
        outcome: "Pass".to_string(),
        gates_passed: 3,
        gates_total: 3,
        note_count: 5,
        link_count: 3,
        composite_score: Some(0.82),
        gate_details: vec![],
        efficiency: EfficiencyReport {
            total_commands: 10,
            unique_commands: 5,
            error_count: 0,
            first_try_success_rate: 0.9,
            iteration_ratio: 2.0,
        },
        quality: QualityReport {
            avg_title_length: 15.0,
            avg_body_length: 250.0,
            avg_tags_per_note: 1.2,
            links_per_note: 0.6,
            orphan_notes: 1,
        },
        setup_success: true,
        setup_commands: vec![],
    };

    writer.write_report(&report).unwrap();

    let report_path = dir.path().join("report.md");
    assert!(report_path.exists());

    let content = fs::read_to_string(&report_path).unwrap();
    assert!(content.contains("# Test Run Report"));
    assert!(content.contains("test_scenario"));
    assert!(content.contains("amp"));
    assert!(content.contains("claude-3-5-sonnet"));
    assert!(content.contains("45.30s"));
    assert!(content.contains("$0.0234"));
    assert!(content.contains("Pass"));
}

#[test]
fn test_write_evaluation_basic() {
    let dir = tempfile::tempdir().unwrap();
    let writer = TranscriptWriter::new(dir.path().to_path_buf()).unwrap();

    let evaluation = EvaluationReport {
        scenario_id: "test_scenario".to_string(),
        tool: "opencode".to_string(),
        model: "gpt-4o".to_string(),
        outcome: "Pass".to_string(),
        judge_score_1_to_5: Some(4.0),
        gates_passed: 2,
        gates_total: 3,
        note_count: 5,
        link_count: 2,
        duration_secs: 30.0,
        cost_usd: 0.015,
        composite_score: 0.82,
        judge_feedback: vec![
            "**Issues:**\nMinor formatting issue".to_string(),
            "**Highlights:**\nGood structure".to_string(),
            "**Criteria Scores:**\n- relevance: 0.85\n- clarity: 0.90".to_string(),
        ],
    };

    writer.write_evaluation(&evaluation).unwrap();

    let eval_path = dir.path().join("evaluation.md");
    assert!(eval_path.exists());

    let content = fs::read_to_string(&eval_path).unwrap();
    assert!(content.contains("# Evaluation"));
    assert!(content.contains("test_scenario"));
    assert!(content.contains("opencode"));
    assert!(content.contains("gpt-4o"));
    assert!(content.contains("**4** / 5"));
    assert!(content.contains("2/3"));
    assert!(content.contains("**Notes Created**: 5"));
    assert!(content.contains("**Links Created**: 2"));
    assert!(content.contains("30.00s"));
    assert!(content.contains("$0.0150"));
    assert!(content.contains("0.82"));
    assert!(content.contains("## Judge Feedback"));
    assert!(content.contains("**Issues:**"));
    assert!(content.contains("**Highlights:**"));
    assert!(content.contains("**Criteria Scores:**"));
    assert!(content.contains("## Human Review"));
    assert!(content.contains("<!--"));
    assert!(content.contains("Human Score: __/5"));
    assert!(content.contains("Further Human Notes:"));
    assert!(content.contains("-->"));
    assert!(content.contains("## Links"));
    assert!(content.contains("[Transcript](transcript.raw.txt)"));
    assert!(content.contains("[Metrics](metrics.json)"));
    assert!(content.contains("[Events](events.jsonl)"));
    assert!(content.contains("[Fixture](../fixture/)"));
    assert!(content.contains("[Store Snapshot](store_snapshot/export.json)"));
}

#[test]
fn test_write_evaluation_without_judge_score() {
    let dir = tempfile::tempdir().unwrap();
    let writer = TranscriptWriter::new(dir.path().to_path_buf()).unwrap();

    let evaluation = EvaluationReport {
        scenario_id: "test_scenario".to_string(),
        tool: "amp".to_string(),
        model: "claude-3-5-sonnet".to_string(),
        outcome: "Pass".to_string(),
        judge_score_1_to_5: None,
        gates_passed: 1,
        gates_total: 2,
        note_count: 3,
        link_count: 1,
        duration_secs: 20.0,
        cost_usd: 0.01,
        composite_score: 0.75,
        judge_feedback: vec![],
    };

    writer.write_evaluation(&evaluation).unwrap();

    let eval_path = dir.path().join("evaluation.md");
    let content = fs::read_to_string(&eval_path).unwrap();
    assert!(!content.contains("Judge Score"));
    assert!(!content.contains("## Judge Feedback"));
}
