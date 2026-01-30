//! Type definitions for test results.
//!
//! This module defines all the data structures used to represent
//! test results, metrics, and cache keys.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A complete test run result record.
///
/// Contains all metadata and metrics for a single scenario execution,
/// including timing, cost, gate results, and quality metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultRecord {
    /// Unique run identifier
    pub id: String,
    /// Scenario identifier (filename without extension)
    pub scenario_id: String,
    /// Hash of the scenario YAML content
    pub scenario_hash: String,
    /// Tool name (e.g., "opencode", "claude-code")
    pub tool: String,
    /// Model name used for this run
    pub model: String,
    /// Qipu git commit hash (short)
    pub qipu_commit: String,
    /// Timestamp when the run completed
    pub timestamp: DateTime<Utc>,
    /// Total duration in seconds
    pub duration_secs: f64,
    /// Estimated cost in USD
    pub cost_usd: f64,
    /// Whether all gates passed
    pub gates_passed: bool,
    /// Detailed evaluation metrics
    pub metrics: EvaluationMetricsRecord,
    /// Optional LLM-as-judge score (0.0-1.0)
    pub judge_score: Option<f64>,
    /// Final outcome ("PASS", "FAIL", "ERROR")
    pub outcome: String,
    /// Path to the saved transcript file
    pub transcript_path: String,
    /// Optional cache key for this result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_key: Option<String>,
}

/// Evaluation metrics for a test run.
///
/// Aggregates gate results, efficiency metrics, quality metrics,
/// and a composite quality score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationMetricsRecord {
    /// Number of gates that passed
    pub gates_passed: usize,
    /// Total number of gates evaluated
    pub gates_total: usize,
    /// Total notes created in the store
    pub note_count: usize,
    /// Total links created in the store
    pub link_count: usize,
    /// Detailed results for each gate
    pub details: Vec<GateResultRecord>,
    /// Efficiency metrics
    pub efficiency: EfficiencyMetricsRecord,
    /// Quality metrics
    pub quality: QualityMetricsRecord,
    /// Composite quality score (0.0-1.0)
    pub composite_score: f64,
}

/// Efficiency metrics measuring tool interaction patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyMetricsRecord {
    /// Total number of commands executed
    pub total_commands: usize,
    /// Number of unique commands executed
    pub unique_commands: usize,
    /// Number of commands that resulted in errors
    pub error_count: usize,
    /// Number of command retries
    pub retry_count: usize,
    /// Number of help invocations
    pub help_invocations: usize,
    /// Rate of commands succeeding on first attempt (0.0-1.0)
    pub first_try_success_rate: f64,
    /// Ratio of total commands to unique commands
    pub iteration_ratio: f64,
}

/// Quality metrics measuring note and link characteristics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetricsRecord {
    /// Average note title length in characters
    pub avg_title_length: f64,
    /// Average note body length in characters
    pub avg_body_length: f64,
    /// Average number of tags per note
    pub avg_tags_per_note: f64,
    /// Number of notes without any tags
    pub notes_without_tags: usize,
    /// Average links per note
    pub links_per_note: f64,
    /// Number of notes with no incoming or outgoing links
    pub orphan_notes: usize,
    /// Number of distinct link types used
    pub link_type_diversity: usize,
    /// Distribution of note types
    pub type_distribution: HashMap<String, usize>,
    /// Total number of notes
    pub total_notes: usize,
    /// Total number of links
    pub total_links: usize,
}

/// Result of evaluating a single gate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResultRecord {
    /// Type of gate evaluated
    pub gate_type: String,
    /// Whether the gate passed
    pub passed: bool,
    /// Human-readable message about the result
    pub message: String,
}

/// Cache key for deduplicating test runs.
///
/// Computed from scenario content, prompt, prime output, tool,
/// model, and qipu version to uniquely identify a test configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct CacheKey {
    /// Hash of the scenario YAML content
    pub scenario_hash: String,
    /// Hash of the task prompt
    pub prompt_hash: String,
    /// Hash of the prime output
    pub prime_output_hash: String,
    /// Tool name
    pub tool: String,
    /// Model name
    pub model: String,
    /// Qipu version/commit
    pub qipu_version: String,
}

impl CacheKey {
    /// Compute a cache key from run parameters.
    ///
    /// Hashes the scenario YAML, prompt, and prime output using SHA256,
    /// and combines with tool, model, and version information.
    ///
    /// # Arguments
    ///
    /// * `scenario_yaml` - Raw scenario YAML content
    /// * `prompt` - Task prompt text
    /// * `prime_output` - Prime output text
    /// * `tool` - Tool name
    /// * `model` - Model name
    /// * `qipu_version` - Qipu version string
    ///
    /// # Returns
    ///
    /// A computed `CacheKey`
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

    /// Convert the cache key to a string representation.
    ///
    /// Used as the filename for cached results.
    ///
    /// # Returns
    ///
    /// A string combining all hash and identifier components
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_cache_key_compute_basic() {
        let scenario_yaml = "name: test\ntask:\n  prompt: test";
        let prompt = "Create a test note";
        let prime_output = "";
        let tool = "opencode";
        let model = "gpt-4o";
        let qipu_version = "abc123";

        let key = CacheKey::compute(
            scenario_yaml,
            prompt,
            prime_output,
            tool,
            model,
            qipu_version,
        );

        assert_eq!(key.tool, "opencode");
        assert_eq!(key.model, "gpt-4o");
        assert_eq!(key.qipu_version, "abc123");
        assert!(!key.scenario_hash.is_empty());
        assert!(!key.prompt_hash.is_empty());
        assert!(!key.prime_output_hash.is_empty());
    }

    #[test]
    fn test_cache_key_consistent_hashes() {
        let scenario_yaml = "name: test\ntask:\n  prompt: test";
        let prompt = "Create a test note";
        let prime_output = "";
        let tool = "opencode";
        let model = "gpt-4o";
        let qipu_version = "abc123";

        let key1 = CacheKey::compute(
            scenario_yaml,
            prompt,
            prime_output,
            tool,
            model,
            qipu_version,
        );
        let key2 = CacheKey::compute(
            scenario_yaml,
            prompt,
            prime_output,
            tool,
            model,
            qipu_version,
        );

        assert_eq!(key1.scenario_hash, key2.scenario_hash);
        assert_eq!(key1.prompt_hash, key2.prompt_hash);
        assert_eq!(key1.prime_output_hash, key2.prime_output_hash);
    }

    #[test]
    fn test_cache_key_different_scenarios() {
        let scenario1 = "name: test1\ntask:\n  prompt: test";
        let scenario2 = "name: test2\ntask:\n  prompt: test";
        let prompt = "Create a test note";
        let prime_output = "";
        let tool = "opencode";
        let model = "gpt-4o";
        let qipu_version = "abc123";

        let key1 = CacheKey::compute(scenario1, prompt, prime_output, tool, model, qipu_version);
        let key2 = CacheKey::compute(scenario2, prompt, prime_output, tool, model, qipu_version);

        assert_ne!(key1.scenario_hash, key2.scenario_hash);
        assert_eq!(key1.prompt_hash, key2.prompt_hash);
        assert_eq!(key1.prime_output_hash, key2.prime_output_hash);
    }

    #[test]
    fn test_cache_key_different_prompts() {
        let scenario_yaml = "name: test\ntask:\n  prompt: test";
        let prompt1 = "Create a test note";
        let prompt2 = "Create a different note";
        let prime_output = "";
        let tool = "opencode";
        let model = "gpt-4o";
        let qipu_version = "abc123";

        let key1 = CacheKey::compute(
            scenario_yaml,
            prompt1,
            prime_output,
            tool,
            model,
            qipu_version,
        );
        let key2 = CacheKey::compute(
            scenario_yaml,
            prompt2,
            prime_output,
            tool,
            model,
            qipu_version,
        );

        assert_eq!(key1.scenario_hash, key2.scenario_hash);
        assert_ne!(key1.prompt_hash, key2.prompt_hash);
        assert_eq!(key1.prime_output_hash, key2.prime_output_hash);
    }

    #[test]
    fn test_cache_key_different_tools() {
        let scenario_yaml = "name: test\ntask:\n  prompt: test";
        let prompt = "Create a test note";
        let prime_output = "";
        let tool1 = "opencode";
        let tool2 = "claude-code";
        let model = "gpt-4o";
        let qipu_version = "abc123";

        let key1 = CacheKey::compute(
            scenario_yaml,
            prompt,
            prime_output,
            tool1,
            model,
            qipu_version,
        );
        let key2 = CacheKey::compute(
            scenario_yaml,
            prompt,
            prime_output,
            tool2,
            model,
            qipu_version,
        );

        assert_eq!(key1.scenario_hash, key2.scenario_hash);
        assert_eq!(key1.prompt_hash, key2.prompt_hash);
        assert_eq!(key1.prime_output_hash, key2.prime_output_hash);
        assert_ne!(key1.tool, key2.tool);
    }

    #[test]
    fn test_cache_key_different_models() {
        let scenario_yaml = "name: test\ntask:\n  prompt: test";
        let prompt = "Create a test note";
        let prime_output = "";
        let tool = "opencode";
        let model1 = "gpt-4o";
        let model2 = "claude-sonnet-4";
        let qipu_version = "abc123";

        let key1 = CacheKey::compute(
            scenario_yaml,
            prompt,
            prime_output,
            tool,
            model1,
            qipu_version,
        );
        let key2 = CacheKey::compute(
            scenario_yaml,
            prompt,
            prime_output,
            tool,
            model2,
            qipu_version,
        );

        assert_eq!(key1.scenario_hash, key2.scenario_hash);
        assert_eq!(key1.prompt_hash, key2.prompt_hash);
        assert_eq!(key1.prime_output_hash, key2.prime_output_hash);
        assert_ne!(key1.model, key2.model);
    }

    #[test]
    fn test_cache_key_as_string() {
        let scenario_yaml = "name: test\ntask:\n  prompt: test";
        let prompt = "Create a test note";
        let prime_output = "";
        let tool = "opencode";
        let model = "gpt-4o";
        let qipu_version = "abc123";

        let key = CacheKey::compute(
            scenario_yaml,
            prompt,
            prime_output,
            tool,
            model,
            qipu_version,
        );
        let key_string = key.as_string();

        assert!(key_string.contains(&key.scenario_hash));
        assert!(key_string.contains(&key.prompt_hash));
        assert!(key_string.contains(&key.prime_output_hash));
        assert!(key_string.contains(&key.tool));
        assert!(key_string.contains(&key.model));
        assert!(key_string.contains(&key.qipu_version));
    }

    #[test]
    fn test_cache_key_equality() {
        let scenario_yaml = "name: test\ntask:\n  prompt: test";
        let prompt = "Create a test note";
        let prime_output = "";
        let tool = "opencode";
        let model = "gpt-4o";
        let qipu_version = "abc123";

        let key1 = CacheKey::compute(
            scenario_yaml,
            prompt,
            prime_output,
            tool,
            model,
            qipu_version,
        );
        let key2 = CacheKey::compute(
            scenario_yaml,
            prompt,
            prime_output,
            tool,
            model,
            qipu_version,
        );

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_cache_key_different_prime_outputs() {
        let scenario_yaml = "name: test\ntask:\n  prompt: test";
        let prompt = "Create a test note";
        let prime_output1 = "note1\nnote2";
        let prime_output2 = "note1\nnote2\nnote3";
        let tool = "opencode";
        let model = "gpt-4o";
        let qipu_version = "abc123";

        let key1 = CacheKey::compute(
            scenario_yaml,
            prompt,
            prime_output1,
            tool,
            model,
            qipu_version,
        );
        let key2 = CacheKey::compute(
            scenario_yaml,
            prompt,
            prime_output2,
            tool,
            model,
            qipu_version,
        );

        assert_eq!(key1.scenario_hash, key2.scenario_hash);
        assert_eq!(key1.prompt_hash, key2.prompt_hash);
        assert_ne!(key1.prime_output_hash, key2.prime_output_hash);
    }

    #[test]
    fn test_result_record_json_round_trip() {
        let original = ResultRecord {
            id: "test-run-id".to_string(),
            scenario_id: "test-scenario".to_string(),
            scenario_hash: "hash123".to_string(),
            tool: "opencode".to_string(),
            model: "gpt-4o".to_string(),
            qipu_commit: "abc123".to_string(),
            timestamp: chrono::Utc::now(),
            duration_secs: 45.5,
            cost_usd: 0.01,
            gates_passed: true,
            metrics: EvaluationMetricsRecord {
                gates_passed: 2,
                gates_total: 2,
                note_count: 1,
                link_count: 0,
                details: vec![GateResultRecord {
                    gate_type: "min_notes".to_string(),
                    passed: true,
                    message: "Passed".to_string(),
                }],
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
                composite_score: 0.95,
            },
            judge_score: Some(0.9),
            outcome: "PASS".to_string(),
            transcript_path: "/path/to/transcript.txt".to_string(),
            cache_key: Some("cache-key-123".to_string()),
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: ResultRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, original.id);
        assert_eq!(deserialized.scenario_id, original.scenario_id);
        assert_eq!(deserialized.scenario_hash, original.scenario_hash);
        assert_eq!(deserialized.tool, original.tool);
        assert_eq!(deserialized.model, original.model);
        assert_eq!(deserialized.qipu_commit, original.qipu_commit);
        assert_eq!(deserialized.timestamp, original.timestamp);
        assert_eq!(deserialized.duration_secs, original.duration_secs);
        assert_eq!(deserialized.cost_usd, original.cost_usd);
        assert_eq!(deserialized.gates_passed, original.gates_passed);
        assert_eq!(
            deserialized.metrics.gates_passed,
            original.metrics.gates_passed
        );
        assert_eq!(
            deserialized.metrics.efficiency.total_commands,
            original.metrics.efficiency.total_commands
        );
        assert_eq!(deserialized.judge_score, original.judge_score);
        assert_eq!(deserialized.outcome, original.outcome);
        assert_eq!(deserialized.transcript_path, original.transcript_path);
        assert_eq!(deserialized.cache_key, original.cache_key);
    }

    #[test]
    fn test_result_record_json_skip_none_cache_key() {
        let record = ResultRecord {
            id: "test-run-id".to_string(),
            scenario_id: "test-scenario".to_string(),
            scenario_hash: "hash123".to_string(),
            tool: "opencode".to_string(),
            model: "gpt-4o".to_string(),
            qipu_commit: "abc123".to_string(),
            timestamp: chrono::Utc::now(),
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
                composite_score: 0.85,
            },
            judge_score: None,
            outcome: "PASS".to_string(),
            transcript_path: "/path/to/transcript.txt".to_string(),
            cache_key: None,
        };

        let json = serde_json::to_string(&record).unwrap();
        assert!(!json.contains("\"cache_key\""));
        assert!(json.contains("\"judge_score\":null"));
    }
}
