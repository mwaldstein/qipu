pub mod cache;
pub mod db;
pub mod types;
pub mod utils;

#[cfg(test)]
pub mod test_helpers;

pub use cache::Cache;
pub use db::ResultsDB;
pub use types::*;
pub use utils::{estimate_cost_from_tokens, generate_run_id, get_qipu_version};

#[cfg(test)]
mod tests {
    use super::*;
    use test_helpers::*;
    use types::CacheKey;

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
        use std::collections::HashMap;
        use types::GateResultRecord;

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
        use std::collections::HashMap;

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
