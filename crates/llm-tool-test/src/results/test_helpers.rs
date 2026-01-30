use crate::results::db::ResultsDB;
use crate::results::types::{
    EfficiencyMetricsRecord, EvaluationMetricsRecord, QualityMetricsRecord, ResultRecord,
};
use chrono::Utc;
use std::collections::HashMap;
use tempfile::TempDir;

pub struct TestDb {
    /// TempDir field to keep the temporary directory alive.
    /// The directory is automatically cleaned up when TestDb is dropped.
    ///
    /// Note: This field is marked #[allow(dead_code)] because rustc considers it
    /// unused (never read), but storing it is essential to prevent the TempDir
    /// from being dropped prematurely.
    #[allow(dead_code)]
    temp_dir: TempDir,
    pub db: ResultsDB,
}

impl TestDb {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let db = ResultsDB::new(temp_dir.path());
        Self { temp_dir, db }
    }
}

pub fn create_test_record(id: &str) -> ResultRecord {
    create_test_record_with_scenario(id, "test-scenario")
}

pub fn create_test_record_with_scenario(id: &str, scenario_id: &str) -> ResultRecord {
    create_test_record_with_tool(id, scenario_id, "opencode")
}

pub fn create_test_record_with_tool(id: &str, scenario_id: &str, tool: &str) -> ResultRecord {
    ResultRecord {
        id: id.to_string(),
        scenario_id: scenario_id.to_string(),
        scenario_hash: "hash123".to_string(),
        tool: tool.to_string(),
        model: "gpt-4o".to_string(),
        qipu_commit: "abc123".to_string(),
        timestamp: Utc::now(),
        duration_secs: 45.5,
        cost_usd: 0.01,
        gates_passed: true,
        metrics: EvaluationMetricsRecord {
            gates_passed: 2,
            gates_total: 2,
            note_count: 1,
            link_count: 0,
            details: vec![],
            efficiency: EfficiencyMetricsRecord {
                total_commands: 3,
                unique_commands: 2,
                error_count: 0,
                retry_count: 1,
                help_invocations: 0,
                first_try_success_rate: 1.0,
                iteration_ratio: 1.5,
            },
            quality: QualityMetricsRecord {
                avg_title_length: 10.0,
                avg_body_length: 50.0,
                avg_tags_per_note: 2.0,
                notes_without_tags: 0,
                links_per_note: 0.0,
                orphan_notes: 1,
                link_type_diversity: 0,
                type_distribution: HashMap::new(),
                total_notes: 1,
                total_links: 0,
            },
            composite_score: 0.9,
        },
        judge_score: Some(0.9),
        outcome: "PASS".to_string(),
        transcript_path: "/path/to/transcript.txt".to_string(),
        cache_key: Some("cache-key-123".to_string()),
    }
}
