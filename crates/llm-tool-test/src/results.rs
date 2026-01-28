use crate::pricing::get_model_pricing;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

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

pub struct ResultsDB {
    results_path: PathBuf,
}

impl ResultsDB {
    pub fn new(base_dir: &Path) -> Self {
        let results_dir = base_dir.join("results");
        std::fs::create_dir_all(&results_dir).ok();
        Self {
            results_path: results_dir.join("results.jsonl"),
        }
    }

    pub fn append(&self, record: &ResultRecord) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.results_path)
            .context("Failed to open results.jsonl")?;

        let line = serde_json::to_string(record)?;
        writeln!(file, "{}", line).context("Failed to write to results.jsonl")?;
        Ok(())
    }

    pub fn load_all(&self) -> Result<Vec<ResultRecord>> {
        if !self.results_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.results_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for line in reader.lines() {
            let line = line.context("Failed to read line from results.jsonl")?;
            let record: ResultRecord =
                serde_json::from_str(&line).context("Failed to parse result record")?;
            records.push(record);
        }

        Ok(records)
    }

    pub fn load_by_id(&self, id: &str) -> Result<Option<ResultRecord>> {
        let records = self.load_all()?;
        Ok(records.into_iter().find(|r| r.id == id))
    }
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

pub struct Cache {
    cache_dir: PathBuf,
}

impl Cache {
    pub fn new(base_dir: &Path) -> Self {
        let cache_dir = base_dir.join("cache");
        std::fs::create_dir_all(&cache_dir).ok();
        Self { cache_dir }
    }

    pub fn get(&self, key: &CacheKey) -> Option<ResultRecord> {
        let cache_file = self.cache_dir.join(key.as_string());
        if !cache_file.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&cache_file).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn put(&self, key: &CacheKey, record: &ResultRecord) -> Result<()> {
        let cache_file = self.cache_dir.join(key.as_string());
        let content = serde_json::to_string_pretty(record)?;
        std::fs::write(&cache_file, content)?;
        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        for entry in std::fs::read_dir(&self.cache_dir)? {
            let path = entry?.path();
            if path.is_file() {
                std::fs::remove_file(path)?;
            }
        }
        Ok(())
    }
}

pub fn generate_run_id() -> String {
    let now = Utc::now();
    format!("run-{}", now.format("%Y%m%d-%H%M%S-%f"))
}

pub fn get_qipu_version() -> Result<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir("..")
        .output()
        .ok();

    if let Some(output) = output {
        if output.status.success() {
            let commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Ok(commit[..8].to_string());
        }
    }

    Ok("unknown".to_string())
}

pub fn estimate_cost_from_tokens(model: &str, input_tokens: usize, output_tokens: usize) -> f64 {
    let Some(pricing) = get_model_pricing(model) else {
        return 0.0;
    };

    let input_cost = (input_tokens as f64 / 1000.0) * pricing.input_cost_per_1k_tokens;
    let output_cost = (output_tokens as f64 / 1000.0) * pricing.output_cost_per_1k_tokens;

    input_cost + output_cost
}

#[cfg(test)]
mod results_test_helpers;

#[cfg(test)]
mod tests {
    use super::*;

    use results_test_helpers::*;

    #[test]
    fn test_estimate_cost_claude_sonnet() {
        let cost = estimate_cost_from_tokens("claude-3-5-sonnet-20241022", 1000, 500);
        let expected_input_cost = (1000.0 / 1000.0) * 3.0;
        let expected_output_cost = (500.0 / 1000.0) * 15.0;
        assert!((cost - (expected_input_cost + expected_output_cost)).abs() < 0.001);
    }

    #[test]
    fn test_estimate_cost_claude_haiku() {
        let cost = estimate_cost_from_tokens("claude-3-5-haiku-20241022", 1000, 500);
        let expected_input_cost = (1000.0 / 1000.0) * 0.8;
        let expected_output_cost = (500.0 / 1000.0) * 4.0;
        assert!((cost - (expected_input_cost + expected_output_cost)).abs() < 0.001);
    }

    #[test]
    fn test_estimate_cost_gpt4o() {
        let cost = estimate_cost_from_tokens("gpt-4o", 1000, 500);
        let expected_input_cost = (1000.0 / 1000.0) * 2.5;
        let expected_output_cost = (500.0 / 1000.0) * 10.0;
        assert!((cost - (expected_input_cost + expected_output_cost)).abs() < 0.001);
    }

    #[test]
    fn test_estimate_cost_unknown_model() {
        let cost = estimate_cost_from_tokens("unknown-model", 1000, 500);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_estimate_cost_amp_smart() {
        let cost = estimate_cost_from_tokens("smart", 1000, 500);
        let expected_input_cost = (4000.0 / 4.0 / 1000.0) * 3.0;
        let expected_output_cost = (2000.0 / 4.0 / 1000.0) * 15.0;
        assert!((cost - (expected_input_cost + expected_output_cost)).abs() < 0.001);
    }

    #[test]
    fn test_estimate_cost_amp_free() {
        let cost = estimate_cost_from_tokens("free", 1000, 500);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_estimate_cost_case_insensitive() {
        let cost1 = estimate_cost_from_tokens("GPT-4O", 1000, 500);
        let cost2 = estimate_cost_from_tokens("gpt-4o", 1000, 500);
        assert!((cost1 - cost2).abs() < 0.001);
    }

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
        let tool2 = "amp";
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
            timestamp: Utc::now(),
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

    #[test]
    fn test_results_db_append_and_load_all() {
        let test_db = TestDb::new();

        let record1 = create_test_record("run-1");
        let record2 = create_test_record("run-2");

        test_db.db.append(&record1).unwrap();
        test_db.db.append(&record2).unwrap();

        let loaded = test_db.db.load_all().unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].id, "run-1");
        assert_eq!(loaded[1].id, "run-2");
    }

    #[test]
    fn test_results_db_load_empty() {
        let test_db = TestDb::new();

        let loaded = test_db.db.load_all().unwrap();
        assert_eq!(loaded.len(), 0);
    }

    #[test]
    fn test_results_db_load_by_id() {
        let test_db = TestDb::new();

        let record1 = create_test_record("run-1");
        let record2 = create_test_record("run-2");

        test_db.db.append(&record1).unwrap();
        test_db.db.append(&record2).unwrap();

        let loaded = test_db.db.load_by_id("run-1").unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().id, "run-1");

        let not_found = test_db.db.load_by_id("run-3").unwrap();
        assert!(not_found.is_none());
    }
}
