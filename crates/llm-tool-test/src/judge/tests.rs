//! Tests for judge module.

use super::eval::{build_judge_prompt, run_judge_with_client};
use super::rubric::load_rubric;
use super::types::{Criterion, OutputFormat, Rubric};
use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

#[test]
fn test_build_judge_prompt() {
    let rubric = Rubric {
        criteria: vec![Criterion {
            id: "test_criterion".to_string(),
            weight: 1.0,
            description: "Test description".to_string(),
        }],
        output: OutputFormat {
            format: "json".to_string(),
            require_fields: vec!["scores".to_string()],
        },
    };

    let prompt = build_judge_prompt("Test task", "Test transcript", "{}", &rubric);

    assert!(prompt.contains("Test task"));
    assert!(prompt.contains("Test transcript"));
    assert!(prompt.contains("test_criterion"));
}

#[test]
fn test_load_rubric_weights_sum_to_one() {
    let rubric_yaml = r#"
criteria:
  - id: relevance
    weight: 0.35
    description: "Notes directly address the task prompt"
  - id: coherence
    weight: 0.35
    description: "Notes are logically connected"
  - id: granularity
    weight: 0.30
    description: "Notes are appropriately scoped"
output:
  format: json
  require_fields:
    - scores
    - weighted_score
"#;
    let temp_dir = tempfile::tempdir().unwrap();
    let rubric_path = temp_dir.path().join("test_rubric.yaml");
    std::fs::write(&rubric_path, rubric_yaml).unwrap();

    let rubric = load_rubric(&rubric_path).unwrap();
    assert_eq!(rubric.criteria.len(), 3);
    assert_eq!(rubric.criteria[0].id, "relevance");
    assert_eq!(rubric.criteria[0].weight, 0.35);
}

#[test]
fn test_load_rubric_weights_sum_error() {
    let rubric_yaml = r#"
criteria:
  - id: criterion1
    weight: 0.5
    description: "First criterion"
  - id: criterion2
    weight: 0.4
    description: "Second criterion"
output:
  format: json
  require_fields:
    - scores
"#;
    let temp_dir = tempfile::tempdir().unwrap();
    let rubric_path = temp_dir.path().join("bad_rubric.yaml");
    std::fs::write(&rubric_path, rubric_yaml).unwrap();

    let result = load_rubric(&rubric_path);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("must sum to 1.0"));
}

#[tokio::test]
async fn test_run_judge_missing_api_key() {
    use super::eval::run_judge;

    let rubric = Rubric {
        criteria: vec![Criterion {
            id: "test".to_string(),
            weight: 1.0,
            description: "Test".to_string(),
        }],
        output: OutputFormat {
            format: "json".to_string(),
            require_fields: vec![],
        },
    };

    std::env::remove_var("OPENAI_API_KEY");
    std::env::remove_var("LLM_TOOL_TEST_API_KEY");

    let result = run_judge("gpt-4o-mini", "transcript", "{}", "task", &rubric).await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("environment variable must be set"));
}

#[tokio::test]
async fn test_run_judge_success() {
    let mock_server = MockServer::start().await;
    let api_url = mock_server.uri();

    let rubric = Rubric {
        criteria: vec![Criterion {
            id: "test_criterion".to_string(),
            weight: 1.0,
            description: "Test criterion".to_string(),
        }],
        output: OutputFormat {
            format: "json".to_string(),
            require_fields: vec![],
        },
    };

    let mock_response = serde_json::json!({
        "choices": [{
            "message": {
                "content": r#"{
                    "scores": {"test_criterion": 0.85},
                    "weighted_score": 0.85,
                    "confidence": 0.9,
                    "issues": [],
                    "highlights": ["Good execution"]
                }"#
            }
        }]
    });

    Mock::given(matchers::method("POST"))
        .and(matchers::path("/v1/chat/completions"))
        .and(matchers::header("authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
        .mount(&mock_server)
        .await;

    let result = run_judge_with_client(
        "gpt-4o-mini",
        "transcript summary",
        r#"{"notes": []}"#,
        "task description",
        &rubric,
        &api_url,
        "test-key",
    )
    .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.scores.get("test_criterion"), Some(&0.85));
    assert_eq!(response.weighted_score, 0.85);
    assert_eq!(response.confidence, 0.9);
}

#[tokio::test]
async fn test_run_judge_api_error() {
    let mock_server = MockServer::start().await;
    let api_url = mock_server.uri();

    let rubric = Rubric {
        criteria: vec![Criterion {
            id: "test".to_string(),
            weight: 1.0,
            description: "Test".to_string(),
        }],
        output: OutputFormat {
            format: "json".to_string(),
            require_fields: vec![],
        },
    };

    Mock::given(matchers::method("POST"))
        .and(matchers::path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(429).set_body_json(serde_json::json!({
            "error": {"message": "Rate limit exceeded"}
        })))
        .mount(&mock_server)
        .await;

    let result = run_judge_with_client(
        "gpt-4o-mini",
        "transcript",
        "{}",
        "task",
        &rubric,
        &api_url,
        "test-key",
    )
    .await;

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("OpenAI API request failed"));
    assert!(err_msg.contains("429"));
}

#[tokio::test]
async fn test_run_judge_invalid_response_format() {
    let mock_server = MockServer::start().await;
    let api_url = mock_server.uri();

    let rubric = Rubric {
        criteria: vec![Criterion {
            id: "test".to_string(),
            weight: 1.0,
            description: "Test".to_string(),
        }],
        output: OutputFormat {
            format: "json".to_string(),
            require_fields: vec![],
        },
    };

    Mock::given(matchers::method("POST"))
        .and(matchers::path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "choices": [{"message": {"content": "not valid json"}}]
        })))
        .mount(&mock_server)
        .await;

    let result = run_judge_with_client(
        "gpt-4o-mini",
        "transcript",
        "{}",
        "task",
        &rubric,
        &api_url,
        "test-key",
    )
    .await;

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Failed to parse judge response JSON"));
}

#[tokio::test]
async fn test_run_judge_missing_content_field() {
    let mock_server = MockServer::start().await;
    let api_url = mock_server.uri();

    let rubric = Rubric {
        criteria: vec![Criterion {
            id: "test".to_string(),
            weight: 1.0,
            description: "Test".to_string(),
        }],
        output: OutputFormat {
            format: "json".to_string(),
            require_fields: vec![],
        },
    };

    Mock::given(matchers::method("POST"))
        .and(matchers::path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "choices": [{"message": {}}]
        })))
        .mount(&mock_server)
        .await;

    let result = run_judge_with_client(
        "gpt-4o-mini",
        "transcript",
        "{}",
        "task",
        &rubric,
        &api_url,
        "test-key",
    )
    .await;

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Invalid OpenAI API response format"));
}
