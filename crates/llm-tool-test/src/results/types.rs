use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultRecord {
    pub id: String,
    pub scenario_id: String,
    pub scenario_hash: String,
    pub tool: String,
    pub model: String,
    pub qipu_commit: String,
    pub timestamp: DateTime<Utc>,
    pub duration_secs: f64,
    pub cost_usd: f64,
    pub gates_passed: bool,
    pub metrics: EvaluationMetricsRecord,
    pub judge_score: Option<f64>,
    pub outcome: String,
    pub transcript_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationMetricsRecord {
    pub gates_passed: usize,
    pub gates_total: usize,
    pub note_count: usize,
    pub link_count: usize,
    pub details: Vec<GateResultRecord>,
    pub efficiency: EfficiencyMetricsRecord,
    pub quality: QualityMetricsRecord,
    pub composite_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyMetricsRecord {
    pub total_commands: usize,
    pub unique_commands: usize,
    pub error_count: usize,
    pub retry_count: usize,
    pub help_invocations: usize,
    pub first_try_success_rate: f64,
    pub iteration_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetricsRecord {
    pub avg_title_length: f64,
    pub avg_body_length: f64,
    pub avg_tags_per_note: f64,
    pub notes_without_tags: usize,
    pub links_per_note: f64,
    pub orphan_notes: usize,
    pub link_type_diversity: usize,
    pub type_distribution: HashMap<String, usize>,
    pub total_notes: usize,
    pub total_links: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResultRecord {
    pub gate_type: String,
    pub passed: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct CacheKey {
    pub scenario_hash: String,
    pub prompt_hash: String,
    pub prime_output_hash: String,
    pub tool: String,
    pub model: String,
    pub qipu_version: String,
}

impl CacheKey {
    pub fn compute(
        scenario_yaml: &str,
        prompt: &str,
        prime_output: &str,
        tool: &str,
        model: &str,
        qipu_version: &str,
    ) -> Self {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(scenario_yaml.as_bytes());
        let scenario_hash = format!("{:x}", hasher.finalize());

        let mut hasher = Sha256::new();
        hasher.update(prompt.as_bytes());
        let prompt_hash = format!("{:x}", hasher.finalize());

        let mut hasher = Sha256::new();
        hasher.update(prime_output.as_bytes());
        let prime_output_hash = format!("{:x}", hasher.finalize());

        Self {
            scenario_hash,
            prompt_hash,
            prime_output_hash,
            tool: tool.to_string(),
            model: model.to_string(),
            qipu_version: qipu_version.to_string(),
        }
    }

    pub fn as_string(&self) -> String {
        format!(
            "{}_{}_{}_{}_{}_{}",
            self.scenario_hash,
            self.prompt_hash,
            self.prime_output_hash,
            self.tool,
            self.model,
            self.qipu_version
        )
    }
}
