//! Tests for result types.

use super::*;

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
    use std::collections::HashMap;

    let original = ResultRecord {
        id: "test-run-id".to_string(),
        scenario_id: "test-scenario".to_string(),
        scenario_hash: "hash123".to_string(),
        tool: "opencode".to_string(),
        model: "gpt-4o".to_string(),
        qipu_commit: "abc123".to_string(),
        timestamp: chrono::Utc::now(),
        duration_secs: 45.5,
        cost_usd: Some(0.01),
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
        cost_usd: Some(0.01),
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
